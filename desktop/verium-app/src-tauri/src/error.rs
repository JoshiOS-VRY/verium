use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("daemon not reachable: {0}")]
    DaemonUnreachable(String),

    #[error("rpc error {code}: {message}")]
    Rpc { code: i64, message: String },

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("invalid config: {0}")]
    Config(String),

    #[error("electrum error {code}: {message}")]
    Electrum { code: i32, message: String },

    #[error("{0}")]
    Other(String),
}

impl AppError {
    pub fn other(msg: impl Into<String>) -> Self {
        AppError::Other(msg.into())
    }

    /// Copy shown in the UI. Keeps daemon detail when it is already readable.
    pub fn user_facing_message(&self) -> String {
        match self {
            AppError::DaemonUnreachable(msg) => friendly_daemon_unreachable(msg),
            AppError::Rpc { code, message } => friendly_rpc_error(*code, message),
            AppError::Electrum { code, message } => friendly_electrum_error(*code, message),
            AppError::Http(err) => {
                if err.is_timeout() || err.is_connect() {
                    "Couldn't reach the node. It may still be starting — try again shortly.".into()
                } else {
                    "A network request failed. Check your connection and try again.".into()
                }
            }
            AppError::Io(_) => {
                "Couldn't read or write a file on this device.".into()
            }
            AppError::Serde(_) => {
                "Received an unexpected response from the node.".into()
            }
            AppError::Config(msg) => msg.clone(),
            AppError::Other(msg) => friendly_freeform_message(msg),
        }
    }

    pub fn is_electrum_rate_limited(&self) -> bool {
        matches!(
            self,
            AppError::Electrum {
                code: -101,
                message: _
            }
        ) || matches!(
            self,
            AppError::Other(msg) if msg.contains("error -101")
                || msg.to_ascii_lowercase().contains("excessive resource")
        )
    }

    pub fn is_indexing_budget_exhausted(&self) -> bool {
        matches!(
            self,
            AppError::Other(msg) if msg.contains("indexing batch limit reached")
        )
    }

    pub fn is_electrum_transport(&self) -> bool {
        matches!(
            self,
            AppError::Other(msg) if msg.contains("electrum connect")
                || msg.contains("electrum connection closed")
                || msg.contains("electrum read:")
                || msg.contains("electrum write:")
                || msg.contains("electrum TLS")
                || msg.contains("electrum call timed out")
        )
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Other(format!("{e:#}"))
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.user_facing_message())
    }
}

pub type AppResult<T> = Result<T, AppError>;

/// Bitcoin-style RPC warmup (daemon starting, loading block index, etc.).
pub fn is_rpc_warmup(code: i64) -> bool {
    code == -28 || code == -10
}

fn friendly_daemon_unreachable(raw: &str) -> String {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("unauthorized") {
        return "Couldn't connect to the node — RPC credentials are missing or incorrect. Check Settings → Daemon connection.".into();
    }
    crate::node::status::friendly_connection_error(raw)
}

fn friendly_rpc_error(code: i64, message: &str) -> String {
    if let Some(mapped) = rpc_code_message(code) {
        return mapped.into();
    }
    refine_operational_message(message)
}

fn friendly_electrum_error(code: i32, message: &str) -> String {
    match code {
        -101 => "The light-wallet server is busy. Wait a moment and try again.".into(),
        -32600 | -32602 => refine_operational_message(message),
        _ if message.to_ascii_lowercase().contains("excessive resource") => {
            "The light-wallet server is busy. Wait a moment and try again.".into()
        }
        _ => refine_operational_message(message),
    }
}

fn friendly_freeform_message(raw: &str) -> String {
    if let Some((code, message)) = parse_prefixed_rpc_error(raw) {
        return friendly_rpc_error(code, message);
    }
    if let Some((code, message)) = parse_prefixed_electrum_error(raw) {
        return friendly_electrum_error(code, message);
    }
    if raw.starts_with("daemon not reachable:") {
        return friendly_daemon_unreachable(raw.trim_start_matches("daemon not reachable:").trim());
    }
    refine_operational_message(raw)
}

fn rpc_code_message(code: i64) -> Option<&'static str> {
    match code {
        -2 => Some("The node is in safe mode, so this action isn't available right now."),
        -3 => Some("One of the values sent to the node has the wrong type."),
        -5 => Some("That address or key doesn't look valid."),
        -6 => Some("Not enough spendable balance for this amount and fee."),
        -7 => Some("The node ran out of memory while handling this request."),
        -9 => Some("The node isn't connected to the network yet."),
        -10 => Some("The node is still downloading the blockchain."),
        -11 => Some("That label name isn't valid."),
        -12 => Some("The wallet ran out of unused addresses. Try again after it refills."),
        -13 => Some("Unlock your wallet first to continue."),
        -14 => Some("Incorrect wallet passphrase."),
        -15 => Some("This action doesn't match the wallet's encryption state."),
        -16 => Some("Couldn't encrypt the wallet."),
        -17 => Some("Wallet is already unlocked."),
        -18 => Some("That wallet isn't loaded on the node."),
        -19 => Some("No wallet is selected on the node."),
        -20 => Some("A local database error occurred. Try restarting the app."),
        -27 => Some("This transaction is already on the blockchain."),
        -28 => Some("The node is still starting up. Try again in a moment."),
        -29 => Some("That peer isn't connected."),
        -31 => Some("Peer networking is disabled on this node."),
        -32 => Some("That RPC command is deprecated on this node."),
        -32601 => Some("That command isn't available on the connected node."),
        -32603 => Some("The node hit an internal error. Check Logs for details."),
        _ => None,
    }
}

