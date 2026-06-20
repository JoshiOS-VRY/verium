//! Coin identity, chain constants, and explorer API URLs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CoinId {
    Verium,
    Vericoin,
}

impl CoinId {
    pub fn ticker(self) -> &'static str {
        match self {
            CoinId::Verium => "VRM",
            CoinId::Vericoin => "VRC",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            CoinId::Verium => "verium",
            CoinId::Vericoin => "vericoin",
        }
    }

    pub fn default_rpc_port(self) -> u16 {
        match self {
            CoinId::Verium => 33987,
            CoinId::Vericoin => 33988,
        }
    }

    pub fn data_dir_name(self) -> &'static str {
        match self {
            CoinId::Verium => "Verium",
            CoinId::Vericoin => "Vericoin",
        }
    }

    pub fn explorer_api_base(self) -> &'static str {
        match self {
            CoinId::Verium => "https://explorer.vericonomy.com/v1/vrm/wallet",
            CoinId::Vericoin => "https://explorer.vericonomy.com/v1/vrc/wallet",
        }
    }

    pub fn explorer_chain_api_base(self) -> &'static str {
        match self {
            CoinId::Verium => "https://explorer.vericonomy.com/v1/vrm",
            CoinId::Vericoin => "https://explorer.vericonomy.com/v1/vrc",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "verium" | "vrm" => Some(CoinId::Verium),
            "vericoin" | "vrc" => Some(CoinId::Vericoin),
            _ => None,
        }
    }
}
