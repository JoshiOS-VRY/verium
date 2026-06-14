//! HD derivation and P2PKH address encoding for light wallet.
//!
//! Crypto delegates to `vericonomy_hd`; gap scan / keystore helpers stay app-local.

pub use vericonomy_hd::{
    coins_to_sats, is_hd_master_secret, is_xprv_secret, normalize_hd_master_secret,
    normalize_light_wallet_seed, normalize_mnemonic_phrase, p2pkh_script_to_address,
    pubkey_address_version, sats_to_coins, uses_core_hd_paths, GAP_SCAN_MAX_INDEX, HdChain,
};

use crate::coin_profile::CoinId;
use crate::error::AppResult;
use crate::sdk_bridge::map_wallet_err;

pub fn coin_type_for(coin: CoinId) -> u32 {
    vericonomy_hd::coin_type_for(coin)
}

pub fn secret_key_prefix(coin: CoinId) -> u8 {
    vericonomy_hd::secret_key_prefix(coin)
}

pub fn pubkey_to_p2pkh_address(coin: CoinId, pubkey: &[u8]) -> AppResult<String> {
    map_wallet_err(vericonomy_hd::pubkey_to_p2pkh_address(coin, pubkey))
}

pub fn parse_root_xpriv(coin: CoinId, secret: &str) -> AppResult<bitcoin::bip32::Xpriv> {
    map_wallet_err(vericonomy_hd::parse_root_xpriv(coin, secret))
}

pub fn derive_address_at(
    coin: CoinId,
    seed_secret: &str,
    bip39_passphrase: Option<&str>,
    index: u32,
) -> AppResult<String> {
    map_wallet_err(vericonomy_hd::derive_address_at(
        coin,
        seed_secret,
        bip39_passphrase,
        index,
    ))
}

pub fn derive_change_address_at(
    coin: CoinId,
    seed_secret: &str,
    bip39_passphrase: Option<&str>,
    index: u32,
) -> AppResult<String> {
    map_wallet_err(vericonomy_hd::derive_change_address_at(
        coin,
        seed_secret,
        bip39_passphrase,
        index,
    ))
}

pub fn derive_address_on_chain(
    coin: CoinId,
    seed_secret: &str,
    bip39_passphrase: Option<&str>,
    chain: HdChain,
    index: u32,
) -> AppResult<String> {
    map_wallet_err(vericonomy_hd::derive_address_on_chain(
        coin,
        seed_secret,
        bip39_passphrase,
        chain,
        index,
    ))
}

pub fn derive_keypair_at(
    coin: CoinId,
    seed_secret: &str,
    bip39_passphrase: Option<&str>,
    index: u32,
) -> AppResult<([u8; 32], Vec<u8>)> {
    map_wallet_err(vericonomy_hd::derive_keypair_at(
        coin,
        seed_secret,
        bip39_passphrase,
        index,
    ))
}

pub fn derive_keypair_on_chain(
    coin: CoinId,
    seed_secret: &str,
    bip39_passphrase: Option<&str>,
    chain: HdChain,
    index: u32,
) -> AppResult<([u8; 32], Vec<u8>)> {
    map_wallet_err(vericonomy_hd::derive_keypair_on_chain(
        coin,
        seed_secret,
        bip39_passphrase,
        chain,
        index,
    ))
}

pub fn derive_wif_at(
    coin: CoinId,
    mnemonic: &str,
    bip39_passphrase: Option<&str>,
    index: u32,
) -> AppResult<String> {
    map_wallet_err(vericonomy_hd::derive_wif_at(
        coin,
        mnemonic,
        bip39_passphrase,
        index,
    ))
}

pub fn address_to_script_pubkey(coin: CoinId, address: &str) -> AppResult<Vec<u8>> {
    map_wallet_err(vericonomy_hd::address_to_script_pubkey(coin, address))
}

/// Local pre-derivation before Electrum gap scan finishes (Core HD: external + internal).
pub const GAP_PRECACHE_MAX: u32 = 80;

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
                let addr = derive_address_on_chain(
                    coin,
                    seed_secret,
                    bip39_passphrase,
                    chain,
                    index,
                )?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recovery::{secret_bytes_to_wif, secret_key_prefix};

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
}
