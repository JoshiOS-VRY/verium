// Copyright (c) 2026 The Vericonomy developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#ifndef VERIUM_STRATUM_PROTOCOL_H
#define VERIUM_STRATUM_PROTOCOL_H

#include <cstdint>
#include <string>
#include <vector>

namespace stratum {

constexpr double SCRYPT2_STRATUM_FACTOR = 65536.0;

struct StratumJob {
    std::string job_id;
    std::string prev_hash_hex;
    std::string coinb1_hex;
    std::string coinb2_hex;
    std::vector<std::string> merkle_steps_hex;
    uint32_t version{0};
    std::string bits_hex;
    uint32_t ntime{0};
    std::string ntime_hex;
};

struct WorkerHeaderWork {
    std::string job_id;
    std::string ntime_hex;
    std::string extranonce2_hex;
    uint8_t header_base[80]{};
};

double StratumToEffectiveDifficulty(double wire_difficulty);
void DifficultyToTarget(double difficulty, uint8_t target[32]);
/** uint32_t[8] target for scanhash/fulltest (matches veriumMiner diff_to_target). */
void DifficultyToTargetWords(double difficulty, uint32_t target[8]);
bool HashMeetsTarget(const uint8_t hash[32], const uint8_t target[32]);

bool ParseStratumUrl(const std::string& url, std::string& host_out, uint16_t& port_out);

bool PrepareWorkerHeaderWork(
    const StratumJob& job,
    const std::vector<uint8_t>& extranonce1,
    const std::vector<uint8_t>& extranonce2,
    WorkerHeaderWork& work_out,
    std::string& error_out);

void HeaderWithNonce(const WorkerHeaderWork& work, uint32_t nonce, uint8_t header[80]);
std::string Uint32LeHex(uint32_t n);

} // namespace stratum

#endif // VERIUM_STRATUM_PROTOCOL_H
