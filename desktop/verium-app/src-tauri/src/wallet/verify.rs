//! Local transaction verification for light wallet (decode + output check).

use bitcoin::hashes::{hash160, Hash};
use bitcoin::secp256k1::{ecdsa::Signature, Message, Secp256k1};
use bitcoin::sighash::EcdsaSighashType;
use bitcoin::script::Instruction;
use bitcoin::{PublicKey, ScriptBuf};

use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};
use crate::chain::types::Utxo;
use crate::wallet::hd;
use crate::wallet::vericonomy_tx::{
    decode_verium_tx, parse_display_txid, verium_signature_hash, VeriumMutableTx,
};

#[derive(Debug, Clone)]
pub struct TxVerifyResult {
    pub ok: bool,
    pub detail: String,
}

pub fn verify_raw_tx_hex(raw_hex: &str) -> AppResult<TxVerifyResult> {
    let bytes = hex::decode(raw_hex.trim())
        .map_err(|e| AppError::other(format!("tx hex decode: {e}")))?;
    let tx = decode_verium_tx(&bytes).map_err(|e| AppError::other(format!("tx decode: {e}")))?;
    if tx.inputs.is_empty() || tx.outputs.is_empty() {
        return Ok(TxVerifyResult {
            ok: false,
            detail: "transaction has no inputs or outputs".into(),
        });
    }
    Ok(TxVerifyResult {
        ok: true,
        detail: format!("{} inputs, {} outputs", tx.inputs.len(), tx.outputs.len()),
    })
}

pub fn script_pays_to(script_hex: &str, expected_script_hex: &str) -> bool {
    script_hex.trim().eq_ignore_ascii_case(expected_script_hex.trim())
}

/// Verify signed transaction outputs match user-approved destinations and amounts.
pub fn verify_send_outputs(
    coin: CoinId,
    raw_hex: &str,
    expected: &[(String, i64)],
) -> AppResult<()> {
    let bytes = hex::decode(raw_hex.trim())
        .map_err(|e| AppError::other(format!("tx hex decode: {e}")))?;
    let tx = decode_verium_tx(&bytes).map_err(|e| AppError::other(format!("tx decode: {e}")))?;
    if tx.inputs.is_empty() || tx.outputs.is_empty() {
        return Err(AppError::other(
            "transaction has no inputs or outputs",
        ));
    }
    for (addr, amount_sats) in expected {
        if *amount_sats <= 0 {
            return Err(AppError::other(format!("invalid amount for {addr}")));
        }
        let script = hd::address_to_script_pubkey(coin, addr)?;
        let amount_u64 = u64::try_from(*amount_sats)
            .map_err(|_| AppError::other(format!("amount overflow for {addr}")))?;
        let matched = tx.outputs.iter().any(|out| {
            out.script_pubkey.as_bytes() == script.as_slice()
                && out.value.to_sat() == amount_u64
        });
        if !matched {
            return Err(AppError::other(format!(
                "signed transaction does not pay {amount_sats} sats to {addr}"
            )));
        }
    }
    Ok(())
}

fn p2pkh_pubkey_hash(script_pubkey: &[u8]) -> Option<&[u8]> {
    if script_pubkey.len() == 25
        && script_pubkey[0] == 0x76
        && script_pubkey[1] == 0xa9
        && script_pubkey[2] == 0x14
        && script_pubkey[23] == 0x88
        && script_pubkey[24] == 0xac
    {
        return Some(&script_pubkey[3..23]);
    }
    None
}

fn parse_p2pkh_script_sig(script_sig: &ScriptBuf) -> AppResult<(Vec<u8>, Vec<u8>)> {
    let mut pushes: Vec<Vec<u8>> = Vec::new();
    for result in script_sig.instructions_minimal() {
        let instruction = result.map_err(|e| AppError::other(format!("scriptSig parse: {e}")))?;
        if let Instruction::PushBytes(bytes) = instruction {
            pushes.push(bytes.as_bytes().to_vec());
        }
    }
    if pushes.len() != 2 {
        return Err(AppError::other(format!(
            "expected P2PKH scriptSig with 2 pushes, got {}",
            pushes.len()
        )));
    }
    Ok((pushes[0].clone(), pushes[1].clone()))
}

