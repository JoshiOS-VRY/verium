//! Portable wallet core with **zero Tauri (and zero app) dependencies**.
//!
//! This crate owns the cryptographic primitives that must behave identically on
//! desktop and on future mobile shells (iOS/Android via uniffi or Tauri Mobile):
//! BIP39 mnemonic generation/validation, BIP32 master-key derivation, and
//! chain-agnostic Base58Check WIF encoding.
//!
//! Chain specifics (the Base58 secret-key prefix and BIP44 coin type) are passed
//! in as plain values so the crate never depends on the app's `CoinId` enum or
//! its preferences/keystore layer. The desktop shell maps `CoinId` to these
//! values; mobile shells can do the same without duplicating crypto logic.

mod error;
pub mod recovery;

pub use error::WalletCoreError;
pub use recovery::{
    derive_account_wif, derive_master_xpriv, generate_mnemonic, master_xpriv_to_wif,
    secret_bytes_to_wif, validate_mnemonic, verification_indices, verify_words_at_indices,
    zeroize_string, RecoveryPhraseBundle,
};
