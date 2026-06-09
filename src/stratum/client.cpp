// Copyright (c) 2026 The Vericonomy developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#include <stratum/client.h>

#include <netbase.h>
#include <util/strencodings.h>
#include <util/system.h>

#include <univalue.h>

#include <algorithm>
#include <cerrno>
#include <cstring>

namespace stratum {
namespace {

std::vector<unsigned char> ReverseBuffer(const std::vector<unsigned char>& buf)
{
    std::vector<unsigned char> out(buf.rbegin(), buf.rend());
    return out;
}

} // namespace

StratumClient::StratumClient() = default;

StratumClient::~StratumClient()
{
    Disconnect();
}

void StratumClient::Disconnect()
{
    if (m_socket != INVALID_SOCKET) {
        CloseSocket(m_socket);
        m_socket = INVALID_SOCKET;
    }
    m_read_buf.clear();
    m_pending.clear();
}

bool StratumClient::Connect(const std::string& host, uint16_t port, std::string& error_out)
{
    Disconnect();
    CService addr;
    if (!Lookup(host.c_str(), addr, port, true)) {
        error_out = strprintf("could not resolve stratum host %s:%u", host, port);
        return false;
    }
    SOCKET h = CreateSocket(addr);
    if (h == INVALID_SOCKET) {
        error_out = "failed to create socket";
        return false;
    }
    if (!ConnectSocketDirectly(addr, h, 10000, true)) {
        CloseSocket(h);
        error_out = strprintf("connect %s:%u failed", host, port);
        return false;
    }
    // CreateSocket() enables non-blocking mode for the P2P stack. Stratum needs a
    // blocking socket: otherwise recv() returns WSAEWOULDBLOCK when the pool is
    // idle between jobs and we treat it as a disconnect after a few seconds.
    if (!SetSocketNonBlocking(h, false)) {
        CloseSocket(h);
        error_out = "failed to set stratum socket blocking mode";
        return false;
    }

    // Short read timeout so the poolminer engine loop can update hashrate while
    // waiting for the next stratum notify (a multi-minute block would otherwise
    // stall recv and freeze the hashrate display at zero).
#ifdef WIN32
    const DWORD recv_timeout_ms = 1000;
    setsockopt(h, SOL_SOCKET, SO_RCVTIMEO, reinterpret_cast<const char*>(&recv_timeout_ms), sizeof(recv_timeout_ms));
#else
    struct timeval recv_timeout;
    recv_timeout.tv_sec = 1;
    recv_timeout.tv_usec = 0;
    setsockopt(h, SOL_SOCKET, SO_RCVTIMEO, &recv_timeout, sizeof(recv_timeout));
#endif
    m_socket = h;
    return true;
}

bool StratumClient::SendLine(const std::string& line, std::string& error_out)
{
    if (m_socket == INVALID_SOCKET) {
        error_out = "not connected";
        return false;
    }
    const std::string payload = line + "\n";
    const char* data = payload.c_str();
    size_t left = payload.size();
    while (left > 0) {
        const int sent = send(m_socket, data, (int)left, 0);
        if (sent <= 0) {
            error_out = "stratum write failed";
            return false;
        }
        data += sent;
        left -= (size_t)sent;
    }
    return true;
}

bool StratumClient::SendRequest(const std::string& method, const UniValue& params, PendingKind kind, std::string& error_out)
{
    const uint64_t id = m_next_id++;
    m_pending[id] = kind;
    UniValue msg(UniValue::VOBJ);
    msg.pushKV("id", (int)id);
    msg.pushKV("method", method);
    msg.pushKV("params", params);
    return SendLine(msg.write(), error_out);
}

bool StratumClient::Subscribe(std::string& error_out)
{
    UniValue params(UniValue::VARR);
    params.push_back("veriumd/1.0");
    return SendRequest("mining.subscribe", params, PendingKind::Subscribe, error_out);
}

bool StratumClient::Authorize(const std::string& username, const std::string& password, std::string& error_out)
{
    UniValue params(UniValue::VARR);
    params.push_back(username);
    params.push_back(password);
    return SendRequest("mining.authorize", params, PendingKind::Authorize, error_out);
}

bool StratumClient::SubmitShare(
    const std::string& username,
    const std::string& job_id,
    const std::string& extranonce2_hex,
    const std::string& ntime_hex,
    const std::string& nonce_hex,
    std::string& error_out)
{
    UniValue params(UniValue::VARR);
    params.push_back(username);
    params.push_back(job_id);
    params.push_back(extranonce2_hex);
    params.push_back(ntime_hex);
    params.push_back(nonce_hex);
    return SendRequest("mining.submit", params, PendingKind::Submit, error_out);
}

bool StratumClient::TryParseBufferedLine(StratumEvent& event_out, std::string& error_out)
{
    const size_t nl = m_read_buf.find('\n');
    if (nl == std::string::npos) {
        return false;
    }

    std::string line = m_read_buf.substr(0, nl);
    m_read_buf.erase(0, nl + 1);
    while (!line.empty() && (line.back() == '\r' || line.back() == '\n' || line.back() == ' ')) {
        line.pop_back();
    }
    if (line.empty()) {
        return true;
    }

    UniValue msg;
    if (!msg.read(line)) {
        LogPrintf("stratum: skipping invalid json line\n");
        event_out = StratumEvent{};
        error_out.clear();
        return true;
    }
    event_out = ParseMessage(msg);
    return true;
}

bool StratumClient::ReadEvent(StratumEvent& event_out, std::string& error_out)
{
    event_out = StratumEvent{};
    if (m_socket == INVALID_SOCKET) {
        error_out = "not connected";
        return false;
    }

    if (TryParseBufferedLine(event_out, error_out)) {
        return true;
    }
    if (!error_out.empty()) {
        return false;
    }

    char buf[4096];
    const int n = recv(m_socket, buf, sizeof(buf), 0);
    if (n == 0) {
        event_out.type = StratumEventType::Disconnected;
        event_out.disconnect_reason = "eof";
        return true;
    }
    if (n < 0) {
#ifdef WIN32
        const int err = WSAGetLastError();
        if (err == WSAEWOULDBLOCK || err == WSAETIMEDOUT || err == WSAEINTR) {
            return true;
        }
        event_out.type = StratumEventType::Disconnected;
        event_out.disconnect_reason = strprintf("recv error (%s)", NetworkErrorString(err));
#else
        if (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR) {
            return true;
        }
        event_out.type = StratumEventType::Disconnected;
        event_out.disconnect_reason = strprintf("recv error (%s)", NetworkErrorString(errno));
#endif
        return true;
    }

    m_read_buf.append(buf, buf + n);
    if (!TryParseBufferedLine(event_out, error_out)) {
        return true;
    }
    return true;
}

StratumEvent StratumClient::ParseMessage(const UniValue& msg)
{
    StratumEvent ev;
    const std::string method = msg.exists("method") ? msg["method"].get_str() : "";
    if (!method.empty()) {
        if (method == "mining.set_difficulty") {
            ev.type = StratumEventType::SetDifficulty;
            const UniValue& params = msg["params"];
            if (params.isArray() && params.size() > 0) {
                ev.wire_difficulty = params[0].isNum() ? params[0].get_real() : 1.0;
            } else {
                ev.wire_difficulty = 1.0;
            }
            return ev;
        }
        if (method == "mining.notify") {
            ev.type = StratumEventType::Notify;
            const UniValue& p = msg["params"];
            if (!p.isArray() || p.size() < 9) {
                ev.type = StratumEventType::Disconnected;
                ev.disconnect_reason = "bad notify";
                return ev;
            }
            const std::string prev_internal = p[1].get_str();
            const std::vector<unsigned char> prev_bytes = ParseHex(prev_internal);
            const std::vector<unsigned char> prev_rev = ReverseBuffer(prev_bytes);
            StratumJob job;
            job.job_id = p[0].get_str();
            job.prev_hash_hex = HexStr(prev_rev);
            job.coinb1_hex = p[2].get_str();
            job.coinb2_hex = p[3].get_str();
            if (p[4].isArray()) {
                for (unsigned i = 0; i < p[4].size(); ++i) {
                    job.merkle_steps_hex.push_back(p[4][i].get_str());
                }
            }
            job.version = (uint32_t)strtoul(p[5].get_str().c_str(), nullptr, 16);
            job.bits_hex = p[6].get_str();
            job.ntime_hex = p[7].get_str();
            const std::vector<unsigned char> ntime_bytes = ParseHex(job.ntime_hex);
            if (ntime_bytes.size() >= 4) {
                memcpy(&job.ntime, ntime_bytes.data(), 4);
            }
            ev.notify.job = std::move(job);
            ev.notify.clean_jobs = p[8].isBool() ? p[8].get_bool() : false;
            return ev;
        }
        LogPrintf("stratum: ignoring unknown method %s\n", method);
        return ev;
    }

    if (!msg.exists("id")) {
        LogPrintf("stratum: ignoring message without id\n");
        return ev;
    }

    const uint64_t id = msg["id"].isNum() ? (uint64_t)msg["id"].get_int64() : 0;
    auto it = m_pending.find(id);
    if (it == m_pending.end()) {
        LogPrintf("stratum: ignoring unexpected response id %u\n", id);
        return ev;
    }
    const PendingKind kind = it->second;
    m_pending.erase(it);

    switch (kind) {
    case PendingKind::Subscribe: {
        ev.type = StratumEventType::Subscribed;
        const UniValue& result = msg["result"];
        if (result.isArray() && result.size() >= 3 && result[0].isArray()) {
            ev.extranonce1_hex = result[1].get_str();
            ev.extranonce2_size = result[2].isNum() ? (size_t)result[2].get_int() : 4;
        } else {
            ev.type = StratumEventType::Disconnected;
            ev.disconnect_reason = "bad subscribe response";
        }
        return ev;
    }
    case PendingKind::Authorize: {
        const bool ok = msg.exists("result") && msg["result"].isBool() && msg["result"].get_bool();
        if (ok) {
            ev.type = StratumEventType::Authorized;
        } else {
            ev.type = StratumEventType::Disconnected;
            ev.disconnect_reason = "authorize rejected";
        }
        return ev;
    }
    case PendingKind::Submit: {
        ev.type = StratumEventType::SubmitResult;
        ev.submit_accepted = msg.exists("result") && msg["result"].isBool() && msg["result"].get_bool();
        if (msg.exists("error") && !msg["error"].isNull()) {
            ev.submit_error = msg["error"].write();
        }
        return ev;
    }
    }
    ev.type = StratumEventType::Disconnected;
    ev.disconnect_reason = "internal parse error";
    return ev;
}

} // namespace stratum
