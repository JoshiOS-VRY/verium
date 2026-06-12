//! Build wallet history rows the same way Core `listtransactions` does:
//! payment sends and change/receive credits — not "spent a 280 VRM input" debits.

use std::collections::HashSet;

use crate::chain::types::WalletTx;
use crate::coin_profile::CoinId;
use crate::indexer_api::{
    IndexerAddressEvent, IndexerAmount, IndexerTransactionDetail, IndexerVin, IndexerVout,
};
use crate::wallet::hd::{coins_to_sats, p2pkh_script_to_address, sats_to_coins};
use crate::wallet::vericonomy_tx::VeriumMutableTx;

pub fn wallet_tx_row_key(tx: &WalletTx) -> String {
    format!(
        "{}:{}:{}",
        tx.txid,
        tx.category,
        tx.address.as_deref().unwrap_or("")
    )
}

/// SQLite `tx_history.category` primary-key suffix (payload keeps the real category).
pub fn cache_storage_category(tx: &WalletTx) -> String {
    wallet_tx_row_key(tx)
}

fn confirmations_for(tip: Option<u32>, block_height: Option<u64>) -> i32 {
    match (tip, block_height) {
        (Some(tip), Some(bh)) if tip >= bh as u32 => (tip - bh as u32 + 1) as i32,
        _ => 0,
    }
}

fn category_receive(height: i32) -> String {
    if height <= 0 {
        "unconfirmed".into()
    } else {
        "receive".into()
    }
}

fn indexer_amount_sats(amount: Option<&IndexerAmount>) -> i64 {
    match amount {
        Some(a) => a
            .amount
            .trim()
            .parse::<f64>()
            .map(coins_to_sats)
            .unwrap_or(0),
        None => 0,
    }
}

struct OutputView {
    address: String,
    value_sats: i64,
}

fn make_row(
    txid: &str,
    height: i32,
    blockheight: Option<u32>,
    time: Option<u64>,
    blockhash: Option<String>,
    confirmations: i32,
    category: &str,
    amount_coins: f64,
    address: &str,
) -> WalletTx {
    WalletTx {
        txid: txid.to_string(),
        height,
        fee_sats: None,
        category: category.to_string(),
        amount: amount_coins,
        address: Some(address.to_string()),
        confirmations,
        time,
        blockhash,
        blockheight,
    }
}

pub fn rows_from_indexer_transaction(
    detail: &IndexerTransactionDetail,
    wallet_addresses: &HashSet<String>,
    tip: Option<u32>,
) -> Vec<WalletTx> {
    if !detail.found {
        return Vec::new();
    }
    let summary = detail.transaction.as_ref();
    let txid = summary
        .map(|t| t.txid.as_str())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            if !detail.txid.is_empty() {
                Some(detail.txid.as_str())
            } else {
                None
            }
        });
    let txid = match txid {
        Some(t) => t.to_string(),
        None => return Vec::new(),
    };

    let height = summary
        .and_then(|t| t.block_height)
        .map(|h| h as i32)
        .unwrap_or(0);
    let blockheight = summary.and_then(|t| t.block_height).map(|h| h as u32);
    let time = summary.and_then(|t| t.time);
    let blockhash = summary.and_then(|t| t.block_hash.clone());
    let confirmations = confirmations_for(tip, summary.and_then(|t| t.block_height));

    if !detail.address_events.is_empty() {
        return rows_from_address_events(
            &txid,
            height,
            blockheight,
            time,
            blockhash,
            confirmations,
            &detail.address_events,
            wallet_addresses,
        );
    }

    let wallet_outputs = indexer_wallet_outputs(&detail.outputs, wallet_addresses);
    let external_outputs = indexer_external_outputs(&detail.outputs, wallet_addresses);
    let is_spend = indexer_is_spend_tx(
        &detail.inputs,
        wallet_addresses,
        &wallet_outputs,
        &external_outputs,
    );
    rows_from_outputs(
        &txid,
        height,
        blockheight,
        time,
        blockhash,
        confirmations,
        is_spend,
        wallet_outputs,
        external_outputs,
    )
}

fn indexer_is_spend_tx(
    inputs: &[IndexerVin],
    wallet_addresses: &HashSet<String>,
    wallet_outputs: &[OutputView],
    external_outputs: &[OutputView],
) -> bool {
    if !external_outputs.is_empty() {
        return true;
    }
    if spend_from_indexer_inputs(inputs, wallet_addresses) {
        return true;
    }
    wallet_outputs.len() > 1
}

fn spend_from_indexer_inputs(inputs: &[IndexerVin], wallet_addresses: &HashSet<String>) -> bool {
    inputs.iter().any(|vin| {
        vin.address
            .as_deref()
            .map(|a| wallet_addresses.contains(a.trim()))
            .unwrap_or(false)
    })
}

