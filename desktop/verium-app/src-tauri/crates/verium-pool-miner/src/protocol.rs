//! Share reconstruction and PoW target math (port of `@pool/protocol`).

use num_bigint::BigUint;
use sha2::{Digest, Sha256};

pub const SCRYPT2_STRATUM_FACTOR: f64 = 65536.0;

const DIFF1_HEX: &str = "00000000ffff0000000000000000000000000000000000000000000000000000";
const DIFF_PRECISION: u64 = 100_000_000;

/// Double SHA-256.
pub fn sha256d(buf: &[u8]) -> [u8; 32] {
    let a = Sha256::digest(buf);
    Sha256::digest(a).into()
}

pub fn reverse_buffer(buf: &[u8]) -> Vec<u8> {
    buf.iter().rev().copied().collect()
}

fn reverse_to_32(buf: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let len = buf.len().min(32);
    for i in 0..len {
        out[i] = buf[buf.len() - 1 - i];
    }
    out
}

fn decode_prev_internal(prev_hash_hex: &str) -> Result<[u8; 32], String> {
    let prev = hex::decode(prev_hash_hex).map_err(|e| format!("prev_hash hex: {e}"))?;
    Ok(reverse_to_32(&prev))
}

pub fn stratum_to_effective_difficulty(stratum_diff: f64) -> f64 {
    stratum_diff / SCRYPT2_STRATUM_FACTOR
}

fn diff1_target() -> BigUint {
    BigUint::parse_bytes(DIFF1_HEX.as_bytes(), 16).expect("DIFF1")
}

fn max_u256() -> BigUint {
    (BigUint::from(1u8) << 256) - 1u8
}

fn biguint_to_be32(v: BigUint) -> [u8; 32] {
    let bytes = v.to_bytes_be();
    let mut out = [0u8; 32];
    let start = 32usize.saturating_sub(bytes.len());
    out[start..].copy_from_slice(&bytes);
    out
}

/// Convert fractional difficulty to a 256-bit target.
pub fn difficulty_to_target(difficulty: f64) -> [u8; 32] {
    if difficulty <= 0.0 {
        return biguint_to_be32(max_u256());
    }
    let scaled = (difficulty * DIFF_PRECISION as f64).round() as u64;
    if scaled == 0 {
        return biguint_to_be32(max_u256());
    }
    let target = diff1_target() * BigUint::from(DIFF_PRECISION) / BigUint::from(scaled);
    let capped = if target > max_u256() {
        max_u256()
    } else {
        target
    };
    biguint_to_be32(capped)
}

/// Interpret a 32-byte scrypt² hash as a 256-bit integer (reverse bytes, big-endian read).
pub fn hash_to_biguint(hash: &[u8; 32]) -> BigUint {
    let mut rev = *hash;
    rev.reverse();
    BigUint::from_bytes_be(&rev)
}

pub fn hash_meets_target(hash: &[u8; 32], target: &[u8; 32]) -> bool {
    hash_meets_target_fast(hash, target)
}

/// Allocation-free target check (consensus hash byte order vs big-endian target).
pub fn hash_meets_target_fast(hash: &[u8; 32], target: &[u8; 32]) -> bool {
    let mut rev = *hash;
    rev.reverse();
    rev <= *target
}

pub fn merkle_join(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut concat = [0u8; 64];
    concat[..32].copy_from_slice(a);
    concat[32..].copy_from_slice(b);
    sha256d(&concat)
}

pub fn merkle_root_with_coinbase(coinbase_hash: [u8; 32], steps: &[[u8; 32]]) -> [u8; 32] {
    let mut root = coinbase_hash;
    for step in steps {
        root = merkle_join(&root, step);
    }
    root
}

#[derive(Clone, Debug)]
pub struct StratumJob {
    pub job_id: String,
    /// GBT display-order previous block hash (hex).
    pub prev_hash_hex: String,
    pub coinb1_hex: String,
    pub coinb2_hex: String,
    pub merkle_steps_hex: Vec<String>,
    pub version: u32,
    pub bits_hex: String,
    pub ntime: u32,
    pub ntime_hex: String,
}

#[derive(Clone, Debug)]
pub struct ShareSubmission {
    pub extranonce1: Vec<u8>,
    pub extranonce2_hex: String,
    pub ntime: u32,
    pub nonce: u32,
}

#[derive(Clone, Debug)]
pub struct ReconstructedShare {
    pub consensus_header: [u8; 80],
    pub merkle_root: [u8; 32],
}

/// Per-worker header template: merkle and fixed fields precomputed; only nonce changes per hash.
#[derive(Clone, Debug)]
pub struct WorkerHeaderWork {
    pub job_id: String,
    pub ntime_hex: String,
    pub extranonce2_hex: String,
    pub header_base: [u8; 80],
}

impl WorkerHeaderWork {
    pub fn header_with_nonce(&self, nonce: u32) -> [u8; 80] {
        let mut header = self.header_base;
        header[76..80].copy_from_slice(&nonce.to_le_bytes());
        header
    }
}

