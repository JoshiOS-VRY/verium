//! Verium / Vericoin legacy transaction wire format and sighash.
//!
//! Both chains extend Bitcoin's transaction serialization with `nTime` (uint32)
//! immediately after `nVersion`. Sighash hashes the full serialized transaction
//! (`ss << txTmp << nHashType` in Verium Core), not the pre-segwit Bitcoin path.

use bitcoin::consensus::{Decodable, Encodable};
use bitcoin::script::Builder;
use bitcoin::secp256k1::{Message, Secp256k1, SecretKey};
use bitcoin::sighash::EcdsaSighashType;
use bitcoin::script::PushBytes;
use bitcoin::{PrivateKey, PublicKey, ScriptBuf, Sequence, TxIn, TxOut, Txid};
use sha2::{Digest, Sha256};

use crate::chain::types::Utxo;
use crate::error::{AppError, AppResult};

/// Mutable transaction in Vericonomy wire format (version + nTime + in/out + locktime).
#[derive(Clone)]
pub struct VeriumMutableTx {
    pub version: i32,
    pub n_time: u32,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
    pub lock_time: u32,
}

pub fn current_n_time() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32
}

pub fn parse_display_txid(txid_hex: &str) -> AppResult<Txid> {
    txid_hex
        .parse()
        .map_err(|e| AppError::other(format!("invalid txid {txid_hex}: {e}")))
}

fn consensus_err(context: &str, e: impl std::fmt::Display) -> AppError {
    AppError::other(format!("{context}: {e}"))
}

pub fn serialize_verium_tx(tx: &VeriumMutableTx) -> AppResult<Vec<u8>> {
    let mut v = Vec::new();
    tx.version
        .consensus_encode(&mut v)
        .map_err(|e| consensus_err("encode version", e))?;
    tx.n_time
        .consensus_encode(&mut v)
        .map_err(|e| consensus_err("encode nTime", e))?;
    tx.inputs
        .consensus_encode(&mut v)
        .map_err(|e| consensus_err("encode vin", e))?;
    tx.outputs
        .consensus_encode(&mut v)
        .map_err(|e| consensus_err("encode vout", e))?;
    tx.lock_time
        .consensus_encode(&mut v)
        .map_err(|e| consensus_err("encode locktime", e))?;
    Ok(v)
}

pub fn decode_verium_tx(bytes: &[u8]) -> AppResult<VeriumMutableTx> {
    let mut slice = bytes;
    Ok(VeriumMutableTx {
        version: Decodable::consensus_decode(&mut slice)
            .map_err(|e| consensus_err("decode version", e))?,
        n_time: Decodable::consensus_decode(&mut slice)
            .map_err(|e| consensus_err("decode nTime", e))?,
        inputs: Decodable::consensus_decode(&mut slice)
            .map_err(|e| consensus_err("decode vin", e))?,
        outputs: Decodable::consensus_decode(&mut slice)
            .map_err(|e| consensus_err("decode vout", e))?,
        lock_time: Decodable::consensus_decode(&mut slice)
            .map_err(|e| consensus_err("decode locktime", e))?,
    })
}

fn strip_code_separators(script: &[u8]) -> Vec<u8> {
    const OP_CODESEPARATOR: u8 = 0xab;
    script
        .iter()
        .copied()
        .filter(|b| *b != OP_CODESEPARATOR)
        .collect()
}

/// Verium/Vericoin legacy P2PKH sighash (`SignatureHash` in `interpreter.cpp`).
pub fn verium_signature_hash(
    tx: &VeriumMutableTx,
    input_index: usize,
    script_code: &[u8],
    hash_type: i32,
) -> AppResult<[u8; 32]> {
    if input_index >= tx.inputs.len() {
        return Err(AppError::other("signature hash input index out of range"));
    }

    let mut tmp = tx.clone();
    let script_code = strip_code_separators(script_code);
    let script_buf = ScriptBuf::from_bytes(script_code);

    for input in &mut tmp.inputs {
        input.script_sig = ScriptBuf::new();
    }
    tmp.inputs[input_index].script_sig = script_buf;

    let sighash_type = hash_type as u32;
    if (sighash_type & 0x1f) == 0x02 {
        // SIGHASH_NONE
        for (i, input) in tmp.inputs.iter_mut().enumerate() {
            if i != input_index {
                input.sequence = Sequence::ZERO;
            }
        }
    } else if (sighash_type & 0x1f) == 0x03 {
        // SIGHASH_SINGLE
        if input_index >= tmp.outputs.len() {
            return Ok([1u8; 32]);
        }
        tmp.outputs.truncate(input_index + 1);
        for i in 0..input_index {
            tmp.outputs[i] = TxOut::NULL;
        }
        for (i, input) in tmp.inputs.iter_mut().enumerate() {
            if i != input_index {
                input.sequence = Sequence::ZERO;
            }
        }
    }

    if sighash_type & 0x80 != 0 {
        // SIGHASH_ANYONECANPAY
        let signed = tmp.inputs[input_index].clone();
        tmp.inputs = vec![signed];
    }

    let mut data = serialize_verium_tx(&tmp)?;
    data.extend_from_slice(&hash_type.to_le_bytes());
    Ok(double_sha256(&data))
}