fn indexer_wallet_outputs(
    outputs: &[IndexerVout],
    wallet_addresses: &HashSet<String>,
) -> Vec<OutputView> {
    let mut out = Vec::new();
    for vout in outputs {
        let addr = vout.address.as_deref().unwrap_or("").trim();
        if addr.is_empty() || !wallet_addresses.contains(addr) {
            continue;
        }
        let value_sats = indexer_amount_sats(vout.value.as_ref());
        if value_sats <= 0 {
            continue;
        }
        out.push(OutputView {
            address: addr.to_string(),
            value_sats,
        });
    }
    out
}

fn indexer_external_outputs(
    outputs: &[IndexerVout],
    wallet_addresses: &HashSet<String>,
) -> Vec<OutputView> {
    let mut out = Vec::new();
    for vout in outputs {
        let addr = vout.address.as_deref().unwrap_or("").trim();
        if addr.is_empty() || wallet_addresses.contains(addr) {
            continue;
        }
        let value_sats = indexer_amount_sats(vout.value.as_ref());
        if value_sats <= 0 {
            continue;
        }
        out.push(OutputView {
            address: addr.to_string(),
            value_sats,
        });
    }
    out
}

fn rows_from_address_events(
    txid: &str,
    height: i32,
    blockheight: Option<u32>,
    time: Option<u64>,
    blockhash: Option<String>,
    confirmations: i32,
    events: &[IndexerAddressEvent],
    wallet_addresses: &HashSet<String>,
) -> Vec<WalletTx> {
    let mut rows = Vec::new();
    for ev in events {
        let addr = ev.address.trim();
        if addr.is_empty() {
            continue;
        }
        let delta_sats = indexer_amount_sats(ev.delta.as_ref());
        if delta_sats == 0 {
            continue;
        }
        let is_ours = wallet_addresses.contains(addr);
        if !is_ours && delta_sats > 0 {
            continue;
        }
        let (category, amount_coins) = if delta_sats < 0 || ev.event_type.as_deref() == Some("send") {
            ("send", -sats_to_coins(delta_sats.abs()))
        } else if height <= 0 {
            ("unconfirmed", sats_to_coins(delta_sats))
        } else {
            ("receive", sats_to_coins(delta_sats))
        };
        rows.push(make_row(
            txid,
            height,
            blockheight,
            time,
            blockhash.clone(),
            confirmations,
            category,
            amount_coins,
            addr,
        ));
    }
    rows
}

fn change_output_address(wallet_outputs: &[OutputView]) -> Option<String> {
    wallet_outputs
        .iter()
        .max_by_key(|o| o.value_sats)
        .map(|o| o.address.clone())
}

fn rows_from_outputs(
    txid: &str,
    height: i32,
    blockheight: Option<u32>,
    time: Option<u64>,
    blockhash: Option<String>,
    confirmations: i32,
    is_spend: bool,
    wallet_outputs: Vec<OutputView>,
    external_outputs: Vec<OutputView>,
) -> Vec<WalletTx> {
    let mut rows = Vec::new();

    if !is_spend {
        for out in wallet_outputs {
            rows.push(make_row(
                txid,
                height,
                blockheight,
                time,
                blockhash.clone(),
                confirmations,
                &category_receive(height),
                sats_to_coins(out.value_sats),
                &out.address,
            ));
        }
        return rows;
    }

    let change_addr = change_output_address(&wallet_outputs);

    for out in wallet_outputs {
        rows.push(make_row(
            txid,
            height,
            blockheight,
            time,
            blockhash.clone(),
            confirmations,
            &category_receive(height),
            sats_to_coins(out.value_sats),
            &out.address,
        ));
        if change_addr.as_deref() != Some(out.address.as_str()) {
            rows.push(make_row(
                txid,
                height,
                blockheight,
                time,
                blockhash.clone(),
                confirmations,
                "send",
                -sats_to_coins(out.value_sats),
                &out.address,
            ));
        }
    }

    for out in external_outputs {
        rows.push(make_row(
            txid,
            height,
            blockheight,
            time,
            blockhash.clone(),
            confirmations,
            "send",
            -sats_to_coins(out.value_sats),
            &out.address,
        ));
    }

    rows
}

pub fn rows_from_decoded_tx(
    coin: CoinId,
    txid: &str,
    decoded: &VeriumMutableTx,
    height: i32,
    tip: Option<u32>,
    wallet_scripts: &HashSet<Vec<u8>>,
    wallet_addresses: &HashSet<String>,
    prev_output_is_ours: &HashSet<(String, u32)>,
) -> Vec<WalletTx> {
    let blockheight = if height > 0 {
        Some(height as u32)
    } else {
        None
    };
    let time = Some(decoded.n_time as u64);
    let confirmations = confirmations_for(tip, blockheight.map(|h| h as u64));

    let is_spend = decoded.inputs.iter().any(|input| {
        prev_output_is_ours.contains(&(
            input.previous_output.txid.to_string(),
            input.previous_output.vout,
        ))
    });

    let mut wallet_outputs = Vec::new();
    let mut external_outputs = Vec::new();

    for out in &decoded.outputs {
        let script = out.script_pubkey.as_bytes();
        let value_sats = out.value.to_sat() as i64;
        if value_sats <= 0 {
            continue;
        }
        let addr = p2pkh_script_to_address(coin, script);
        let view = OutputView {
            address: addr.unwrap_or_default(),
            value_sats,
        };
        if script_matches_wallet(script, wallet_scripts) {
            wallet_outputs.push(view);
        } else if !view.address.is_empty() {
            external_outputs.push(view);
        }
    }

  // Filter wallet outputs to known addresses when we have the set
    wallet_outputs.retain(|o| wallet_addresses.contains(o.address.as_str()));

    rows_from_outputs(
        txid,
        height,
        blockheight,
        time,
        None,
        confirmations,
        is_spend,
        wallet_outputs,
        external_outputs,
    )
}

