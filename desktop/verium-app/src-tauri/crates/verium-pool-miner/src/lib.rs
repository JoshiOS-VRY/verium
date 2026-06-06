//! Native Verium Stratum pool miner for the Vericonomy desktop wallet.

mod engine;
mod hash;
mod memory;
mod protocol;
pub mod stratum;

pub use engine::{EngineConfig, EngineStatus, PoolMinerEngine};
pub use hash::Scratchpad;
pub use memory::{
    clamp_pool_threads, pool_memory_limits, PoolMemoryLimits, SystemMemory, SCRATCHPAD_BYTES,
};
pub use protocol::{
    difficulty_to_target, hash_meets_target, parse_stratum_url, prepare_worker_header_work,
    reconstruct_share, stratum_to_effective_difficulty, ShareSubmission, StratumJob,
    WorkerHeaderWork,
};
