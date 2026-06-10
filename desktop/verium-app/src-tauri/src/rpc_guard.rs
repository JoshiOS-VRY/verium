//! RPC console method policy for `rpc_raw_call`.

use crate::error::{AppError, AppResult};

const BLOCKED_METHODS: &[&str] = &[
    "dumpprivkey",
    "dumpwallet",
    "importprivkey",
    "importwallet",
    "sethdseed",
    "sendtoaddress",
    "sendmany",
    "sendfrom",
    "signrawtransactionwithwallet",
    "walletpassphrasechange",
    "encryptwallet",
];

pub fn assert_rpc_method_allowed(method: &str) -> AppResult<()> {
    let normalized = method.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(AppError::other("method must not be empty"));
    }
    if BLOCKED_METHODS.contains(&normalized.as_str()) {
        return Err(AppError::other(format!(
            "RPC method '{method}' is blocked in the wallet console. Use the wallet UI or enable 2FA-gated commands."
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_dumpprivkey() {
        assert!(assert_rpc_method_allowed("dumpprivkey").is_err());
    }

    #[test]
    fn allows_getblockchaininfo() {
        assert!(assert_rpc_method_allowed("getblockchaininfo").is_ok());
    }
}
