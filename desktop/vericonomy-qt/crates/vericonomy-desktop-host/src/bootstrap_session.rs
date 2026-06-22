//! Per-coin bootstrap cancel flags (Tauri `AppState::bootstrap_*` subset).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use tokio::sync::Mutex;

use crate::coin::CoinId;

static SESSIONS: OnceLock<Mutex<HashMap<CoinId, Arc<AtomicBool>>>> = OnceLock::new();

fn sessions() -> &'static Mutex<HashMap<CoinId, Arc<AtomicBool>>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn begin(coin: CoinId) -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    if let Ok(mut guard) = sessions().try_lock() {
        guard.insert(coin, flag.clone());
    }
    flag
}

pub async fn begin_async(coin: CoinId) -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    sessions().lock().await.insert(coin, flag.clone());
    flag
}

pub fn end(coin: CoinId) {
    if let Ok(mut guard) = sessions().try_lock() {
        guard.remove(&coin);
    }
}

pub async fn end_async(coin: CoinId) {
    sessions().lock().await.remove(&coin);
}

pub fn request_cancel(coin: CoinId) {
    if let Ok(guard) = sessions().try_lock() {
        if let Some(flag) = guard.get(&coin) {
            flag.store(true, Ordering::SeqCst);
        }
    }
}

pub async fn request_cancel_async(coin: CoinId) {
    if let Some(flag) = sessions().lock().await.get(&coin) {
        flag.store(true, Ordering::SeqCst);
    }
}

pub fn is_cancelled(cancel: &AtomicBool) -> bool {
    cancel.load(Ordering::SeqCst)
}
