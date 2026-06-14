//! Verium / Vericoin legacy transaction wire format and sighash (SDK wrapper).

use bitcoin::{ScriptBuf, Txid};

pub use vericonomy_tx::{SignInput, VeriumMutableTx};

use crate::chain::types::Utxo;
use crate::error::AppResult;
use crate::sdk_bridge::map_wallet_err;

pub fn current_n_time() -> u32 {
    vericonomy_tx::current_n_time()
}

pub fn parse_display_txid(txid_hex: &str) -> AppResult<Txid> {
    map_wallet_err(vericonomy_tx::parse_display_txid(txid_hex))
}

pub fn wire_txid_from_raw(raw: &[u8]) -> Txid {
    vericonomy_tx::wire_txid_from_raw(raw)
}

pub fn reverse_display_txid_hex(txid_hex: &str) -> AppResult<String> {
    map_wallet_err(vericonomy_tx::reverse_display_txid_hex(txid_hex))
}

pub fn serialize_verium_tx(tx: &VeriumMutableTx) -> AppResult<Vec<u8>> {
    map_wallet_err(vericonomy_tx::serialize_verium_tx(tx))
}

pub fn decode_verium_tx(bytes: &[u8]) -> AppResult<VeriumMutableTx> {
    map_wallet_err(vericonomy_tx::decode_verium_tx(bytes))
}

pub fn verium_signature_hash(
    tx: &VeriumMutableTx,
    input_index: usize,
    script_code: &[u8],
    hash_type: i32,
) -> AppResult<[u8; 32]> {
    map_wallet_err(vericonomy_tx::verium_signature_hash(
        tx,
        input_index,
        script_code,
        hash_type,
    ))
}

pub fn double_sha256(data: &[u8]) -> [u8; 32] {
    vericonomy_tx::double_sha256(data)
}

pub fn display_txid_from_raw(raw: &[u8]) -> String {
    vericonomy_tx::display_txid_from_raw(raw)
}

pub fn sign_script_sig(secret_bytes: &[u8; 32], sighash: [u8; 32]) -> AppResult<ScriptBuf> {
    map_wallet_err(vericonomy_tx::sign_script_sig(secret_bytes, sighash))
}

pub fn build_signed_tx_hex(
    tx: &mut VeriumMutableTx,
    utxos: &[Utxo],
    secrets: &[[u8; 32]],
) -> AppResult<String> {
    let inputs: Vec<SignInput> = utxos
        .iter()
        .map(|u| SignInput {
            script_hex: u.script_hex.clone(),
        })
        .collect();
    map_wallet_err(vericonomy_tx::build_signed_tx_hex(tx, &inputs, secrets))
}
