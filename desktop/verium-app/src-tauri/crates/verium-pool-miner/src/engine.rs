//! In-process pool mining engine: Stratum I/O thread + worker threads.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use tracing::{debug, info, warn};

use crate::hash::Scratchpad;
use crate::protocol::{
    difficulty_to_target, hash_meets_target_fast, prepare_worker_header_work,
    stratum_to_effective_difficulty, uint32_le_hex, StratumJob, WorkerHeaderWork,
};
use crate::stratum::{StratumClient, StratumEvent};

/// Re-read pool job state every N hashes (scrypt is slow; avoids RwLock per nonce).
const JOB_REFRESH_INTERVAL: u32 = 32;
const MAX_SHARE_QUEUE: usize = 256;

#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub threads: u32,
}

#[derive(Clone, Debug, Default)]
pub struct EngineStatus {
    pub running: bool,
    pub hashrate_hm: f64,
    pub worker: String,
    pub last_log_line: String,
    pub active_threads: u32,
}

struct JobState {
    job: Option<Arc<StratumJob>>,
    extranonce1: Vec<u8>,
    extranonce2_size: usize,
    wire_difficulty: f64,
    effective_difficulty: f64,
    target: [u8; 32],
    generation: u64,
}

impl Default for JobState {
    fn default() -> Self {
        Self {
            job: None,
            extranonce1: Vec::new(),
            extranonce2_size: 4,
            wire_difficulty: 0.0,
            effective_difficulty: 0.0,
            target: [0xff; 32],
            generation: 0,
        }
    }
}

struct ShareFound {
    job_id: String,
    extranonce2_hex: String,
    ntime_hex: String,
    nonce_hex: String,
}