pub fn double_sha256(data: &[u8]) -> [u8; 32] {
    let h1 = Sha256::digest(data);
    let h2 = Sha256::digest(h1);
    let mut out = [0u8; 32];
    out.copy_from_slice(&h2);
    out
}

pub fn display_txid_from_raw(raw: &[u8]) -> String {
    let mut bytes = double_sha256(raw);
    bytes.reverse();
    hex::encode(bytes)
}

pub fn sign_script_sig(
    secret_bytes: &[u8; 32],
    sighash: [u8; 32],
) -> AppResult<ScriptBuf> {
    let secp = Secp256k1::new();
    let msg = Message::from_digest_slice(&sighash)
        .map_err(|e| AppError::other(format!("message: {e}")))?;
    let sk = SecretKey::from_slice(secret_bytes)
        .map_err(|e| AppError::other(format!("secret key: {e}")))?;
    let sig = secp.sign_ecdsa(&msg, &sk);
    let mut sig_bytes = sig.serialize_der().to_vec();
    sig_bytes.push(EcdsaSighashType::All as u8);

    let pk = PublicKey::from_private_key(
        &secp,
        &PrivateKey::new(sk, bitcoin::NetworkKind::Main),
    );
    let sig_push = <&PushBytes>::try_from(sig_bytes.as_slice())
        .map_err(|_| AppError::other("signature too long for script push"))?;
    let pk_bytes = pk.to_bytes();
    let pk_push = <&PushBytes>::try_from(pk_bytes.as_slice())
        .map_err(|_| AppError::other("pubkey too long for script push"))?;
    Ok(Builder::new()
        .push_slice(sig_push)
        .push_slice(pk_push)
        .into_script())
}

pub fn build_signed_tx_hex(
    tx: &mut VeriumMutableTx,
    utxos: &[Utxo],
    secrets: &[[u8; 32]],
) -> AppResult<String> {
    if secrets.len() != tx.inputs.len() || utxos.len() != tx.inputs.len() {
        return Err(AppError::other("signing input count mismatch"));
    }
    for (i, secret) in secrets.iter().enumerate() {
        let script = hex::decode(&utxos[i].script_hex)
            .map_err(|e| AppError::other(format!("script: {e}")))?;
        let sighash = verium_signature_hash(tx, i, &script, EcdsaSighashType::All as i32)?;
        tx.inputs[i].script_sig = sign_script_sig(secret, sighash)?;
    }
    Ok(hex::encode(serialize_verium_tx(tx)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::absolute::LockTime;
    use bitcoin::transaction::Version;
    use bitcoin::OutPoint;

    #[test]
    fn wire_format_includes_n_time_after_version() {
        let tx = VeriumMutableTx {
            version: 1,
            n_time: 1_700_000_000,
            inputs: vec![TxIn {
                previous_output: OutPoint::null(),
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: bitcoin::Witness::new(),
            }],
            outputs: vec![TxOut::NULL],
            lock_time: 0,
        };
        let raw = serialize_verium_tx(&tx).unwrap();
        assert_eq!(&raw[0..4], &[1, 0, 0, 0]);
        assert_eq!(u32::from_le_bytes(raw[4..8].try_into().unwrap()), 1_700_000_000);
        assert_eq!(raw[8], 1); // one input
    }

    #[test]
    fn round_trip_decode() {
        let tx = VeriumMutableTx {
            version: Version::ONE.0,
            n_time: 42,
            inputs: vec![],
            outputs: vec![],
            lock_time: LockTime::ZERO.to_consensus_u32(),
        };
        let raw = serialize_verium_tx(&tx).unwrap();
        let decoded = decode_verium_tx(&raw).unwrap();
        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.n_time, 42);
    }
}
