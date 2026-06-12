//! SQLite cache for light-wallet sync state.

use rusqlite::{params, Connection};

use crate::chain::types::{Utxo, WalletTx};
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
            );
            CREATE TABLE IF NOT EXISTS tx_history (
                txid TEXT NOT NULL,
                category TEXT NOT NULL,
                sort_time INTEGER NOT NULL,
                payload TEXT NOT NULL,
                PRIMARY KEY (txid, category)
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

    pub fn remove_utxo(&self, txid: &str, vout: u32) -> AppResult<bool> {
        let n = self
            .conn
            .execute(
                "DELETE FROM utxo_cache WHERE txid = ?1 AND vout = ?2",
                params![txid, vout],
            )
            .map_err(|e| AppError::other(format!("sqlite utxo delete: {e}")))?;
        Ok(n > 0)
    }

    pub fn upsert_utxo(&self, utxo: &Utxo) -> AppResult<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.conn
            .execute(
                "INSERT INTO utxo_cache(txid, vout, value_sats, script_hex, height, fetched_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(txid, vout) DO UPDATE SET
                   value_sats = excluded.value_sats,
                   script_hex = excluded.script_hex,
                   height = excluded.height,
                   fetched_at = excluded.fetched_at",
                params![
                    utxo.txid,
                    utxo.vout,
                    utxo.value_sats,
                    utxo.script_hex,
                    utxo.height,
                    now
                ],
            )
            .map_err(|e| AppError::other(format!("sqlite utxo upsert: {e}")))?;
        Ok(())
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

    /// Reconcile the cached UTXO set to `utxos` using an incremental diff:
    /// only changed/new rows are upserted and only vanished rows are deleted.
    /// When the set is unchanged (the steady-state case) this performs **zero**
    /// writes, avoiding the previous DELETE-all + INSERT-all write amplification
    /// on every sync.
    /// Returns `true` when the cached set changed (insert, update, or delete).
    pub fn replace_utxos(&self, utxos: &[Utxo]) -> AppResult<bool> {
        use std::collections::{HashMap, HashSet};

        let mut changed = false;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // Snapshot existing rows keyed by (txid, vout).
        let mut existing: HashMap<(String, u32), (i64, String, u32)> = HashMap::new();
        {
            let mut stmt = self
                .conn
                .prepare("SELECT txid, vout, value_sats, script_hex, height FROM utxo_cache")
                .map_err(|e| AppError::other(format!("sqlite prepare: {e}")))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        (row.get::<_, String>(0)?, row.get::<_, u32>(1)?),
                        (
                            row.get::<_, i64>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, u32>(4)?,
                        ),
                    ))
                })
                .map_err(|e| AppError::other(format!("sqlite query: {e}")))?;
            for row in rows {
                let (k, v) = row.map_err(|e| AppError::other(format!("sqlite row: {e}")))?;
                existing.insert(k, v);
            }
        }

        let incoming_keys: HashSet<(String, u32)> =
            utxos.iter().map(|u| (u.txid.clone(), u.vout)).collect();

        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| AppError::other(format!("sqlite tx: {e}")))?;

        // Upsert only new or changed rows.
        {
            let mut upsert = tx
                .prepare(
                    "INSERT INTO utxo_cache(txid, vout, value_sats, script_hex, height, fetched_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                     ON CONFLICT(txid, vout) DO UPDATE SET
                       value_sats = excluded.value_sats,
                       script_hex = excluded.script_hex,
                       height = excluded.height,
                       fetched_at = excluded.fetched_at",
                )
                .map_err(|e| AppError::other(format!("sqlite prepare upsert: {e}")))?;
            for u in utxos {
                let unchanged = matches!(
                    existing.get(&(u.txid.clone(), u.vout)),
                    Some((value, script, height))
                        if *value == u.value_sats && script == &u.script_hex && *height == u.height
                );
                if unchanged {
                    continue;
                }
                changed = true;
                upsert
                    .execute(params![
                        u.txid,
                        u.vout,
                        u.value_sats,
                        u.script_hex,
                        u.height,
                        now
                    ])
                    .map_err(|e| AppError::other(format!("sqlite utxo upsert: {e}")))?;
            }
        }

        // Delete rows that are no longer present.
        {
            let mut delete = tx
                .prepare("DELETE FROM utxo_cache WHERE txid = ?1 AND vout = ?2")
                .map_err(|e| AppError::other(format!("sqlite prepare delete: {e}")))?;
            for (txid, vout) in existing.keys() {
                if !incoming_keys.contains(&(txid.clone(), *vout)) {
                    changed = true;
                    delete
                        .execute(params![txid, vout])
                        .map_err(|e| AppError::other(format!("sqlite utxo delete: {e}")))?;
                }
            }
        }

        tx.commit()
            .map_err(|e| AppError::other(format!("sqlite commit: {e}")))?;
        Ok(changed)
    }

    /// Replace the cached transaction history with `txs` (full set, newest first
    /// determined by `time`). Serialized as JSON per row so the read path is a
    /// pure local SQLite query — no Electrum `get_history` calls.
    pub fn replace_tx_history(&self, txs: &[WalletTx]) -> AppResult<()> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| AppError::other(format!("sqlite tx: {e}")))?;
        tx.execute("DELETE FROM tx_history", [])
            .map_err(|e| AppError::other(format!("sqlite clear history: {e}")))?;
        {
            let mut stmt = tx
                .prepare(
                    "INSERT OR REPLACE INTO tx_history(txid, category, sort_time, payload)
                     VALUES (?1, ?2, ?3, ?4)",
                )
                .map_err(|e| AppError::other(format!("sqlite prepare history: {e}")))?;
            for t in txs {
                let payload = serde_json::to_string(t)
                    .map_err(|e| AppError::other(format!("history encode: {e}")))?;
                let sort_time = t.time.unwrap_or(0) as i64;
                let storage_category = crate::wallet::listtransactions_rows::cache_storage_category(t);
                stmt.execute(params![t.txid, storage_category, sort_time, payload])
                    .map_err(|e| AppError::other(format!("sqlite history insert: {e}")))?;
            }
        }
        tx.commit()
            .map_err(|e| AppError::other(format!("sqlite commit: {e}")))?;
        Ok(())
    }

    pub fn list_tx_history(&self, limit: usize) -> AppResult<Vec<WalletTx>> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload FROM tx_history ORDER BY sort_time DESC LIMIT ?1")
            .map_err(|e| AppError::other(format!("sqlite prepare: {e}")))?;
        let rows = stmt
            .query_map(params![limit as i64], |row| row.get::<_, String>(0))
            .map_err(|e| AppError::other(format!("sqlite query: {e}")))?;
        let mut out = Vec::new();
        for row in rows {
            let payload = row.map_err(|e| AppError::other(format!("sqlite row: {e}")))?;
            if let Ok(tx) = serde_json::from_str::<WalletTx>(&payload) {
                out.push(tx);
            }
        }
        Ok(out)
    }

    pub fn tx_history_count(&self) -> AppResult<usize> {
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM tx_history")
            .map_err(|e| AppError::other(format!("sqlite prepare: {e}")))?;
        let n: i64 = stmt
            .query_row([], |row| row.get(0))
            .map_err(|e| AppError::other(format!("sqlite count: {e}")))?;
        Ok(n.max(0) as usize)
    }
}

pub fn clear_coin_cache(coin: CoinId) -> AppResult<()> {
    let path = cache_path(coin);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