fn script_matches_wallet(script: &[u8], wallet_scripts: &HashSet<Vec<u8>>) -> bool {
    wallet_scripts.iter().any(|s| s.as_slice() == script)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer_api::IndexerTransactionSummary;

    fn addr_set(values: &[&str]) -> HashSet<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn self_send_shows_payment_send_and_receive_not_input_spend() {
        let receive = "VReceiveAddr1111111111111111111111";
        let change = "VChangeAddr11111111111111111111111";
        let wallet = addr_set(&[receive, change]);

        let detail = IndexerTransactionDetail {
            chain_id: None,
            txid: String::new(),
            found: true,
            trusted: true,
            transaction: Some(IndexerTransactionSummary {
                txid: "abc".repeat(32),
                block_height: Some(100),
                block_hash: None,
                tx_index: None,
                time: Some(1_000),
                is_coinbase: false,
                is_coinstake: false,
            }),
            inputs: vec![IndexerVin {
                n: 0,
                prev_txid: Some("prev".repeat(32)),
                prev_vout: Some(0),
                address: Some("VOldInputAddr111111111111111111111".into()),
                value: Some(IndexerAmount {
                    amount: "280.0".into(),
                    ticker: "VRM".into(),
                    decimal_places: Some(8),
                }),
                resolved: true,
            }],
            outputs: vec![
                IndexerVout {
                    n: 0,
                    address: Some(receive.into()),
                    value: Some(IndexerAmount {
                        amount: "1.0".into(),
                        ticker: "VRM".into(),
                        decimal_places: Some(8),
                    }),
                    is_spent: false,
                },
                IndexerVout {
                    n: 1,
                    address: Some(change.into()),
                    value: Some(IndexerAmount {
                        amount: "279.0".into(),
                        ticker: "VRM".into(),
                        decimal_places: Some(8),
                    }),
                    is_spent: false,
                },
            ],
            address_events: vec![],
        };

        let rows = rows_from_indexer_transaction(&detail, &wallet, Some(100));
        assert_eq!(rows.len(), 3);
        let sends: Vec<_> = rows
            .iter()
            .filter(|r| r.category == "send")
            .map(|r| r.amount)
            .collect();
        let receives: Vec<_> = rows
            .iter()
            .filter(|r| r.category == "receive")
            .map(|r| r.amount)
            .collect();
        assert_eq!(sends, vec![-1.0]);
        assert_eq!(receives, vec![1.0, 279.0]);
    }

    #[test]
    fn external_send_shows_payment_and_change_rows() {
        let change = "VChangeAddr11111111111111111111111";
        let external = "VExternalAddr111111111111111111111";
        let wallet = addr_set(&[change]);

        let detail = IndexerTransactionDetail {
            chain_id: None,
            txid: String::new(),
            found: true,
            trusted: true,
            transaction: Some(IndexerTransactionSummary {
                txid: "def".repeat(32),
                block_height: Some(200),
                block_hash: None,
                tx_index: None,
                time: Some(2_000),
                is_coinbase: false,
                is_coinstake: false,
            }),
            inputs: vec![IndexerVin {
                n: 0,
                prev_txid: Some("prev".repeat(32)),
                prev_vout: Some(0),
                address: Some("VOldInputAddr111111111111111111111".into()),
                value: Some(IndexerAmount {
                    amount: "280.0".into(),
                    ticker: "VRM".into(),
                    decimal_places: Some(8),
                }),
                resolved: true,
            }],
            outputs: vec![
                IndexerVout {
                    n: 0,
                    address: Some(external.into()),
                    value: Some(IndexerAmount {
                        amount: "1.0".into(),
                        ticker: "VRM".into(),
                        decimal_places: Some(8),
                    }),
                    is_spent: false,
                },
                IndexerVout {
                    n: 1,
                    address: Some(change.into()),
                    value: Some(IndexerAmount {
                        amount: "279.0".into(),
                        ticker: "VRM".into(),
                        decimal_places: Some(8),
                    }),
                    is_spent: false,
                },
            ],
            address_events: vec![],
        };

        let rows = rows_from_indexer_transaction(&detail, &wallet, Some(200));
        let sends: Vec<_> = rows
            .iter()
            .filter(|r| r.category == "send")
            .map(|r| r.amount)
            .collect();
        let receives: Vec<_> = rows
            .iter()
            .filter(|r| r.category == "receive")
            .map(|r| r.amount)
            .collect();
        assert_eq!(sends, vec![-1.0]);
        assert_eq!(receives, vec![279.0]);
    }
}
