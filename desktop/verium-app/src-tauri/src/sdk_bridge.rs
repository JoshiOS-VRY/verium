//! Maps between app errors and SDK [`WalletError`].

use vericonomy_errors::WalletError;

use crate::error::{AppError, AppResult};

pub fn map_wallet_err<T>(result: vericonomy_errors::Result<T>) -> AppResult<T> {
    result.map_err(Into::into)
}

impl From<WalletError> for AppError {
    fn from(e: WalletError) -> Self {
        match e {
            WalletError::Rpc { code, message } => AppError::Rpc {
                code: code as i64,
                message,
            },
            WalletError::Electrum { code, message } => AppError::Electrum {
                code: code as i32,
                message,
            },
            WalletError::Http { message, .. } => AppError::other(format!("http error: {message}")),
            WalletError::Io { message, .. } => AppError::other(format!("io error: {message}")),
            WalletError::Serde { message, .. } => AppError::other(format!("serde error: {message}")),
            WalletError::Storage { message, .. } => {
                AppError::other(format!("storage error: {message}"))
            }
            WalletError::LockedWallet { .. } => {
                AppError::other("Unlock your wallet first to continue.")
            }
            WalletError::WrongPassphrase { .. } => {
                AppError::other("Incorrect wallet passphrase.")
            }
            WalletError::InsufficientFunds { message, .. } => AppError::Rpc {
                code: -6,
                message,
            },
            other => AppError::Other(other.to_string()),
        }
    }
}

impl From<AppError> for WalletError {
    fn from(e: AppError) -> Self {
        match e {
            AppError::Rpc { code, message } => WalletError::Rpc {
                code: code.max(0) as u32,
                message,
            },
            AppError::Electrum { code, message } => WalletError::Electrum {
                code: code.max(0) as u32,
                message,
            },
            AppError::Http(err) => WalletError::Http {
                code: WalletError::CODE_HTTP,
                message: err.to_string(),
            },
            AppError::Io(err) => WalletError::Io {
                code: WalletError::CODE_IO,
                message: err.to_string(),
            },
            AppError::Serde(err) => WalletError::Serde {
                code: WalletError::CODE_SERDE,
                message: err.to_string(),
            },
            AppError::Config(msg) => WalletError::Other {
                code: WalletError::CODE_OTHER,
                message: msg,
            },
            AppError::DaemonUnreachable(msg) => WalletError::other(format!(
                "daemon not reachable: {msg}"
            )),
            AppError::Other(msg) => WalletError::other(msg),
        }
    }
}
