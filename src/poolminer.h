// Copyright (c) 2026 The Vericonomy developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#ifndef VERIUM_POOLMINER_H
#define VERIUM_POOLMINER_H

#include <string>

#include <univalue.h>

/** Runtime status for in-process Stratum pool mining in veriumd. */
struct PoolMinerRuntimeStatus
{
    bool running{false};
    double hashrate_hm{0.0};
    std::string worker;
    std::string backend{"native"};
    std::string last_log_line;
    int threads{0};
};

UniValue PoolMinerStatusToJSON(const PoolMinerRuntimeStatus& st);

PoolMinerRuntimeStatus GetPoolMinerStatus();

/** Empty string on success; otherwise human-readable error. */
std::string PoolMinerStart(int threads, const std::string& stratum_url,
    const std::string& username, const std::string& password);

/** Empty string on success; otherwise human-readable error. */
std::string PoolMinerStop();

/** Called from node shutdown — stops workers and Stratum thread. */
void StopPoolMinerOnShutdown();

#endif // VERIUM_POOLMINER_H
