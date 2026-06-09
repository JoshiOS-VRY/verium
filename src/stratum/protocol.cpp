// Copyright (c) 2026 The Vericonomy developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#include <stratum/protocol.h>

#include <arith_uint256.h>
#include <hash.h>
#include <util/strencodings.h>

#include <algorithm>
#include <cstring>

namespace stratum {
namespace {

constexpr uint64_t DIFF_PRECISION = 100000000ULL;

static std::string TrimCopy(const std::string& s)
{
    const size_t start = s.find_first_not_of(" \t\r\n");
    if (start == std::string::npos) {
        return {};
    }
    const size_t end = s.find_last_not_of(" \t\r\n");
    return s.substr(start, end - start + 1);
}

void Sha256d(const uint8_t* data, size_t len, uint8_t out[32])
{
    CHash256 hasher;
    hasher.Write(data, len).Finalize(out);
}

void ReverseTo32(const std::vector<unsigned char>& buf, uint8_t out[32])
{
    memset(out, 0, 32);
    const size_t len = std::min<size_t>(buf.size(), 32);
    for (size_t i = 0; i < len; ++i) {
        out[i] = buf[buf.size() - 1 - i];
    }
}

bool DecodePrevInternal(const std::string& prev_hash_hex, uint8_t out[32])
{
    const std::vector<unsigned char> prev = ParseHex(prev_hash_hex);
    if (prev.empty()) {
        return false;
    }
    ReverseTo32(prev, out);
    return true;
}

void SerializeHeaderParts(
    uint32_t version,
    const uint8_t prev_internal[32],
    const uint8_t merkle_root[32],
    uint32_t ntime,
    const std::string& bits_hex,
    uint32_t nonce,
    uint8_t header[80])
{
    uint32_t bits = 0;
    if (!bits_hex.empty()) {
        bits = (uint32_t)strtoul(bits_hex.c_str(), nullptr, 16);
    }
    memcpy(header + 0, &version, 4);
    memcpy(header + 4, prev_internal, 32);
    memcpy(header + 36, merkle_root, 32);
    memcpy(header + 68, &ntime, 4);
    memcpy(header + 72, &bits, 4);
    memcpy(header + 76, &nonce, 4);
}

uint8_t MerkleJoin(const uint8_t a[32], const uint8_t b[32], uint8_t out[32])
{
    uint8_t concat[64];
    memcpy(concat, a, 32);
    memcpy(concat + 32, b, 32);
    Sha256d(concat, 64, out);
    return 0;
}

void MerkleRootWithCoinbase(const uint8_t coinbase_hash[32], const std::vector<std::vector<unsigned char>>& steps, uint8_t out[32])
{
    memcpy(out, coinbase_hash, 32);
    for (const auto& step : steps) {
        uint8_t step32[32]{};
        const size_t len = std::min<size_t>(step.size(), 32);
        memcpy(step32, step.data(), len);
        MerkleJoin(out, step32, out);
    }
}

std::vector<uint8_t> AssembleCoinbase(
    const std::vector<unsigned char>& coinb1,
    const std::vector<uint8_t>& extranonce1,
    const std::vector<uint8_t>& extranonce2,
    const std::vector<unsigned char>& coinb2)
{
    std::vector<uint8_t> out;
    out.reserve(coinb1.size() + extranonce1.size() + extranonce2.size() + coinb2.size());
    out.insert(out.end(), coinb1.begin(), coinb1.end());
    out.insert(out.end(), extranonce1.begin(), extranonce1.end());
    out.insert(out.end(), extranonce2.begin(), extranonce2.end());
    out.insert(out.end(), coinb2.begin(), coinb2.end());
    return out;
}

void ArithToTargetBE(const arith_uint256& bn, uint8_t target[32])
{
    std::string hex = bn.GetHex();
    while (hex.size() < 64) {
        hex = "0" + hex;
    }
    const std::vector<unsigned char> bytes = ParseHex(hex);
    memset(target, 0, 32);
    const size_t len = std::min<size_t>(bytes.size(), 32);
    memcpy(target + (32 - len), bytes.data() + (bytes.size() - len), len);
}

} // namespace

double StratumToEffectiveDifficulty(double wire_difficulty)
{
    return wire_difficulty / SCRYPT2_STRATUM_FACTOR;
}

void DifficultyToTargetWords(double difficulty, uint32_t target[8])
{
    if (difficulty <= 0.0) {
        memset(target, 0xff, 32);
        return;
    }
    int k = 6;
    double diff = difficulty;
    while (k > 0 && diff > 1.0) {
        diff /= 4294967296.0;
        --k;
    }
    const uint64_t m = (uint64_t)(4294901760.0 / diff);
    if (m == 0 && k == 6) {
        memset(target, 0xff, 32);
        return;
    }
    memset(target, 0, 32);
    target[k] = (uint32_t)m;
    target[k + 1] = (uint32_t)(m >> 32);
}

void DifficultyToTarget(double difficulty, uint8_t target[32])
{
    if (difficulty <= 0.0) {
        memset(target, 0xff, 32);
        return;
    }
    const uint64_t scaled = (uint64_t)(difficulty * (double)DIFF_PRECISION + 0.5);
    if (scaled == 0) {
        memset(target, 0xff, 32);
        return;
    }
    arith_uint256 diff1;
    diff1.SetHex("00000000ffff0000000000000000000000000000000000000000000000000000");
    arith_uint256 bnTarget = diff1 * arith_uint256(DIFF_PRECISION) / arith_uint256(scaled);
    arith_uint256 maxUint;
    maxUint.SetHex("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    if (bnTarget > maxUint) {
        bnTarget = maxUint;
    }
    ArithToTargetBE(bnTarget, target);
}

bool HashMeetsTarget(const uint8_t hash[32], const uint8_t target[32])
{
    uint8_t rev[32];
    for (int i = 0; i < 32; ++i) {
        rev[i] = hash[31 - i];
    }
    return memcmp(rev, target, 32) <= 0;
}

bool ParseStratumUrl(const std::string& url, std::string& host_out, uint16_t& port_out)
{
    std::string trimmed = TrimCopy(url);
    const std::string prefixes[] = {"stratum+tcp://", "stratum://"};
    for (const auto& prefix : prefixes) {
        if (trimmed.rfind(prefix, 0) == 0) {
            trimmed = trimmed.substr(prefix.size());
            break;
        }
    }
    const size_t colon = trimmed.rfind(':');
    if (colon == std::string::npos || colon == 0 || colon + 1 >= trimmed.size()) {
        return false;
    }
    host_out = trimmed.substr(0, colon);
    try {
        const int p = std::stoi(trimmed.substr(colon + 1));
        if (p <= 0 || p > 65535) {
            return false;
        }
        port_out = (uint16_t)p;
    } catch (...) {
        return false;
    }
    return !host_out.empty();
}

bool PrepareWorkerHeaderWork(
    const StratumJob& job,
    const std::vector<uint8_t>& extranonce1,
    const std::vector<uint8_t>& extranonce2,
    WorkerHeaderWork& work_out,
    std::string& error_out)
{
    const std::vector<unsigned char> coinb1 = ParseHex(job.coinb1_hex);
    const std::vector<unsigned char> coinb2 = ParseHex(job.coinb2_hex);
    if (coinb1.empty() && !job.coinb1_hex.empty()) {
        error_out = "invalid coinb1";
        return false;
    }
    if (coinb2.empty() && !job.coinb2_hex.empty()) {
        error_out = "invalid coinb2";
        return false;
    }

    std::vector<uint8_t> coinbase = AssembleCoinbase(coinb1, extranonce1, extranonce2, coinb2);
    if (coinbase.size() >= 8) {
        memcpy(coinbase.data() + 4, &job.ntime, 4);
    }
    uint8_t coinbase_hash[32];
    Sha256d(coinbase.data(), coinbase.size(), coinbase_hash);

    std::vector<std::vector<unsigned char>> steps;
    steps.reserve(job.merkle_steps_hex.size());
    for (const auto& h : job.merkle_steps_hex) {
        steps.push_back(ParseHex(h));
    }
    uint8_t merkle_root[32];
    MerkleRootWithCoinbase(coinbase_hash, steps, merkle_root);

    uint8_t prev_internal[32];
    if (!DecodePrevInternal(job.prev_hash_hex, prev_internal)) {
        error_out = "invalid prev_hash";
        return false;
    }

    work_out.job_id = job.job_id;
    work_out.ntime_hex = job.ntime_hex;
    work_out.extranonce2_hex = HexStr(extranonce2);
    SerializeHeaderParts(job.version, prev_internal, merkle_root, job.ntime, job.bits_hex, 0, work_out.header_base);
    return true;
}

void HeaderWithNonce(const WorkerHeaderWork& work, uint32_t nonce, uint8_t header[80])
{
    memcpy(header, work.header_base, 80);
    memcpy(header + 76, &nonce, 4);
}

std::string Uint32LeHex(uint32_t n)
{
    uint8_t b[4];
    memcpy(b, &n, 4);
    return HexStr(b, b + 4);
}

} // namespace stratum