pub fn assemble_coinbase(
    coinb1: &[u8],
    extranonce1: &[u8],
    extranonce2: &[u8],
    coinb2: &[u8],
) -> Vec<u8> {
    let mut out =
        Vec::with_capacity(coinb1.len() + extranonce1.len() + extranonce2.len() + coinb2.len());
    out.extend_from_slice(coinb1);
    out.extend_from_slice(extranonce1);
    out.extend_from_slice(extranonce2);
    out.extend_from_slice(coinb2);
    out
}

/// Consensus 80-byte header (veriumd / pool share validation).
pub fn serialize_header(
    version: u32,
    prev_hash_hex: &str,
    merkle_root: &[u8; 32],
    ntime: u32,
    bits_hex: &str,
    nonce: u32,
) -> [u8; 80] {
    let prev_internal = decode_prev_internal(prev_hash_hex).expect("prev_hash hex");
    serialize_header_parts(version, &prev_internal, merkle_root, ntime, bits_hex, nonce)
}

fn serialize_header_parts(
    version: u32,
    prev_internal: &[u8; 32],
    merkle_root: &[u8; 32],
    ntime: u32,
    bits_hex: &str,
    nonce: u32,
) -> [u8; 80] {
    let bits = u32::from_str_radix(bits_hex, 16).unwrap_or(0);
    let mut header = [0u8; 80];
    header[0..4].copy_from_slice(&version.to_le_bytes());
    header[4..36].copy_from_slice(prev_internal);
    header[36..68].copy_from_slice(merkle_root);
    header[68..72].copy_from_slice(&ntime.to_le_bytes());
    header[72..76].copy_from_slice(&bits.to_le_bytes());
    header[76..80].copy_from_slice(&nonce.to_le_bytes());
    header
}

/// Build a worker-local header template (nonce zero) for the mining hot loop.
pub fn prepare_worker_header_work(
    job: &StratumJob,
    extranonce1: &[u8],
    extranonce2: &[u8],
) -> Result<WorkerHeaderWork, String> {
    let coinb1 = hex::decode(&job.coinb1_hex).map_err(|e| format!("coinb1: {e}"))?;
    let coinb2 = hex::decode(&job.coinb2_hex).map_err(|e| format!("coinb2: {e}"))?;
    let mut coinbase = assemble_coinbase(&coinb1, extranonce1, extranonce2, &coinb2);
    if coinbase.len() >= 8 {
        coinbase[4..8].copy_from_slice(&job.ntime.to_le_bytes());
    }
    let coinbase_hash = sha256d(&coinbase);
    let mut steps = Vec::with_capacity(job.merkle_steps_hex.len());
    for h in &job.merkle_steps_hex {
        let decoded = hex::decode(h).map_err(|e| format!("merkle step: {e}"))?;
        let mut b = [0u8; 32];
        let len = decoded.len().min(32);
        b[..len].copy_from_slice(&decoded[..len]);
        steps.push(b);
    }
    let merkle_root = merkle_root_with_coinbase(coinbase_hash, &steps);
    let prev_internal = decode_prev_internal(&job.prev_hash_hex)?;
    let header_base = serialize_header_parts(
        job.version,
        &prev_internal,
        &merkle_root,
        job.ntime,
        &job.bits_hex,
        0,
    );
    Ok(WorkerHeaderWork {
        job_id: job.job_id.clone(),
        ntime_hex: job.ntime_hex.clone(),
        extranonce2_hex: hex::encode(extranonce2),
        header_base,
    })
}

pub fn reconstruct_share(job: &StratumJob, sub: &ShareSubmission) -> ReconstructedShare {
    let en2 = hex::decode(&sub.extranonce2_hex).expect("extranonce2");
    let work = prepare_worker_header_work(job, &sub.extranonce1, &en2).expect("prepare work");
    let consensus_header = work.header_with_nonce(sub.nonce);
    let merkle_root = {
        let mut root = [0u8; 32];
        root.copy_from_slice(&consensus_header[36..68]);
        root
    };
    ReconstructedShare {
        consensus_header,
        merkle_root,
    }
}

pub fn uint32_le_hex(n: u32) -> String {
    hex::encode(n.to_le_bytes())
}

/// Parse `stratum+tcp://host:port` or `host:port`.
pub fn parse_stratum_url(url: &str) -> Result<(String, u16), String> {
    let trimmed = url.trim();
    let without_scheme = trimmed
        .strip_prefix("stratum+tcp://")
        .or_else(|| trimmed.strip_prefix("stratum://"))
        .unwrap_or(trimmed);
    let (host, port_str) = without_scheme
        .rsplit_once(':')
        .ok_or_else(|| format!("invalid stratum url: {url}"))?;
    let port: u16 = port_str
        .parse()
        .map_err(|_| format!("invalid stratum port: {port_str}"))?;
    Ok((host.to_string(), port))
}
