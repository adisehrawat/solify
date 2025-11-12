pub mod tui;
pub mod commands;
pub mod utils;

pub use utils::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_RPC_URL: &str = "https://api.devnet.solana.com";

#[derive(Debug, Clone)]
pub struct CliConfig {
    pub rpc_url: String,
    pub verbose: bool,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            rpc_url: DEFAULT_RPC_URL.to_string(),
            verbose: false,
        }
    }
}

impl CliConfig {
    pub fn new(rpc_url: String, verbose: bool) -> Self {
        Self { rpc_url, verbose }
    }
}

