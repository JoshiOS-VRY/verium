// Copyright (c) 2026 The Vericonomy developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#include <poolminer.h>

#include <crypto/scrypt_dispatch.h>
#include <stratum/client.h>
#include <stratum/protocol.h>
#include <sync.h>
#include <util/strencodings.h>
#include <util/system.h>
#include <util/threadnames.h>
#include <util/time.h>

#include <atomic>
#include <chrono>
#include <cstring>
#include <memory>
#include <thread>
#include <vector>

namespace {

constexpr uint32_t JOB_REFRESH_INTERVAL = 32;
constexpr size_t MAX_SHARE_QUEUE = 256;

struct ShareFound {
    std::string job_id;
    std::string extranonce2_hex;
    std::string ntime_hex;
    std::string nonce_hex;
};

struct JobState {
    std::shared_ptr<stratum::StratumJob> job;
    std::vector<uint8_t> extranonce1;
    size_t extranonce2_size{4};
    double wire_difficulty{0.0};
    double effective_difficulty{0.0};
    uint8_t target[32]{};
    uint64_t generation{0};
};

struct EngineConfig {
    std::string host;
    uint16_t port{0};
    std::string username;
    std::string password;
    int threads{1};
};

CCriticalSection g_pool_mutex;
std::atomic<bool> g_shutdown{false};
std::atomic<bool> g_cancel{false};
std::thread g_engine_thread;
std::vector<std::thread> g_worker_threads;

bool g_running = false;
double g_hashrate_hm = 0.0;
std::string g_worker;
std::string g_backend = "native";
std::string g_last_log;
int g_threads = 0;

CCriticalSection g_job_mutex;
JobState g_job_state;

CCriticalSection g_share_mutex;
std::vector<ShareFound> g_share_queue;

std::atomic<uint64_t> g_hash_counter{0};

void SetLastLog(const std::string& line)
{
    LOCK(g_pool_mutex);
    g_last_log = line;
}

void WorkerLoop(int thread_id, int thread_count)
{
    util::ThreadRename(strprintf("poolminer-%d", thread_id));
    // ScryptDispatchHash uses a per-thread scratch buffer internally (~128 MiB).

    const std::string en2_hex = (thread_id == 0 && thread_count == 1)
        ? "00000000"
        : strprintf("%08x", thread_id);
    std::vector<uint8_t> en2 = ParseHex(en2_hex);

    uint64_t local_gen = 0;
    uint32_t nonce = (uint32_t)thread_id;
    stratum::WorkerHeaderWork header_work;
    bool have_work = false;
    uint8_t target[32]{};
    uint32_t hashes_since_refresh = 0;

    while (!g_cancel.load() && !g_shutdown.load()) {
        if (!have_work || hashes_since_refresh >= JOB_REFRESH_INTERVAL) {
            hashes_since_refresh = 0;
            JobState snapshot;
            {
                LOCK(g_job_mutex);
                snapshot = g_job_state;
            }

            if (!snapshot.job || snapshot.extranonce1.empty() || snapshot.effective_difficulty <= 0.0) {
                MilliSleep(200);
                continue;
            }

            memcpy(target, snapshot.target, 32);
            if (snapshot.generation != local_gen || !have_work) {
                local_gen = snapshot.generation;
                nonce = (uint32_t)thread_id;
                std::vector<uint8_t> en2_use = en2;
                if (snapshot.extranonce2_size != 4) {
                    en2_use.assign(snapshot.extranonce2_size, 0);
                }
                std::string err;
                if (!stratum::PrepareWorkerHeaderWork(*snapshot.job, snapshot.extranonce1, en2_use, header_work, err)) {
                    LogPrintf("poolminer: thread %d prepare work failed: %s\n", thread_id, err);
                    MilliSleep(500);
                    continue;
                }
                have_work = true;
            }
        }

        if (!have_work) {
            MilliSleep(200);
            continue;
        }

        uint8_t header[80];
        stratum::HeaderWithNonce(header_work, nonce, header);

        char hashbuf[32];
        ScryptDispatchHash(header, hashbuf);

        g_hash_counter.fetch_add(1, std::memory_order_relaxed);
        hashes_since_refresh++;

        if (stratum::HashMeetsTarget(reinterpret_cast<const uint8_t*>(hashbuf), target)) {
            ShareFound found;
            found.job_id = header_work.job_id;
            found.extranonce2_hex = header_work.extranonce2_hex;
            found.ntime_hex = header_work.ntime_hex;
            found.nonce_hex = stratum::Uint32LeHex(nonce);
            LOCK(g_share_mutex);
            if (g_share_queue.size() < MAX_SHARE_QUEUE) {
                g_share_queue.push_back(std::move(found));
            }
            LogPrintf("poolminer: thread %d found share nonce=%u\n", thread_id, nonce);
        }

        nonce += (uint32_t)thread_count;
        if (nonce % 4096 == (uint32_t)thread_id) {
            std::this_thread::yield();
        }
    }
}

void JoinWorkerThreads()
{
    g_cancel.store(true);
    for (auto& t : g_worker_threads) {
        if (t.joinable()) {
            t.join();
        }
    }
    g_worker_threads.clear();
}

void MarkPoolMinerStopped()
{
    LOCK(g_pool_mutex);
    g_running = false;
    g_hashrate_hm = 0.0;
}

void RunEngine(EngineConfig cfg)
{
    util::ThreadRename("poolminer-engine");

    g_cancel.store(false);
    g_hash_counter.store(0);
    {
        LOCK(g_job_mutex);
        g_job_state = JobState{};
    }
    {
        LOCK(g_share_mutex);
        g_share_queue.clear();
    }

    stratum::StratumClient client;
    std::string err;
    if (!client.Connect(cfg.host, cfg.port, err)) {
        SetLastLog(strprintf("Pool connect failed: %s", err));
        MarkPoolMinerStopped();
        return;
    }
    if (!client.Subscribe(err)) {
        SetLastLog(strprintf("Subscribe failed: %s", err));
        MarkPoolMinerStopped();
        return;
    }
    if (!client.Authorize(cfg.username, cfg.password, err)) {
        SetLastLog(strprintf("Authorize failed: %s", err));
        MarkPoolMinerStopped();
        return;
    }

    SetLastLog("Subscribed; waiting for jobs…");

    g_worker_threads.clear();
    g_worker_threads.reserve((size_t)cfg.threads);
    for (int i = 0; i < cfg.threads; ++i) {
        g_worker_threads.emplace_back(WorkerLoop, i, cfg.threads);
    }

    int64_t hashrate_window_start = GetTimeMillis();
    uint64_t last_hash_total = 0;

    while (!g_cancel.load() && !g_shutdown.load()) {
        {
            std::vector<ShareFound> pending;
            {
                LOCK(g_share_mutex);
                pending.swap(g_share_queue);
            }
            for (const auto& share : pending) {
                if (!client.SubmitShare(cfg.username, share.job_id, share.extranonce2_hex, share.ntime_hex, share.nonce_hex, err)) {
                    LogPrintf("poolminer: submit failed: %s\n", err);
                } else {
                    SetLastLog(strprintf("Share submitted (job %s)", share.job_id));
                }
            }
        }

        bool handled_stratum = false;
        for (;;) {
            stratum::StratumEvent event;
            if (!client.ReadEvent(event, err)) {
                SetLastLog(strprintf("Stratum read error: %s", err));
                g_cancel.store(true);
                break;
            }
            if (event.type == stratum::StratumEventType::None) {
                break;
            }
            handled_stratum = true;

            switch (event.type) {
        case stratum::StratumEventType::Subscribed: {
            const std::vector<unsigned char> en1 = ParseHex(event.extranonce1_hex);
            LOCK(g_job_mutex);
            g_job_state.extranonce1.assign(en1.begin(), en1.end());
            g_job_state.extranonce2_size = event.extranonce2_size;
            SetLastLog(strprintf("Subscribed (extranonce1=%s)", event.extranonce1_hex));
            break;
        }
        case stratum::StratumEventType::Authorized:
            SetLastLog("Authorized on pool");
            break;
        case stratum::StratumEventType::SetDifficulty: {
            const double effective = stratum::StratumToEffectiveDifficulty(event.wire_difficulty);
            uint8_t target[32];
            stratum::DifficultyToTarget(effective, target);
            LOCK(g_job_mutex);
            g_job_state.wire_difficulty = event.wire_difficulty;
            g_job_state.effective_difficulty = effective;
            memcpy(g_job_state.target, target, 32);
            g_job_state.generation++;
            SetLastLog(strprintf("Difficulty %.6f (wire %.2f)", effective, event.wire_difficulty));
            break;
        }
        case stratum::StratumEventType::Notify: {
            LOCK(g_job_mutex);
            if (event.notify.clean_jobs) {
                g_job_state.generation++;
            }
            g_job_state.job = std::make_shared<stratum::StratumJob>(event.notify.job);
            g_job_state.generation++;
            SetLastLog("New pool job received");
            break;
        }
        case stratum::StratumEventType::SubmitResult:
            SetLastLog(event.submit_accepted
                ? "Share accepted by pool"
                : strprintf("Share rejected: %s", event.submit_error));
            break;
            case stratum::StratumEventType::Disconnected:
                SetLastLog(strprintf("Stratum disconnected: %s", event.disconnect_reason));
                g_cancel.store(true);
                break;
            default:
                break;
            }
            if (g_cancel.load()) {
                break;
            }
        }

        if (!handled_stratum) {
            MilliSleep(10);
        }

        const int64_t now = GetTimeMillis();
        if (now - hashrate_window_start >= 5000) {
            const uint64_t total = g_hash_counter.load(std::memory_order_relaxed);
            const uint64_t delta = total - last_hash_total;
            const double sec = (now - hashrate_window_start) / 1000.0;
            if (sec > 0.0) {
                LOCK(g_pool_mutex);
                g_hashrate_hm = (double)delta * 60.0 / sec;
            }
            last_hash_total = total;
            hashrate_window_start = now;
        }
    }

    JoinWorkerThreads();
    MarkPoolMinerStopped();
}

} // namespace

