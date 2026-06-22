//! Light-wallet sessions via `vericonomy-wallet-facade` (Tauri-compatible storage).

use std::collections::HashMap;
use std::sync::Arc;

use std::sync::OnceLock;
use tokio::sync::Mutex;
use vericonomy_chain::types::WalletTx;
use vericonomy_storage::{LightKeystoreService, WalletStores};
use vericonomy_storage_file::{FileKeystoreStore, FileSecretStore};
use vericonomy_storage_sqlite::SqliteWalletCache;
use vericonomy_wallet_engine::DEFAULT_TX_FEE_COINS_PER_KB;
use vericonomy_wallet_facade::explorer_history::ExplorerHistorySource;
use vericonomy_wallet_facade::LightWalletSession;

use crate::coin::CoinId;
use crate::error::{HostError, HostResult};
use crate::prefs::{self, CoinSdkExt, map_wallet_err};

type Keystore = FileKeystoreStore<FileSecretStore>;
type Cache = SqliteWalletCache;
type Session = LightWalletSession<Keystore, Cache, Cache>;

static SESSIONS: OnceLock<Mutex<HashMap<String, Arc<Session>>>> = OnceLock::new();
static HISTORY: ExplorerHistorySource = ExplorerHistorySource;

fn sessions() -> &'static Mutex<HashMap<String, Arc<Session>>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn open_stores() -> WalletStores<Keystore, Cache, Cache> {
    let base = crate::config::app_config_base();
    let _ = std::fs::create_dir_all(&base);
    let secret = FileSecretStore::new(base.join("secure"));
    let keystore = FileKeystoreStore::new(&base, secret);
    let cache = SqliteWalletCache::new(&base);
    WalletStores::new(keystore, cache.clone(), cache)
}

pub async fn session_for(coin: CoinId) -> HostResult<Arc<Session>> {
    let key = coin.as_str().to_string();
    let mut guard = sessions().lock().await;
    if let Some(s) = guard.get(&key) {
        return Ok(Arc::clone(s));
    }
    let prefs = prefs::load_prefs()?;
    let servers = prefs::electrum_servers_for(&prefs, coin);
    let stores = open_stores();
    let session = LightWalletSession::new(
        coin.to_sdk(),
        &servers,
        LightKeystoreService::new(stores.keystore),
        stores.utxo_cache,
        stores.tx_cache,
    )
    .map_err(map_wallet_err)?;
    let arc = Arc::new(session);
    guard.insert(key, Arc::clone(&arc));
    Ok(arc)
}

pub async fn light_wallet_exists(coin: CoinId) -> HostResult<bool> {
    let session = session_for(coin).await?;
    session.exists().await.map_err(map_wallet_err)
}

pub async fn light_wallet_is_unlocked(coin: CoinId) -> HostResult<bool> {
    let session = session_for(coin).await?;
    let status = session.status().await.map_err(map_wallet_err)?;
    Ok(status.unlocked)
}

pub async fn light_wallet_import(
    coin: CoinId,
    seed_secret: &str,
    passphrase: &str,
    label: Option<&str>,
) -> HostResult<()> {
    let trimmed = vericonomy_hd::normalize_light_wallet_seed(seed_secret);
    let sdk = coin.to_sdk();
    if vericonomy_hd::is_hd_master_secret(sdk, &trimmed) {
        vericonomy_hd::parse_root_xpriv(sdk, &trimmed).map_err(map_wallet_err)?;
    } else if !vericonomy_wallet_core::validate_mnemonic(&trimmed) {
        return Err(HostError::other(
            "invalid recovery phrase — enter 24-word BIP39 or HD master xprv from Security → Export",
        ));
    }
    let session = session_for(coin).await?;
    session
        .import_wallet(&trimmed, passphrase, label)
        .await
        .map_err(map_wallet_err)?;
    session
        .sync(&HISTORY)
        .await
        .map_err(map_wallet_err)?;
    Ok(())
}

pub async fn light_wallet_unlock(coin: CoinId, passphrase: &str) -> HostResult<()> {
    let session = session_for(coin).await?;
    session
        .unlock(passphrase, 4 * 60 * 60)
        .await
        .map_err(map_wallet_err)?;
    session
        .sync(&HISTORY)
        .await
        .map_err(map_wallet_err)?;
    Ok(())
}

pub async fn light_wallet_lock(coin: CoinId) -> HostResult<()> {
    let session = session_for(coin).await?;
    session.lock().await.map_err(map_wallet_err)
}

pub async fn light_wallet_sync(coin: CoinId) -> HostResult<()> {
    let session = session_for(coin).await?;
    session
        .sync(&HISTORY)
        .await
        .map_err(map_wallet_err)?;
    Ok(())
}

pub async fn light_wallet_balance(coin: CoinId) -> HostResult<vericonomy_chain::types::WalletBalance> {
    let session = session_for(coin).await?;
    session.balance().await.map_err(map_wallet_err)
}

pub async fn light_wallet_new_address(coin: CoinId, passphrase: &str) -> HostResult<String> {
    let session = session_for(coin).await?;
    session
        .new_receive_address(passphrase)
        .await
        .map_err(map_wallet_err)
}

pub async fn light_wallet_send(
    coin: CoinId,
    address: &str,
    amount: f64,
    fee_rate: Option<f64>,
    passphrase: &str,
) -> HostResult<String> {
    let session = session_for(coin).await?;
    let amount_sats = vericonomy_hd::coins_to_sats(amount);
    if amount_sats <= 0 {
        return Err(HostError::other("amount must be greater than zero"));
    }
    let prefs = prefs::load_prefs()?;
    let default_rate = prefs
        .extra
        .get("tx_fee_rate_vrm_per_kb")
        .and_then(|v| v.as_f64())
        .unwrap_or(DEFAULT_TX_FEE_COINS_PER_KB);
    let rate = fee_rate.unwrap_or(default_rate);
    let result = session
        .send_to_address(&HISTORY, address, amount_sats, Some(rate), passphrase)
        .await
        .map_err(map_wallet_err)?;
    let _ = session.sync(&HISTORY).await;
    Ok(result.txid)
}

pub async fn light_wallet_list_transactions(
    coin: CoinId,
    limit: usize,
) -> HostResult<Vec<WalletTx>> {
    let session = session_for(coin).await?;
    session
        .list_transactions(&HISTORY, limit)
        .await
        .map_err(map_wallet_err)
}

pub async fn light_wallet_export_secret(coin: CoinId, passphrase: &str) -> HostResult<String> {
    let stores = open_stores();
    let keystore = LightKeystoreService::new(stores.keystore);
    keystore
        .unlocked_mnemonic(coin.to_sdk(), passphrase)
        .await
        .map_err(map_wallet_err)
}

pub fn is_light_mode(coin: CoinId) -> HostResult<bool> {
    let prefs = prefs::load_prefs()?;
    Ok(prefs::wallet_mode_for(&prefs, coin).is_light())
}