pub struct PoolMinerEngine {
    cancel: Arc<AtomicBool>,
    status: Arc<Mutex<EngineStatus>>,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl PoolMinerEngine {
    pub fn new() -> Self {
        Self {
            cancel: Arc::new(AtomicBool::new(false)),
            status: Arc::new(Mutex::new(EngineStatus::default())),
            join: Mutex::new(None),
        }
    }

    pub fn status(&self) -> EngineStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn set_log_line(&self, line: impl Into<String>) {
        if let Ok(mut st) = self.status.lock() {
            st.last_log_line = line.into();
        }
    }

    pub fn stop(&self) {
        self.cancel.store(true, Ordering::SeqCst);
        if let Some(handle) = self.join.lock().unwrap().take() {
            let _ = handle.join();
        }
        let mut st = self.status.lock().unwrap();
        st.running = false;
        st.hashrate_hm = 0.0;
    }

    pub fn start(&self, cfg: EngineConfig) -> Result<(), String> {
        self.stop();
        self.cancel.store(false, Ordering::SeqCst);

        let threads = cfg.threads.max(1);

        {
            let mut st = self.status.lock().unwrap();
            st.running = true;
            st.worker = cfg.username.clone();
            st.hashrate_hm = 0.0;
            st.active_threads = threads;
            st.last_log_line = format!(
                "Connecting to {}:{} ({} threads)…",
                cfg.host, cfg.port, threads
            );
        }

        let cancel = Arc::clone(&self.cancel);
        let status = Arc::clone(&self.status);

        let handle = thread::spawn(move || {
            if let Err(e) = run_engine(cfg, threads, cancel, Arc::clone(&status)) {
                warn!("pool miner engine stopped: {e}");
                update_status(&status, |s| {
                    s.running = false;
                    s.hashrate_hm = 0.0;
                    s.last_log_line = format!("Pool miner error: {e}");
                });
            }
        });
        *self.join.lock().unwrap() = Some(handle);
        Ok(())
    }
}

fn run_engine(
    cfg: EngineConfig,
    thread_count: u32,
    cancel: Arc<AtomicBool>,
    status: Arc<Mutex<EngineStatus>>,
) -> Result<(), String> {
    let job_state = Arc::new(RwLock::new(JobState::default()));
    let hash_counter = Arc::new(AtomicU64::new(0));
    let share_queue: Arc<Mutex<Vec<ShareFound>>> = Arc::new(Mutex::new(Vec::new()));

    let mut worker_handles = Vec::new();
    for thread_id in 0..thread_count {
        let scratch = Scratchpad::try_new().ok_or_else(|| {
            format!(
                "Failed to allocate scrypt scratchpad for thread {thread_id} (~128 MiB). Lower thread count or free RAM."
            )
        })?;
        let job_state = Arc::clone(&job_state);
        let cancel = Arc::clone(&cancel);
        let hash_counter = Arc::clone(&hash_counter);
        let share_queue = Arc::clone(&share_queue);
        let handle = thread::spawn(move || {
            worker_loop(
                thread_id,
                thread_count,
                scratch,
                job_state,
                cancel,
                hash_counter,
                share_queue,
            );
        });
        worker_handles.push(handle);
    }

    let mut client = StratumClient::connect(&cfg.host, cfg.port)?;
    client.subscribe()?;
    client.authorize(&cfg.username, &cfg.password)?;

    update_status(&status, |st| {
        st.last_log_line = "Subscribed; waiting for jobs…".into();
    });

    let mut hashrate_window_start = Instant::now();
    let mut last_hash_total = 0u64;

    loop {
        if cancel.load(Ordering::SeqCst) {
            break;
        }

        let pending: Vec<ShareFound> = {
            let mut q = share_queue.lock().unwrap();
            std::mem::take(&mut *q)
        };
        for share in pending {
            let id = client.submit_share(
                &cfg.username,
                &share.job_id,
                &share.extranonce2_hex,
                &share.ntime_hex,
                &share.nonce_hex,
            )?;
            debug!("submitted share id={id}");
            update_status(&status, |st| {
                st.last_log_line = format!("Share submitted (job {})", share.job_id);
            });
        }

        match client.read_event()? {
            None => {
                thread::sleep(Duration::from_millis(10));
            }
            Some(StratumEvent::Subscribed {
                extranonce1_hex,
                extranonce2_size,
            }) => {
                let en1 = hex::decode(&extranonce1_hex).unwrap_or_default();
                let mut st = job_state.write().unwrap();
                st.extranonce1 = en1;
                st.extranonce2_size = extranonce2_size;
                update_status(&status, |s| {
                    s.last_log_line = format!("Subscribed (extranonce1={extranonce1_hex})");
                });
            }
            Some(StratumEvent::Authorized) => {
                update_status(&status, |s| {
                    s.last_log_line = "Authorized on pool".into();
                });
            }
            Some(StratumEvent::SetDifficulty(wire)) => {
                let effective = stratum_to_effective_difficulty(wire);
                let target = difficulty_to_target(effective);
                let mut st = job_state.write().unwrap();
                st.wire_difficulty = wire;
                st.effective_difficulty = effective;
                st.target = target;
                st.generation += 1;
                update_status(&status, |s| {
                    s.last_log_line = format!("Difficulty {effective:.6} (wire {wire})");
                });
            }
            Some(StratumEvent::Notify(notify)) => {
                let mut st = job_state.write().unwrap();
                if notify.clean_jobs {
                    st.generation += 1;
                }
                st.job = Some(Arc::new(notify.job));
                st.generation += 1;
                update_status(&status, |s| {
                    s.last_log_line = "New pool job received".into();
                });
            }
            Some(StratumEvent::SubmitResult {
                accepted,
                error,
                ..
            }) => {
                update_status(&status, |s| {
                    s.last_log_line = if accepted {
                        "Share accepted by pool".into()
                    } else {
                        format!("Share rejected: {}", error.unwrap_or_default())
                    };
                });
            }
            Some(StratumEvent::Disconnected(reason)) => {
                return Err(format!("stratum disconnected: {reason}"));
            }
        }

        let elapsed = hashrate_window_start.elapsed();
        if elapsed >= Duration::from_secs(5) {
            let total = hash_counter.load(Ordering::Relaxed);
            let delta = total.saturating_sub(last_hash_total);
            let hm = delta as f64 * 60.0 / elapsed.as_secs_f64();
            update_status(&status, |s| s.hashrate_hm = hm);
            last_hash_total = total;
            hashrate_window_start = Instant::now();
        }
    }

    for h in worker_handles {
        let _ = h.join();
    }
    update_status(&status, |s| s.running = false);
    Ok(())
}

fn worker_loop(
    thread_id: u32,
    thread_count: u32,
    scratch: Scratchpad,
    job_state: Arc<RwLock<JobState>>,
    cancel: Arc<AtomicBool>,
    hash_counter: Arc<AtomicU64>,
    share_queue: Arc<Mutex<Vec<ShareFound>>>,
) {
    let en2_hex = if thread_id == 0 && thread_count == 1 {
        "00000000".to_string()
    } else {
        format!("{:08x}", thread_id)
    };

    let mut local_gen = 0u64;
    let mut nonce: u32 = thread_id;
    let mut header_work: Option<WorkerHeaderWork> = None;
    let mut target = [0xffu8; 32];
    let mut hashes_since_refresh = 0u32;

    while !cancel.load(Ordering::SeqCst) {
        if header_work.is_none() || hashes_since_refresh >= JOB_REFRESH_INTERVAL {
            hashes_since_refresh = 0;
            let snapshot = {
                let st = job_state.read().unwrap();
                if st.job.is_none() || st.extranonce1.is_empty() || st.effective_difficulty <= 0.0
                {
                    None
                } else {
                    Some((
                        st.generation,
                        st.target,
                        Arc::clone(st.job.as_ref().unwrap()),
                        st.extranonce1.clone(),
                        st.extranonce2_size,
                    ))
                }
            };

            let Some((gen, new_target, job, en1, en2_size)) = snapshot else {
                thread::sleep(Duration::from_millis(200));
                continue;
            };

            target = new_target;
            if gen != local_gen || header_work.is_none() {
                local_gen = gen;
                nonce = thread_id;
                let en2_hex = if en2_size == 4 {
                    en2_hex.clone()
                } else {
                    "00".repeat(en2_size * 2)
                };
                let en2 = hex::decode(&en2_hex).unwrap_or_default();
                match prepare_worker_header_work(&job, &en1, &en2) {
                    Ok(work) => header_work = Some(work),
                    Err(e) => {
                        warn!("thread {thread_id}: prepare work failed: {e}");
                        thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                }
            }
        }

        let Some(work) = header_work.as_ref() else {
            thread::sleep(Duration::from_millis(200));
            continue;
        };

        let header = work.header_with_nonce(nonce);
        let hash = scratch.hash(&header);
        hash_counter.fetch_add(1, Ordering::Relaxed);
        hashes_since_refresh += 1;

        if hash_meets_target_fast(&hash, &target) {
            let found = ShareFound {
                job_id: work.job_id.clone(),
                extranonce2_hex: work.extranonce2_hex.clone(),
                ntime_hex: work.ntime_hex.clone(),
                nonce_hex: uint32_le_hex(nonce),
            };
            let mut q = share_queue.lock().unwrap();
            if q.len() < MAX_SHARE_QUEUE {
                q.push(found);
            }
            info!("thread {thread_id}: found share at nonce={nonce}");
        }

        nonce = nonce.wrapping_add(thread_count);
        if nonce % 4096 == thread_id {
            thread::yield_now();
        }
    }
}

fn update_status(status: &Arc<Mutex<EngineStatus>>, f: impl FnOnce(&mut EngineStatus)) {
    if let Ok(mut st) = status.lock() {
        f(&mut st);
    }
}
