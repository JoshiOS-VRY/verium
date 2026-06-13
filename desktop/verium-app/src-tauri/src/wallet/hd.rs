//! HD derivation and P2PKH address encoding for light wallet.
//!
//! Full-node Verium/Vericoin wallets use Bitcoin Core-style paths (`m/0'/0'/n'` receive,
//! `m/0'/1'/n'` change). BIP39 light wallets created in-app use BIP44 (`m/44'/coin'/0'/0/n`).

use std::str::FromStr;

use bip39::{Language, Mnemonic};
use bitcoin::bip32::{ChainCode, ChildNumber, DerivationPath, Fingerprint, Xpriv};
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{PrivateKey, PublicKey, PubkeyHash, ScriptBuf};
use sha2::{Digest, Sha256};
use ripemd::Ripemd160;

use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};
use crate::hardware_wallet::coin_type_for;
use crate::recovery::secret_bytes_to_wif;

const COIN_SATS: f64 = 100_000_000.0;
const BIP32_EXTKEY_SIZE: usize = 74;
const EXT_SECRET_PREFIX_LEN: usize = 4;

/// External (receive) or internal (change) chain — matches `wallet.cpp` HD layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdChain {
    External,
    Internal,
}

impl HdChain {
    fn hardened_index(self) -> u32 {
        match self {
            HdChain::External => 0,
            HdChain::Internal => 1,
        }
    }
}

/// Full-node HD master keys use Core-style paths; BIP39 mnemonics use BIP44.
pub fn uses_core_hd_paths(coin: CoinId, seed_secret: &str) -> bool {
    is_hd_master_secret(coin, seed_secret)
}

pub fn pubkey_address_version(coin: CoinId) -> u8 {
    match coin {
        CoinId::Verium => 70,
        CoinId::Vericoin => 70,
    }
}

fn base58check_encode(version: u8, payload: &[u8]) -> String {
    let mut data = Vec::with_capacity(1 + payload.len() + 4);
    data.push(version);
    data.extend_from_slice(payload);
    let hash1 = Sha256::digest(&data);
    let hash2 = Sha256::digest(hash1);
    data.extend_from_slice(&hash2[..4]);
    bs58::encode(data).into_string()
}

fn hash160(data: &[u8]) -> [u8; 20] {
    let sha = Sha256::digest(data);
    let rip = Ripemd160::digest(sha);
    let mut out = [0u8; 20];
    out.copy_from_slice(&rip);
    out
}

pub fn pubkey_to_p2pkh_address(coin: CoinId, pubkey: &[u8]) -> AppResult<String> {
    let pk_hash = hash160(pubkey);
    Ok(base58check_encode(pubkey_address_version(coin), &pk_hash))
}

/// Decode a standard P2PKH `scriptPubKey` (`OP_DUP OP_HASH160 … OP_EQUALVERIFY OP_CHECKSIG`).
pub fn p2pkh_script_to_address(coin: CoinId, script: &[u8]) -> Option<String> {
    if script.len() != 25
        || script[0] != 0x76
        || script[1] != 0xa9
        || script[2] != 0x14
        || script[23] != 0x88
        || script[24] != 0xac
    {
        return None;
    }
    Some(base58check_encode(pubkey_address_version(coin), &script[3..23]))
}

/// Known Vericonomy extended-secret Base58 prefixes (mainnet + test variants).
fn ext_secret_prefixes(coin: CoinId) -> &'static [[u8; EXT_SECRET_PREFIX_LEN]] {
    match coin {
        CoinId::Verium => &[
            [0xE3, 0xCC, 0xAE, 0x01], // mainnet
            [0x04, 0x35, 0x83, 0x94], // legacy test/regtest
            [0xDA, 0xCE, 0xCE, 0x01], // binarytest VRM
        ],
        CoinId::Vericoin => &[
            [0xE3, 0xCC, 0xAE, 0x01], // mainnet
            [0xDA, 0xCE, 0xAE, 0x01], // binarytest VRC
        ],
    }
}

