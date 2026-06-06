//! CPU thread ceiling for pool mining (logical CPUs − 1).

use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub struct SystemMemory {
    pub total_bytes: u64,
    pub available_bytes: u64,
}

/// scrypt² scratchpad size (`SCRYPT_SCRATCHPAD_SIZE` in scrypt2.c) — informational for UI.
pub const SCRATCHPAD_BYTES: u64 = 134_218_239;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolMemoryLimits {
    /// Max mining threads (logical CPUs − 1).
    pub max_safe_threads: u32,
    pub scratchpad_mib: u32,
    pub total_ram_mib: u64,
    pub available_ram_mib: u64,
}

pub fn pool_memory_limits(mem: SystemMemory, cpu_ceiling: Option<u32>) -> PoolMemoryLimits {
    let max_threads = cpu_ceiling.filter(|&n| n > 0).unwrap_or(1);
    PoolMemoryLimits {
        max_safe_threads: max_threads,
        scratchpad_mib: (SCRATCHPAD_BYTES / (1024 * 1024)) as u32,
        total_ram_mib: mem.total_bytes / (1024 * 1024),
        available_ram_mib: mem.available_bytes / (1024 * 1024),
    }
}

/// Clamp `requested` threads to the CPU ceiling (logical CPUs − 1).
pub fn clamp_pool_threads(
    _mem: SystemMemory,
    requested: u32,
    cpu_ceiling: Option<u32>,
) -> Result<u32, String> {
    let requested = requested.max(1);
    let ceiling = cpu_ceiling.filter(|&n| n > 0).unwrap_or(requested);
    Ok(requested.min(ceiling).max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_to_cpu_ceiling() {
        let mem = SystemMemory {
            total_bytes: 8 * 1024 * 1024 * 1024,
            available_bytes: 2 * 1024 * 1024 * 1024,
        };
        assert_eq!(clamp_pool_threads(mem, 99, Some(14)).unwrap(), 14);
        assert_eq!(clamp_pool_threads(mem, 8, Some(14)).unwrap(), 8);
    }
}
