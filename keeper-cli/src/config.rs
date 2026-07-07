use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeeperConfig {
    /// Keeper name/identifier
    pub name: String,

    /// Secret key (seed phrase or private key)
    pub secret_key: String,

    /// Soroban RPC endpoint
    pub rpc_url: String,

    /// Aether keeper contract ID
    pub contract_id: String,

    /// Network (testnet/mainnet)
    pub network: String,

    /// Poll interval in seconds
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,

    /// Maximum concurrent task executions
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_tasks: usize,
}

fn default_poll_interval() -> u64 {
    60
}

fn default_max_concurrent() -> usize {
    5
}