pub fn normalize_hd_master_secret(secret: &str) -> String {
    secret
        .trim()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

fn is_numbered_prefix_token(token: &str) -> bool {
    let t = token.trim_end_matches('.');
    !t.is_empty() && t.chars().all(|c| c.is_ascii_digit())
}

/// Collapse whitespace for BIP39 phrases; strips numbered-list prefixes from copy/paste.
pub fn normalize_mnemonic_phrase(phrase: &str) -> String {
    phrase
        .split_whitespace()
        .filter(|w| !is_numbered_prefix_token(w))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Normalize user-provided seed material: phrases keep spaced words; HD keys strip whitespace.
pub fn normalize_light_wallet_seed(secret: &str) -> String {
    let trimmed = secret.trim();
    if trimmed.starts_with("xprv") {
        return normalize_hd_master_secret(trimmed);
    }
    if trimmed.contains(char::is_whitespace) {
        normalize_mnemonic_phrase(trimmed)
    } else {
        normalize_hd_master_secret(trimmed)
    }
}

/// True when `secret` is a BIP32 master key (Bitcoin xprv or Vericonomy export).
pub fn is_hd_master_secret(coin: CoinId, secret: &str) -> bool {
    let trimmed = normalize_hd_master_secret(secret);
    if trimmed.starts_with("xprv") {
        return true;
    }
    if trimmed.contains(' ') {
        return false;
    }
    let Ok(data) = bs58::decode(&trimmed).with_check(None).into_vec() else {
        return false;
    };
    if data.len() != EXT_SECRET_PREFIX_LEN + BIP32_EXTKEY_SIZE {
        return false;
    }
    let prefix: [u8; EXT_SECRET_PREFIX_LEN] = data[0..EXT_SECRET_PREFIX_LEN]
        .try_into()
        .unwrap_or([0; 4]);
    ext_secret_prefixes(coin).contains(&prefix)
}

fn bip32_payload_to_xpriv(payload: &[u8]) -> AppResult<Xpriv> {
    if payload.len() != BIP32_EXTKEY_SIZE {
        return Err(AppError::other("extended key payload has wrong length"));
    }
    if payload[41] != 0 {
        return Err(AppError::other("extended private key has invalid padding byte"));
    }
    let depth = payload[0];
    let parent_fingerprint: [u8; 4] = payload[1..5]
        .try_into()
        .map_err(|_| AppError::other("extended key fingerprint missing"))?;
    let parent_fingerprint = Fingerprint::from(parent_fingerprint);
    let child_number = u32::from_be_bytes([payload[5], payload[6], payload[7], payload[8]]);
    let child_number = ChildNumber::from(child_number);
    let chain_code_bytes: [u8; 32] = payload[9..41]
        .try_into()
        .map_err(|_| AppError::other("extended key chain code missing"))?;
    let chain_code = ChainCode::from(chain_code_bytes);
    let private_key = bitcoin::secp256k1::SecretKey::from_slice(&payload[42..74])
        .map_err(|e| AppError::other(format!("extended key secret invalid: {e}")))?;
    Ok(Xpriv {
        network: bitcoin::NetworkKind::Main,
        depth,
        parent_fingerprint,
        child_number,
        chain_code,
        private_key,
    })
}

fn parse_vericonomy_ext_key(coin: CoinId, secret: &str) -> AppResult<Xpriv> {
    let trimmed = normalize_hd_master_secret(secret);
    let data = bs58::decode(&trimmed)
        .with_check(None)
        .into_vec()
        .map_err(|e| AppError::other(format!("invalid extended key encoding: {e}")))?;
    if data.len() != EXT_SECRET_PREFIX_LEN + BIP32_EXTKEY_SIZE {
        return Err(AppError::other(
            "extended key length is wrong — paste the full master key from Security → Export",
        ));
    }
    let prefix: [u8; EXT_SECRET_PREFIX_LEN] = data[0..EXT_SECRET_PREFIX_LEN]
        .try_into()
        .map_err(|_| AppError::other("extended key prefix missing"))?;
    if !ext_secret_prefixes(coin).contains(&prefix) {
        return Err(AppError::other(
            "extended key prefix does not match this chain — export from the same coin (VRM/VRC) in full-node mode",
        ));
    }
    bip32_payload_to_xpriv(&data[EXT_SECRET_PREFIX_LEN..])
}

pub fn parse_root_xpriv(coin: CoinId, secret: &str) -> AppResult<Xpriv> {
    let trimmed = normalize_hd_master_secret(secret);
    if trimmed.starts_with("xprv") {
        return Xpriv::from_str(&trimmed)
            .map_err(|e| AppError::other(format!("invalid xprv: {e}")));
    }
    parse_vericonomy_ext_key(coin, &trimmed)
}

#[deprecated(note = "use is_hd_master_secret(coin, secret)")]
pub fn is_xprv_secret(secret: &str) -> bool {
    secret.trim().starts_with("xprv")
}

pub fn derive_address_at(
    coin: CoinId,
    seed_secret: &str,
    bip39_passphrase: Option<&str>,
    index: u32,
) -> AppResult<String> {
    derive_address_on_chain(coin, seed_secret, bip39_passphrase, HdChain::External, index)
}

pub fn derive_change_address_at(
    coin: CoinId,
    seed_secret: &str,
    bip39_passphrase: Option<&str>,
    index: u32,
) -> AppResult<String> {
    derive_address_on_chain(coin, seed_secret, bip39_passphrase, HdChain::Internal, index)
}

pub fn derive_address_on_chain(
    coin: CoinId,
    seed_secret: &str,
    bip39_passphrase: Option<&str>,
    chain: HdChain,
    index: u32,
) -> AppResult<String> {
    let (_, pubkey) = derive_keypair_on_chain(coin, seed_secret, bip39_passphrase, chain, index)?;
    pubkey_to_p2pkh_address(coin, &pubkey)
}

pub fn derive_keypair_at(
    coin: CoinId,
    seed_secret: &str,
    bip39_passphrase: Option<&str>,
    index: u32,
) -> AppResult<([u8; 32], Vec<u8>)> {
    derive_keypair_on_chain(coin, seed_secret, bip39_passphrase, HdChain::External, index)
}

fn parse_root_xpriv_from_seed(
    coin: CoinId,
    seed_secret: &str,
    bip39_passphrase: Option<&str>,
) -> AppResult<Xpriv> {
    if is_hd_master_secret(coin, seed_secret) {
        return parse_root_xpriv(coin, seed_secret);
    }
    let mnemonic = Mnemonic::parse_in(Language::English, seed_secret.trim())
        .map_err(|e| AppError::other(format!("invalid mnemonic: {e}")))?;
    let seed = mnemonic.to_seed(bip39_passphrase.unwrap_or(""));
    Xpriv::new_master(bitcoin::NetworkKind::Main, &seed)
        .map_err(|e| AppError::other(format!("master key derivation failed: {e}")))
}

fn derivation_path_for(
    coin: CoinId,
    seed_secret: &str,
    chain: HdChain,
    index: u32,
) -> AppResult<DerivationPath> {
    if uses_core_hd_paths(coin, seed_secret) {
        // Match veriumd wallet.cpp: m/0'/0'/k' and m/0'/1'/k' (address index is hardened).
        let account = ChildNumber::from_hardened_idx(0)
            .map_err(|e| AppError::other(format!("invalid account index: {e}")))?;
        let chain_idx = ChildNumber::from_hardened_idx(chain.hardened_index())
            .map_err(|e| AppError::other(format!("invalid chain index: {e}")))?;
        let addr = ChildNumber::from_hardened_idx(index)
            .map_err(|e| AppError::other(format!("invalid address index: {e}")))?;
        return Ok(DerivationPath::from(vec![account, chain_idx, addr]));
    }
    if chain != HdChain::External {
        return Err(AppError::other("BIP44 mnemonics only support external receive chain"));
    }
    let coin_type = coin_type_for(coin);
    format!("m/44'/{coin_type}'/0'/0/{index}")
        .parse()
        .map_err(|e| AppError::other(format!("invalid derivation path: {e}")))
}

pub fn derive_keypair_on_chain(
    coin: CoinId,
    seed_secret: &str,
    bip39_passphrase: Option<&str>,
    chain: HdChain,
    index: u32,
) -> AppResult<([u8; 32], Vec<u8>)> {
    let secp = Secp256k1::new();
    let xpriv = parse_root_xpriv_from_seed(coin, seed_secret, bip39_passphrase)?;
    let path = derivation_path_for(coin, seed_secret, chain, index)?;
    let child = xpriv
        .derive_priv(&secp, &path)
        .map_err(|e| AppError::other(format!("derive failed: {e}")))?;
    let secret = child.private_key.secret_bytes();
    let privkey = PrivateKey::new(child.private_key, bitcoin::NetworkKind::Main);
    let pubkey = PublicKey::from_private_key(&secp, &privkey);
    Ok((secret, pubkey.to_bytes()))
}

pub fn derive_wif_at(
    coin: CoinId,
    mnemonic: &str,
    bip39_passphrase: Option<&str>,
    index: u32,
) -> AppResult<String> {
    let (secret, _) = derive_keypair_at(coin, mnemonic, bip39_passphrase, index)?;
    Ok(secret_bytes_to_wif(coin, &secret))
}

pub fn address_to_script_pubkey(coin: CoinId, address: &str) -> AppResult<Vec<u8>> {
    let decoded = bs58::decode(address)
        .with_check(Some(pubkey_address_version(coin)))
        .into_vec()
        .map_err(|e| AppError::other(format!("invalid address: {e}")))?;
    if decoded.len() != 21 {
        return Err(AppError::other("address payload wrong length"));
    }
    let pk_hash = &decoded[1..];
    let hash = PubkeyHash::from_slice(pk_hash)
        .map_err(|e| AppError::other(format!("pubkey hash: {e}")))?;
    let script = ScriptBuf::new_p2pkh(&hash);
    Ok(script.as_bytes().to_vec())
}

/// Result of one resumable gap-scan slice on a single HD chain.
pub struct ChainScanSlice {
    pub scripts: Vec<String>,
    pub next_index: u32,
    pub chain_complete: bool,
    pub budget_paused: bool,
}

async fn discover_chain_script_hexes(
    coin: CoinId,
    seed_secret: &str,
    bip39_passphrase: Option<&str>,
    chain: HdChain,
    gap_limit: u32,
    start_index: u32,
    backend: &dyn crate::chain::ChainBackend,
    funded_out: &mut Vec<String>,
    hook: Option<&dyn crate::wallet::gap_scan_hook::GapScanHook>,
    utxo_refresh_on_funded: bool,
) -> AppResult<ChainScanSlice> {
    let batch_size = crate::chain::electrum::indexing::SCRIPTS_PER_BATCH;
    let mut scripts = Vec::new();
    let mut consecutive_empty = 0u32;
    let mut index = start_index;
    while consecutive_empty < gap_limit && index <= 500 {
        let batch_end = (index + batch_size).min(501);
        let mut batch_scripts = Vec::new();
        for idx in index..batch_end {
            let addr = derive_address_on_chain(coin, seed_secret, bip39_passphrase, chain, idx)?;
            let script = address_to_script_pubkey(coin, &addr)?;
            batch_scripts.push(hex::encode(&script));
        }
        let balances = match backend.get_balances_per_script(&batch_scripts).await {
            Ok(b) => b,
            Err(e) if e.is_indexing_budget_exhausted() => {
                return Ok(ChainScanSlice {
                    scripts,
                    next_index: index,
                    chain_complete: false,
                    budget_paused: true,
                });
            }
            Err(e) => return Err(e),
        };
        let mut batch_had_funded = false;
        for (script_hex, bal) in batch_scripts.into_iter().zip(balances) {
            if bal.total_sats() == 0 {
                consecutive_empty += 1;
            } else {
                consecutive_empty = 0;
                if !funded_out.iter().any(|s| s == &script_hex) {
                    funded_out.push(script_hex.clone());
                    batch_had_funded = true;
                }
            }
            scripts.push(script_hex);
            if consecutive_empty >= gap_limit {
                break;
            }
        }
        if batch_had_funded {
            if let Some(h) = hook {
                h.on_funded_batch(funded_out.as_slice(), utxo_refresh_on_funded)
                    .await?;
            }
        }
        index = batch_end;
        if consecutive_empty >= gap_limit {
            break;
        }
    }
    let chain_complete = consecutive_empty >= gap_limit || index > 500;
    Ok(ChainScanSlice {
        scripts,
        next_index: index,
        chain_complete,
        budget_paused: false,
    })
}

/// Pre-derive watch-only scripts locally (no Electrum) so balance can load immediately.
pub fn precache_light_wallet_scripts(coin: CoinId, seed_secret: &str) -> AppResult<()> {
    let mut scripts = Vec::new();
    let chains: &[HdChain] = if uses_core_hd_paths(coin, seed_secret) {
        &[HdChain::External, HdChain::Internal]
    } else {
        &[HdChain::External]
    };
    for &chain in chains {
        for index in 0..=GAP_PRECACHE_MAX {
            let addr = derive_address_on_chain(coin, seed_secret, None, chain, index)?;
            let script = address_to_script_pubkey(coin, &addr)?;
            scripts.push(hex::encode(&script));
        }
    }
    crate::wallet::keystore::set_cached_script_hexes(coin, &scripts)
}

/// Local pre-derivation before Electrum gap scan finishes (Core HD: external + internal).
pub const GAP_PRECACHE_MAX: u32 = 80;
/// Upper bound for HD index search (gap scan stops at index 500).
pub const GAP_SCAN_MAX_INDEX: u32 = 501;

/// Expected script count from precache alone (no gap scan yet).
/// Map script pubkeys to P2PKH addresses by scanning the HD wallet up to [`GAP_SCAN_MAX_INDEX`].
pub fn resolve_addresses_for_script_hexes(
    coin: CoinId,
    seed_secret: &str,
    bip39_passphrase: Option<&str>,
    script_hexes: &[&str],
) -> AppResult<std::collections::HashMap<String, String>> {
    use std::collections::{HashMap, HashSet};

    let targets: HashSet<&str> = script_hexes.iter().copied().collect();
    let mut map = HashMap::new();
    if targets.is_empty() {
        return Ok(map);
    }

    if uses_core_hd_paths(coin, seed_secret) {
        for chain in [HdChain::External, HdChain::Internal] {
            for index in 0..GAP_SCAN_MAX_INDEX {
                let addr =
                    derive_address_on_chain(coin, seed_secret, bip39_passphrase, chain, index)?;
                let script_hex = hex::encode(address_to_script_pubkey(coin, &addr)?);
                if targets.contains(script_hex.as_str()) {
                    map.insert(script_hex, addr);
                    if map.len() == targets.len() {
                        return Ok(map);
                    }
                }
            }
        }
    } else {
        for index in 0..GAP_SCAN_MAX_INDEX {
            let addr = derive_address_at(coin, seed_secret, bip39_passphrase, index)?;
            let script_hex = hex::encode(address_to_script_pubkey(coin, &addr)?);
            if targets.contains(script_hex.as_str()) {
                map.insert(script_hex, addr);
                if map.len() == targets.len() {
                    return Ok(map);
                }
            }
        }
    }
    Ok(map)
}

/// Fill empty `Utxo::address` fields from `script_hex` (SQLite cache does not store addresses).
pub fn enrich_utxo_addresses(
    coin: CoinId,
    seed_secret: &str,
    bip39_passphrase: Option<&str>,
    utxos: &mut [crate::chain::types::Utxo],
) -> AppResult<()> {
    let scripts: Vec<&str> = utxos
        .iter()
        .filter(|u| u.address.is_empty() && !u.script_hex.is_empty())
        .map(|u| u.script_hex.as_str())
        .collect();
    let map = resolve_addresses_for_script_hexes(coin, seed_secret, bip39_passphrase, &scripts)?;
    for utxo in utxos {
        if utxo.address.is_empty() {
            if let Some(addr) = map.get(&utxo.script_hex) {
                utxo.address = addr.clone();
            }
        }
    }
    Ok(())
}

pub fn precache_script_count(coin: CoinId, seed_secret: &str) -> u32 {
    let chains: u32 = if uses_core_hd_paths(coin, seed_secret) {
        2
    } else {
        1
    };
    (GAP_PRECACHE_MAX + 1) * chains
}

/// Resumable gap scan across external (+ internal for Core HD) chains.
pub async fn discover_script_hexes(
    coin: CoinId,
    seed_secret: &str,
    bip39_passphrase: Option<&str>,
    gap_limit: u32,
    progress: &mut crate::wallet::keystore::IndexingProgress,
    backend: &dyn crate::chain::ChainBackend,
    funded_out: &mut Vec<String>,
    hook: Option<&dyn crate::wallet::gap_scan_hook::GapScanHook>,
    utxo_refresh_on_funded: bool,
) -> AppResult<(Vec<String>, bool)> {
    let mut merged = Vec::new();

    if !progress.gap_external_done {
        let external = discover_chain_script_hexes(
            coin,
            seed_secret,
            bip39_passphrase,
            HdChain::External,
            gap_limit,
            progress.gap_external,
            backend,
            funded_out,
            hook,
            utxo_refresh_on_funded,
        )
        .await?;
        merged.extend(external.scripts);
        progress.gap_external = external.next_index;
        if external.budget_paused {
            return Ok((merged, false));
        }
        if !external.chain_complete {
            return Ok((merged, false));
        }
        progress.gap_external_done = true;
    }

    if !uses_core_hd_paths(coin, seed_secret) {
        return Ok((merged, true));
    }

    if !progress.gap_internal_done {
        let internal = discover_chain_script_hexes(
            coin,
            seed_secret,
            bip39_passphrase,
            HdChain::Internal,
            gap_limit,
            progress.gap_internal,
            backend,
            funded_out,
            hook,
            utxo_refresh_on_funded,
        )
        .await?;
        for script in internal.scripts {
            if !merged.contains(&script) {
                merged.push(script);
            }
        }
        progress.gap_internal = internal.next_index;
        if internal.budget_paused {
            return Ok((merged, false));
        }
        if !internal.chain_complete {
            return Ok((merged, false));
        }
        progress.gap_internal_done = true;
    }
    Ok((merged, true))
}

pub fn sats_to_coins(sats: i64) -> f64 {
    sats as f64 / COIN_SATS
}

pub fn coins_to_sats(coins: f64) -> i64 {
    (coins * COIN_SATS).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recovery::secret_key_prefix;

    #[test]
    fn wif_prefix_matches_recovery() {
        let coin = CoinId::Verium;
        let secret = [1u8; 32];
        let wif = secret_bytes_to_wif(coin, &secret);
        let decoded = bs58::decode(&wif).with_check(None).into_vec().unwrap();
        assert_eq!(decoded[0], secret_key_prefix(coin));
    }

    #[test]
    fn normalize_light_wallet_seed_preserves_mnemonic_words() {
        let phrase = "legal winner thank year wave sausage worth useful legal winner thank yellow";
        let normalized = normalize_light_wallet_seed(phrase);
        assert_eq!(normalized, phrase);
        assert!(crate::recovery::validate_mnemonic(&normalized).unwrap());
    }

    #[test]
    fn normalize_light_wallet_seed_strips_numbered_copy_prefixes() {
        let numbered = "1. legal\n2. winner\n3. thank";
        let normalized = normalize_light_wallet_seed(numbered);
        assert_eq!(normalized, "legal winner thank");
    }

    #[test]
    fn normalize_hd_master_secret_still_strips_xprv_whitespace() {
        let key = "  xprv9s21ZrQH143K   3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRN4VIcWnJYtb5AF2Q2AT9aMu  ";
        let normalized = normalize_light_wallet_seed(key);
        assert!(!normalized.contains(' '));
        assert!(normalized.starts_with("xprv"));
    }

    #[test]
    fn bip44_path_uses_slip44_coin_type_not_hardened_encoding() {
        let phrase = "legal winner thank year wave sausage worth useful legal winner thank yellow";
        let path = derivation_path_for(CoinId::Verium, phrase, HdChain::External, 0).unwrap();
        assert_eq!(format!("m/{path}"), "m/44'/462'/0'/0/0");
        let vrc = derivation_path_for(CoinId::Vericoin, phrase, HdChain::External, 5).unwrap();
        assert_eq!(format!("m/{vrc}"), "m/44'/463'/0'/0/5");
    }

    #[test]
    fn core_hd_paths_use_hardened_address_index() {
        let ext_key = "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRN4VIcWnJYtb5AF2Q2AT9aMu";
        assert!(uses_core_hd_paths(CoinId::Verium, ext_key));
        let path = derivation_path_for(CoinId::Verium, ext_key, HdChain::External, 0).unwrap();
        assert_eq!(format!("m/{path}"), "m/0'/0'/0'");
        let change = derivation_path_for(CoinId::Verium, ext_key, HdChain::Internal, 3).unwrap();
        assert_eq!(format!("m/{change}"), "m/0'/1'/3'");
    }
}