/// Re-verify each P2PKH input signature locally before broadcasting.
pub fn verify_signed_p2pkh_inputs(raw_hex: &str, utxos: &[Utxo]) -> AppResult<()> {
    let bytes = hex::decode(raw_hex.trim())
        .map_err(|e| AppError::other(format!("tx hex decode: {e}")))?;
    let tx = decode_verium_tx(&bytes).map_err(|e| AppError::other(format!("tx decode: {e}")))?;
    if tx.inputs.len() != utxos.len() {
        return Err(AppError::other("signed input count does not match utxos"));
    }

    let secp = Secp256k1::new();
    for (i, utxo) in utxos.iter().enumerate() {
        let expected_txid = parse_display_txid(&utxo.txid)?;
        if tx.inputs[i].previous_output.txid != expected_txid {
            return Err(AppError::other(format!(
                "input {i} references wrong parent txid (expected {}, got {})",
                utxo.txid,
                tx.inputs[i].previous_output.txid
            )));
        }
        if tx.inputs[i].previous_output.vout != utxo.vout {
            return Err(AppError::other(format!(
                "input {i} references wrong vout (expected {}, got {})",
                utxo.vout,
                tx.inputs[i].previous_output.vout
            )));
        }

        let script_pubkey = hex::decode(utxo.script_hex.trim())
            .map_err(|e| AppError::other(format!("utxo script hex: {e}")))?;
        let pubkey_hash = p2pkh_pubkey_hash(&script_pubkey).ok_or_else(|| {
            AppError::other(format!(
                "input {} spends non-P2PKH script (only standard P2PKH sends are supported)",
                i
            ))
        })?;

        let (sig_bytes, pk_bytes) = parse_p2pkh_script_sig(&tx.inputs[i].script_sig)?;
        if sig_bytes.is_empty() {
            return Err(AppError::other(format!("input {i} has empty signature")));
        }
        let sighash_type = sig_bytes[sig_bytes.len() - 1];
        if sighash_type != EcdsaSighashType::All as u8 {
            return Err(AppError::other(format!(
                "input {i} uses unsupported sighash type {sighash_type}"
            )));
        }
        let der_sig = &sig_bytes[..sig_bytes.len() - 1];

        let pk = PublicKey::from_slice(&pk_bytes)
            .map_err(|e| AppError::other(format!("input {i} pubkey invalid: {e}")))?;
        let derived_hash = hash160::Hash::hash(&pk_bytes);
        if derived_hash.as_byte_array() != pubkey_hash {
            return Err(AppError::other(format!(
                "input {i} pubkey does not match output script (wrong signing key)"
            )));
        }

        let sighash = verium_signature_hash(
            &tx,
            i,
            &script_pubkey,
            EcdsaSighashType::All as i32,
        )?;
        let msg = Message::from_digest_slice(&sighash)
            .map_err(|e| AppError::other(format!("sighash message: {e}")))?;
        let sig = Signature::from_der(der_sig)
            .map_err(|e| AppError::other(format!("input {i} DER signature invalid: {e}")))?;
        secp.verify_ecdsa(&msg, &sig, &pk.inner)
            .map_err(|e| AppError::other(format!("input {i} signature invalid: {e}")))?;
    }
    Ok(())
}

/// Confirm each UTXO's cached script matches the parent transaction on the network.
pub fn utxo_script_matches_parent_tx(utxo: &Utxo, parent: &VeriumMutableTx) -> AppResult<()> {
    let out = parent
        .outputs
        .get(utxo.vout as usize)
        .ok_or_else(|| AppError::other(format!("parent tx missing vout {}", utxo.vout)))?;
    let on_chain = hex::encode(out.script_pubkey.as_bytes());
    if !script_pays_to(&on_chain, &utxo.script_hex) {
        return Err(AppError::other(format!(
            "UTXO {}:{} on-chain script does not match wallet script",
            utxo.txid, utxo.vout
        )));
    }
    Ok(())
}
