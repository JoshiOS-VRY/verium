// Copyright (c) 2026 The Vericonomy developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#ifndef VERIUM_STRATUM_CLIENT_H
#define VERIUM_STRATUM_CLIENT_H

#include <stratum/protocol.h>

#include <compat.h>
#include <univalue.h>

#include <cstdint>
#include <map>
#include <string>

namespace stratum {

struct StratumNotify {
    StratumJob job;
    bool clean_jobs{false};
};

enum class StratumEventType {
    None,
    Subscribed,
    Authorized,
    SetDifficulty,
    Notify,
    SubmitResult,
    Disconnected,
};

struct StratumEvent {
    StratumEventType type{StratumEventType::None};
    std::string extranonce1_hex;
    size_t extranonce2_size{4};
    double wire_difficulty{0.0};
    StratumNotify notify;
    bool submit_accepted{false};
    std::string submit_error;
    std::string disconnect_reason;
};

class StratumClient {
public:
    StratumClient();
    ~StratumClient();

    StratumClient(const StratumClient&) = delete;
    StratumClient& operator=(const StratumClient&) = delete;

    bool Connect(const std::string& host, uint16_t port, std::string& error_out);
    void Disconnect();

    bool Subscribe(std::string& error_out);
    bool Authorize(const std::string& username, const std::string& password, std::string& error_out);
    bool SubmitShare(
        const std::string& username,
        const std::string& job_id,
        const std::string& extranonce2_hex,
        const std::string& ntime_hex,
        const std::string& nonce_hex,
        std::string& error_out);

    /** Returns false on hard I/O error; `event.type == None` when no full line yet. */
    bool ReadEvent(StratumEvent& event_out, std::string& error_out);

private:
    enum class PendingKind { Subscribe, Authorize, Submit };

    bool SendLine(const std::string& line, std::string& error_out);
    bool SendRequest(const std::string& method, const UniValue& params, PendingKind kind, std::string& error_out);
    bool TryParseBufferedLine(StratumEvent& event_out, std::string& error_out);
    StratumEvent ParseMessage(const UniValue& msg);

    SOCKET m_socket{INVALID_SOCKET};
    std::string m_read_buf;
    uint64_t m_next_id{1};
    std::map<uint64_t, PendingKind> m_pending;
};

} // namespace stratum

#endif // VERIUM_STRATUM_CLIENT_H
