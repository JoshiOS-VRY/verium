//! SQLite cache for light-wallet sync state.

use rusqlite::{params, Connection};

use crate::chain::types::Utxo;
use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};

fn cache_path(coin: CoinId) -> std::path::PathBuf {
    crate::config::app_config_base().join(format!("light-cache-{}.sqlite", coin.as_str()))
}

pub struct LightWalletCache {
    conn: Connection,
}

impl LightWalletCache {
    pub fn open(coin: CoinId) -> AppResult<Self> {
        let path = cache_path(coin);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)
            .map_err(|e| AppError::other(format!("sqlite open: {e}")))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sync_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS utxo_cache (
                txid TEXT NOT NULL,
                vout INTEGER NOT NULL,
                value_sats INTEGER NOT NULL,
                script_hex TEXT NOT NULL,
                height INTEGER NOT NULL,
                fetched_at INTEGER NOT NULL,
                PRIMARY KEY (txid, vout)
            );",
        )
        .map_err(|e| AppError::other(format!("sqlite schema: {e}")))?;
        Ok(Self { conn })
    }

    pub fn set_meta(&self, key: &str, value: &str) -> AppResult<()> {
        self.conn
            .execute(
                "INSERT INTO sync_meta(key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )
            .map_err(|e| AppError::other(format!("sqlite meta: {e}")))?;
        Ok(())
    }

    pub fn get_meta(&self, key: &str) -> AppResult<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM sync_meta WHERE key = ?1")
            .map_err(|e| AppError::other(format!("sqlite prepare: {e}")))?;
        let mut rows = stmt
            .query(params![key])
            .map_err(|e| AppError::other(format!("sqlite query: {e}")))?;
        if let Some(row) = rows.next().map_err(|e| AppError::other(format!("sqlite row: {e}")))? {
            let v: String = row.get(0).map_err(|e| AppError::other(format!("sqlite get: {e}")))?;
            return Ok(Some(v));
        }
        Ok(None)
    }

    pub fn cached_wallet_balance(&self) -> AppResult<Option<(i64, i64)>> {
        let confirmed = self
            .get_meta("confirmed_balance_sats")?
            .and_then(|v| v.parse().ok());
        let unconfirmed = self
            .get_meta("unconfirmed_balance_sats")?
            .and_then(|v| v.parse().ok());
        match confirmed {
            Some(c) => Ok(Some((c, unconfirmed.unwrap_or(0)))),
            None => Ok(None),
        }
    }

    pub fn store_wallet_balance(&self, confirmed_sats: i64, unconfirmed_sats: i64) -> AppResult<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.set_meta("confirmed_balance_sats", &confirmed_sats.to_string())?;
        self.set_meta("unconfirmed_balance_sats", &unconfirmed_sats.to_string())?;
        self.set_meta("balance_fetched_at", &now.to_string())
    }

    pub fn funded_script_hexes(&self) -> AppResult<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT script_hex FROM utxo_cache")
            .map_err(|e| AppError::other(format!("sqlite prepare: {e}")))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| AppError::other(format!("sqlite query: {e}")))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| AppError::other(format!("sqlite row: {e}")))?);
        }
        Ok(out)
    }

    pub fn sum_utxo_values(&self) -> AppResult<i64> {
        let mut stmt = self
            .conn
            .prepare("SELECT COALESCE(SUM(value_sats), 0) FROM utxo_cache")
            .map_err(|e| AppError::other(format!("sqlite prepare: {e}")))?;
        let sum: i64 = stmt
            .query_row([], |row| row.get(0))
            .map_err(|e| AppError::other(format!("sqlite sum utxos: {e}")))?;
        Ok(sum)
    }

    pub fn list_utxos(&self) -> AppResult<Vec<Utxo>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT txid, vout, value_sats, script_hex, height FROM utxo_cache ORDER BY height DESC",
            )
            .map_err(|e| AppError::other(format!("sqlite prepare: {e}")))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Utxo {
                    txid: row.get(0)?,
                    vout: row.get(1)?,
                    value_sats: row.get(2)?,
                    script_hex: row.get(3)?,
                    height: row.get(4)?,
                    address: String::new(),
                    confirmations: 0,
                })
            })
            .map_err(|e| AppError::other(format!("sqlite query: {e}")))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| AppError::other(format!("sqlite row: {e}")))?);
        }
        Ok(out)
    }

    pub fn replace_utxos(&self, utxos: &[Utxo]) -> AppResult<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| AppError::other(format!("sqlite tx: {e}")))?;
        tx.execute("DELETE FROM utxo_cache", [])
            .map_err(|e| AppError::other(format!("sqlite clear utxos: {e}")))?;
        for u in utxos {
            tx.execute(
                "INSERT INTO utxo_cache(txid, vout, value_sats, script_hex, height, fetched_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    u.txid,
                    u.vout,
                    u.value_sats,
                    u.script_hex,
                    u.height,
                    now
                ],
            )
            .map_err(|e| AppError::other(format!("sqlite utxo insert: {e}")))?;
        }
        tx.commit()
            .map_err(|e| AppError::other(format!("sqlite commit: {e}")))?;
        Ok(())
    }
}

pub fn clear_coin_cache(coin: CoinId) -> AppResult<()> {
    let path = cache_path(coin);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
