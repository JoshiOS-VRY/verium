//! Local transaction signing for light wallet (SDK wrapper).

use crate::chain::types::Utxo;
use crate::coin_profile::CoinId;
use crate::error::AppResult;
use crate::sdk_bridge::map_wallet_err;

pub use vericonomy_wallet_engine::signer::SignedTx;

pub fn build_unsigned_hex(
    coin: CoinId,
    mnemonic: &str,
    bip39_passphrase: Option<&str>,
    utxos: &[Utxo],
    outputs: &[(String, i64)],
    change_address: &str,
    fee_sats: i64,
) -> AppResult<String> {
    map_wallet_err(vericonomy_wallet_engine::signer::build_unsigned_hex(
        coin,
        mnemonic,
        bip39_passphrase,
        utxos,
        outputs,
        change_address,
        fee_sats,
    ))
}

pub fn sign_transaction(
    coin: CoinId,
    mnemonic: &str,
    bip39_passphrase: Option<&str>,
    utxos: &[Utxo],
    outputs: &[(String, i64)],
    change_address: &str,
    fee_sats: i64,
) -> AppResult<SignedTx> {
    map_wallet_err(vericonomy_wallet_engine::signer::sign_transaction(
        coin,
        mnemonic,
        bip39_passphrase,
        utxos,
        outputs,
        change_address,
        fee_sats,
    ))
}
