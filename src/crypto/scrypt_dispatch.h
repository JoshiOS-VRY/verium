// Copyright (c) 2026 The Verium Core developers
// Distributed under the MIT software license.

#ifndef BITCOIN_CRYPTO_SCRYPT_DISPATCH_H
#define BITCOIN_CRYPTO_SCRYPT_DISPATCH_H

#include <atomic>
#include <stdint.h>
#include <uint256.h>

/** Active SIMD / ISA tier selected at runtime after KAT self-test. */
enum class ScryptDispatchTier {
    REFERENCE = 0,
    AVX2 = 1,
    AVX512 = 2,
    ARM_CRYPTO = 3,
    ARM_NEON = 4,
};

/** Detect CPU, validate all compiled tiers against reference KAT, pick fastest passing tier. */
bool ScryptDispatchInit();

ScryptDispatchTier ScryptDispatchTierActive();
const char* ScryptDispatchTierName(ScryptDispatchTier tier);
const char* ScryptDispatchTierNameActive();

/** Throughput (parallel nonces) for the active tier. */
int ScryptDispatchBestThroughput();
int ScryptDispatchActiveThroughput();

/** Mining hot path — dispatches to active tier. */
bool ScryptDispatch_N_1_1_256_multi(void* input, uint256 hashTarget, int* nHashesDone, unsigned char* scratchbuf);

/** PoW hash (scrypt^2) — dispatches to active tier. */
void ScryptDispatchHash(const void* input, char* output);

/**
 * Pool mining hot path — SIMD multi-lane batch with Stratum target check.
 * *logical_nonce is the worker's Stratum nonce (in/out). On success,
 * *winning_logical_nonce is the share nonce for submission.
 */
bool ScryptDispatchPoolBatch(
    const void* header_template,
    const uint8_t pool_target[32],
    uint32_t* logical_nonce,
    uint32_t nonce_stride,
    int* nHashesDone,
    unsigned char* scratchbuf,
    uint32_t* winning_logical_nonce);

void ScryptDispatchPoolPrepareWork(const void* header80, uint32_t pdata[20], uint32_t midstate[8]);

/** Burst hash up to burst_hashes nonces with cached pdata/midstate (pool hot path). */
bool ScryptDispatchPoolBurst(
    uint32_t pdata[20],
    uint32_t midstate[8],
    const uint32_t pool_target[8],
    uint32_t* logical_nonce,
    uint32_t nonce_stride,
    int burst_hashes,
    int* nHashesDone,
    unsigned char* scratchbuf,
    uint32_t* winning_logical_nonce,
    const std::atomic<bool>* stop_flag = nullptr);

/** Solo mining scratch — full multi-lane buffer (~128 MiB × SCRYPT_MAX_WAYS). */
unsigned char* ScryptDispatchBufferAlloc();

void ScryptDispatchBufferFree(unsigned char* buf);

/** Pool mining scratch — single-lane (~128 MiB) when throughput is 1, else full buffer. */
unsigned char* ScryptDispatchPoolScratchAlloc();

void ScryptDispatchPoolScratchFree(unsigned char* buf);

#endif // BITCOIN_CRYPTO_SCRYPT_DISPATCH_H
