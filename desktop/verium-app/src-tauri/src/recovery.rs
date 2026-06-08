//! Thin desktop shell over [`vericonomy_wallet_core::recovery`].
//!
//! The cryptographic logic (BIP39, BIP32, WIF) lives in the portable
//! `vericonomy-wallet-core` crate so it can be reused by future mobile shells.
//! This module only maps the app's [`CoinId`] to each chain's Base58 secret-key
//! prefix and adapts the core error type into [`AppError`].

use vericonomy_wallet_core::recovery as core;

use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};

pub use vericonomy_wallet_core::recovery::RecoveryPhraseBundle;

fn map_err(e: vericonomy_wallet_core::WalletCoreError) -> AppError {
    AppError::other(e.to_string())
}

/// Secret-key version byte for Base58Check WIF (must match veriumd `base58Prefixes[SECRET_KEY]`).
///
/// Both mainnet chains use `128 + PUBKEY_ADDRESS(70) = 198` per `chainparams.cpp`
/// (`vericoin/src/chainparams.cpp` line 126 and Verium's equivalent). A WIF encoded
/// with the wrong prefix is rejected by `DecodeSecret`, which silently breaks
/// `sethdseed` and recovery-phrase restore on that chain.
pub fn secret_key_prefix(coin: CoinId) -> u8 {
    match coin {
        CoinId::Verium => 198,   // 128 + 70
        CoinId::Vericoin => 198, // 128 + 70
    }
}

/// Encode a compressed secp256k1 secret as chain-correct WIF for `DecodeSecret` / sethdseed.
pub fn secret_bytes_to_wif(coin: CoinId, secret: &[u8; 32]) -> String {
    core::secret_bytes_to_wif(secret_key_prefix(coin), secret)
}

/// Generate a new 24-word BIP39 mnemonic (256-bit entropy).
pub fn generate_mnemonic() -> AppResult<RecoveryPhraseBundle> {
    core::generate_mnemonic().map_err(map_err)
}

/// Validate a BIP39 mnemonic (checksum included).
pub fn validate_mnemonic(phrase: &str) -> AppResult<bool> {
    Ok(core::validate_mnemonic(phrase))
}

/// Derive the BIP32 master extended private key from a mnemonic + optional BIP39 passphrase.
pub fn derive_master_xpriv(phrase: &str, bip39_passphrase: Option<&str>) -> AppResult<String> {
    core::derive_master_xpriv(phrase, bip39_passphrase).map_err(map_err)
}

/// Derive master private key WIF in the chain's native format for sethdseed.
pub fn master_xpriv_to_wif(
    coin: CoinId,
    phrase: &str,
    bip39_passphrase: Option<&str>,
) -> AppResult<String> {
    core::master_xpriv_to_wif(secret_key_prefix(coin), phrase, bip39_passphrase).map_err(map_err)
}

/// Pick random word indices (0-based) for verification challenge.
pub fn verification_indices(word_count: u32, count: usize) -> Vec<usize> {
    core::verification_indices(word_count, count)
}

/// Verify user-supplied words at given indices.
pub fn verify_words_at_indices(phrase: &str, indices: &[usize], answers: &[String]) -> bool {
    core::verify_words_at_indices(phrase, indices, answers)
}

/// Derive a child private key WIF at m/44'/coin_type'/0'/0/index for address preview.
pub fn derive_account_wif(
    coin: CoinId,
    phrase: &str,
    bip39_passphrase: Option<&str>,
    coin_type: u32,
    index: u32,
) -> AppResult<String> {
    core::derive_account_wif(
        secret_key_prefix(coin),
        phrase,
        bip39_passphrase,
        coin_type,
        index,
    )
    .map_err(map_err)
}

/// Zeroize a string in place (best-effort).
pub fn zeroize_string(s: &mut String) {
    core::zeroize_string(s);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_chains_share_mainnet_secret_prefix() {
        assert_eq!(secret_key_prefix(CoinId::Verium), 198);
        assert_eq!(secret_key_prefix(CoinId::Vericoin), 198);
    }

    #[test]
    fn master_wif_uses_198_prefix_for_both_chains() {
        let phrase = "legal winner thank year wave sausage worth useful legal winner thank yellow";
        for coin in [CoinId::Verium, CoinId::Vericoin] {
            let wif = master_xpriv_to_wif(coin, phrase, None).unwrap();
            let decoded = bs58::decode(&wif).with_check(None).into_vec().unwrap();
            assert_eq!(decoded[0], 198, "{coin:?} master WIF prefix");
            assert_eq!(decoded.len(), 34, "{coin:?} master WIF length");
            assert_eq!(decoded[33], 1, "{coin:?} compressed flag");
        }
    }
}
