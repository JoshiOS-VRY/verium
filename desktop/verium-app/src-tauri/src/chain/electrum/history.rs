//! Enrich Electrum scripthash history rows with amounts and timestamps from raw txs.

use std::collections::{HashMap, HashSet};

use async_trait::async_trait;

use crate::chain::types::WalletTx;
use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};
use crate::wallet::hd::{p2pkh_script_to_address, sats_to_coins};
use crate::wallet::vericonomy_tx::{decode_verium_tx, VeriumMutableTx};

#[async_trait]
pub trait HistoryTxFetcher {
    async fn fetch_raw_tx_hex(&self, txid: &str) -> AppResult<String>;
}

fn script_matches_wallet(script: &[u8], wallet_scripts: &HashSet<Vec<u8>>) -> bool {
    wallet_scripts.iter().any(|s| s.as_slice() == script)
}

fn category_for(height: i32, delta_sats: i64) -> String {
    if height <= 0 {
        if delta_sats < 0 {
            "send".into()
        } else {
            "unconfirmed".into()
        }
    } else if delta_sats < 0 {
        "send".into()
    } else {
        "receive".into()
    }
}

fn decode_tx_hex(raw_hex: &str) -> AppResult<VeriumMutableTx> {
    let bytes = hex::decode(raw_hex.trim())
        .map_err(|e| AppError::other(format!("tx hex decode: {e}")))?;
    decode_verium_tx(&bytes).map_err(|e| AppError::other(format!("tx decode: {e}")))
}

async fn ensure_decoded(
    txid: &str,
    cache: &mut HashMap<String, VeriumMutableTx>,
    fetcher: &impl HistoryTxFetcher,
) -> AppResult<()> {
    if cache.contains_key(txid) {
        return Ok(());
    }
    let raw = fetcher.fetch_raw_tx_hex(txid).await?;
    cache.insert(txid.to_string(), decode_tx_hex(&raw)?);
    Ok(())
}

/// Copy enriched fields from a prior cache generation so sync refreshes do not wipe them.
pub fn merge_preserved_enrichment(new_rows: &mut [WalletTx], existing: &[WalletTx]) {
    let prior: HashMap<&str, &WalletTx> = existing.iter().map(|t| (t.txid.as_str(), t)).collect();
    for row in new_rows {
        if let Some(old) = prior.get(row.txid.as_str()) {
            if old.time.is_some() {
                row.time = old.time;
                row.amount = old.amount;
                row.address.clone_from(&old.address);
                row.category.clone_from(&old.category);
                if row.fee_sats.is_none() {
                    row.fee_sats = old.fee_sats;
                }
            }
        }
    }
}

/// Populate `amount`, `time`, `address`, and `category` on history rows fetched via
/// `blockchain.scripthash.get_history` (which only returns txid + height).
pub async fn enrich_wallet_history(
    coin: CoinId,
    script_hexes: &[String],
    txs: &mut [WalletTx],
    fetcher: &impl HistoryTxFetcher,
    max_rows: Option<usize>,
) -> AppResult<()> {
    let wallet_scripts: HashSet<Vec<u8>> = script_hexes
        .iter()
        .filter_map(|h| hex::decode(h.trim()).ok())
        .collect();
    if wallet_scripts.is_empty() || txs.is_empty() {
        return Ok(());
    }

    let mut decoded_cache: HashMap<String, VeriumMutableTx> = HashMap::new();
    let mut enriched = 0usize;

    for row in txs.iter_mut() {
        if row.time.is_some() {
            continue;
        }
        if let Some(max) = max_rows {
            if enriched >= max {
                break;
            }
        }

        let row_txid = row.txid.clone();
        if let Err(e) = ensure_decoded(&row_txid, &mut decoded_cache, fetcher).await {
            tracing::debug!("history enrich skip {}: {e}", row_txid);
            continue;
        }

        let prev_txids: Vec<String> = decoded_cache
            .get(&row_txid)
            .map(|tx| {
                tx.inputs
                    .iter()
                    .map(|input| input.previous_output.txid.to_string())
                    .collect()
            })
            .unwrap_or_default();

        for prev_txid in prev_txids {
            if let Err(e) = ensure_decoded(&prev_txid, &mut decoded_cache, fetcher).await {
                tracing::debug!("history enrich prev {} for {}: {e}", prev_txid, row_txid);
            }
        }

        let decoded = decoded_cache.get(&row_txid).unwrap();
        row.time = Some(decoded.n_time as u64);

        let input_refs: Vec<(String, u32)> = decoded
            .inputs
            .iter()
            .map(|input| {
                (
                    input.previous_output.txid.to_string(),
                    input.previous_output.vout,
                )
            })
            .collect();

        let mut prev_output_values: HashMap<(String, u32), i64> = HashMap::new();
        let mut prev_output_is_ours: HashSet<(String, u32)> = HashSet::new();

        for (txid, vout) in &input_refs {
            let key = (txid.clone(), *vout);
            if prev_output_values.contains_key(&key) {
                continue;
            }
            let prev_tx = match decoded_cache.get(txid) {
                Some(t) => t,
                None => continue,
            };
            let vout_idx = *vout as usize;
            if vout_idx >= prev_tx.outputs.len() {
                continue;
            }
            let prev_out = &prev_tx.outputs[vout_idx];
            prev_output_values.insert(key, prev_out.value.to_sat() as i64);
            let prev_script = prev_out.script_pubkey.as_bytes();
            if script_matches_wallet(prev_script, &wallet_scripts) {
                prev_output_is_ours.insert((txid.clone(), *vout));
            }
        }

        let mut delta = 0i64;
        let mut address: Option<String> = None;

        for out in &decoded.outputs {
            let script = out.script_pubkey.as_bytes();
            if script_matches_wallet(script, &wallet_scripts) {
                delta += out.value.to_sat() as i64;
                if address.is_none() {
                    address = p2pkh_script_to_address(coin, script);
                }
            }
        }

        for (txid, vout) in &input_refs {
            if prev_output_is_ours.contains(&(txid.clone(), *vout)) {
                if let Some(value_sats) = prev_output_values.get(&(txid.clone(), *vout)) {
                    delta -= *value_sats;
                }
                if address.is_none() {
                    if let Some(prev_tx) = decoded_cache.get(txid) {
                        let vout_idx = *vout as usize;
                        if vout_idx < prev_tx.outputs.len() {
                            let prev_script =
                                prev_tx.outputs[vout_idx].script_pubkey.as_bytes();
                            address = p2pkh_script_to_address(coin, prev_script);
                        }
                    }
                }
            }
        }

        row.amount = sats_to_coins(delta);
        row.address = address;
        row.category = category_for(row.height, delta);
        enriched += 1;
    }

    Ok(())
}

/// History cache written before enrichment stored zero amounts and no timestamps.
pub fn history_rows_need_enrichment(txs: &[WalletTx]) -> bool {
    txs.iter().any(|t| !t.txid.is_empty() && t.time.is_none())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_unenriched_rows() {
        let rows = vec![WalletTx {
            txid: "abc".into(),
            height: 1,
            fee_sats: None,
            category: "receive".into(),
            amount: 0.0,
            address: None,
            confirmations: 1,
            time: None,
            blockhash: None,
            blockheight: Some(1),
        }];
        assert!(history_rows_need_enrichment(&rows));
    }
}