fn refine_operational_message(message: &str) -> String {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return "Something went wrong. Try again.".into();
    }

    let lower = trimmed.to_ascii_lowercase();

    if lower.contains("incorrect wallet passphrase")
        || lower.contains("decrypt failed")
        || lower.contains("passphrase entered was incorrect")
    {
        return "Incorrect wallet passphrase.".into();
    }
    if lower.contains("wallet passphrase is required") || lower.contains("passphrase required to send")
    {
        return "Enter your wallet passphrase to send.".into();
    }
    if lower.contains("insufficient funds") {
        return "Not enough spendable balance for this amount and fee.".into();
    }
    if lower.contains("fee too low") || lower.contains("min relay fee") {
        return "The network fee is too low. Raise the fee and try again.".into();
    }
    if lower.contains("dust") {
        return "That amount is too small to send after fees.".into();
    }
    if lower.contains("absurdly-high-fee") || lower.contains("absurdly high fee") {
        return "The fee looks unusually high. Lower the fee and try again.".into();
    }
    if lower.contains("transaction too large") {
        return "This transaction is too large. Try sending a smaller amount.".into();
    }
    if lower.contains("invalid address") {
        return "That address doesn't look valid.".into();
    }
    if lower.contains("wallet is locked") || lower.contains("unlock needed") {
        return "Unlock your wallet first to continue.".into();
    }
    if lower.contains("rescanning") {
        return "The wallet is rescanning. Wait for it to finish, then try again.".into();
    }
    if lower.contains("warming up") || lower.contains("loading block index") {
        return "The node is still starting up. Try again in a moment.".into();
    }
    if lower.contains("work queue depth exceeded") {
        return "The node is busy starting up. Try again in a moment.".into();
    }
    if lower.contains("electrum connect")
        || lower.contains("electrum connection closed")
        || lower.contains("electrum read:")
        || lower.contains("electrum write:")
        || lower.contains("electrum tls")
        || lower.contains("electrum call timed out")
    {
        return "Couldn't reach the light-wallet server. Check your connection or try another server.".into();
    }
    if lower.contains("excessive resource") || lower.contains("error -101") {
        return "The light-wallet server is busy. Wait a moment and try again.".into();
    }
    if lower.contains("indexing batch limit reached") {
        return "Address lookup is taking longer than expected. Try again in a few minutes.".into();
    }
    if lower.contains("manual rescan cooldown") || lower.contains("rescan cooldown") {
        return "You can run another address rescan in about an hour.".into();
    }

    trimmed.to_string()
}

fn parse_prefixed_rpc_error(raw: &str) -> Option<(i64, &str)> {
    let rest = raw.strip_prefix("rpc error ")?;
    let (code_str, message) = rest.split_once(": ")?;
    let code = code_str.parse().ok()?;
    Some((code, message))
}

fn parse_prefixed_electrum_error(raw: &str) -> Option<(i32, &str)> {
    let rest = raw.strip_prefix("electrum error ")?;
    let (code_str, message) = rest.split_once(": ")?;
    let code = code_str.parse().ok()?;
    Some((code, message))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_insufficient_funds_maps_friendly() {
        let err = AppError::Rpc {
            code: -6,
            message: "Insufficient funds".into(),
        };
        assert_eq!(
            err.user_facing_message(),
            "Not enough spendable balance for this amount and fee."
        );
    }

    #[test]
    fn rpc_wrong_passphrase_maps_friendly() {
        let err = AppError::Rpc {
            code: -14,
            message: "The wallet passphrase entered was incorrect".into(),
        };
        assert_eq!(err.user_facing_message(), "Incorrect wallet passphrase.");
    }

    #[test]
    fn warmup_code_maps_friendly() {
        let err = AppError::Rpc {
            code: -28,
            message: "Verium Core is loading…".into(),
        };
        assert_eq!(
            err.user_facing_message(),
            "The node is still starting up. Try again in a moment."
        );
    }

    #[test]
    fn wallet_error_message_heuristics() {
        let err = AppError::Rpc {
            code: -4,
            message: "Fee amount not valid".into(),
        };
        assert_eq!(err.user_facing_message(), "Fee amount not valid");
    }

    #[test]
    fn electrum_rate_limit_maps_friendly() {
        let err = AppError::Electrum {
            code: -101,
            message: "excessive resource".into(),
        };
        assert_eq!(
            err.user_facing_message(),
            "The light-wallet server is busy. Wait a moment and try again."
        );
    }

    #[test]
    fn serialize_uses_user_facing_message() {
        let err = AppError::Rpc {
            code: -13,
            message: "Please enter the wallet passphrase with walletpassphrase first.".into(),
        };
        let json = serde_json::to_string(&err).expect("serialize");
        assert_eq!(json, "\"Unlock your wallet first to continue.\"");
    }
}
