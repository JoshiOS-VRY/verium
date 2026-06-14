//! P2PKH send-address format checks shared by light and full-node sends (SDK wrapper).

use crate::coin_profile::CoinId;
use crate::error::AppResult;
use crate::sdk_bridge::map_wallet_err;

pub use vericonomy_wallet_engine::address::P2PKH_ADDRESS_LEN;

/// Validate a destination address before building or broadcasting a send.
pub fn validate_send_address(coin: CoinId, address: &str) -> AppResult<()> {
    map_wallet_err(vericonomy_wallet_engine::validate_send_address(coin, address))
}