UniValue PoolMinerStatusToJSON(const PoolMinerRuntimeStatus& st)
{
    UniValue obj(UniValue::VOBJ);
    obj.pushKV("running", st.running);
    obj.pushKV("hashrate_hm", st.hashrate_hm);
    obj.pushKV("worker", st.worker);
    obj.pushKV("backend", st.backend);
    obj.pushKV("last_log_line", st.last_log_line);
    obj.pushKV("threads", st.threads);
    return obj;
}

PoolMinerRuntimeStatus GetPoolMinerStatus()
{
    PoolMinerRuntimeStatus st;
    LOCK(g_pool_mutex);
    st.running = g_running;
    st.hashrate_hm = g_hashrate_hm;
    st.worker = g_worker;
    st.backend = g_backend;
    st.last_log_line = g_last_log;
    st.threads = g_threads;
    return st;
}

std::string PoolMinerStart(int threads, const std::string& stratum_url,
    const std::string& username, const std::string& password)
{
    if (threads < 1) {
        return "Thread count must be at least 1";
    }
    if (stratum_url.empty() || username.empty()) {
        return "Stratum URL and worker username are required";
    }
    if (!ScryptDispatchInit() || ScryptDispatchBestThroughput() <= 0) {
        return "Scrypt mining is unavailable in this veriumd build (consensus self-test failed)";
    }

    std::string host;
    uint16_t port = 0;
    if (!stratum::ParseStratumUrl(stratum_url, host, port)) {
        return strprintf("Invalid stratum URL: %s", stratum_url);
    }

    PoolMinerStop();

    EngineConfig cfg;
    cfg.host = host;
    cfg.port = port;
    cfg.username = username;
    cfg.password = password.empty() ? "x" : password;
    cfg.threads = threads;

    g_cancel.store(false);
    {
        LOCK(g_pool_mutex);
        g_running = true;
        g_hashrate_hm = 0.0;
        g_worker = username;
        g_backend = "native";
        g_threads = threads;
        g_last_log = strprintf("Connecting to %s:%u (%d threads)…", host, port, threads);
    }

    if (g_engine_thread.joinable()) {
        g_engine_thread.join();
    }
    g_engine_thread = std::thread([cfg]() { RunEngine(cfg); });

    return {};
}

std::string PoolMinerStop()
{
    g_cancel.store(true);
    if (g_engine_thread.joinable()) {
        g_engine_thread.join();
    }
    LOCK(g_pool_mutex);
    g_running = false;
    g_hashrate_hm = 0.0;
    g_last_log = "Pool miner stopped";
    return {};
}

void StopPoolMinerOnShutdown()
{
    g_shutdown.store(true);
    PoolMinerStop();
}
