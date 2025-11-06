//! Configuration types and structures

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub environment: Environment,
    pub rpc: RpcConfig,
    pub mempool: MempoolConfig,
    pub filters: FilterConfig,
    pub simulation: SimulationConfig,
    pub profitability: ProfitabilityConfig,
    pub gas: GasConfig,
    pub submission: SubmissionConfig,
    pub wallet: WalletConfig,
    pub database: DatabaseConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Environment {
    pub name: String,
    pub log_level: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RpcConfig {
    pub primary_provider: String,
    pub backup_provider: Option<String>,
    pub websocket_timeout_seconds: u64,
    pub max_reconnect_attempts: u32,
    pub rate_limit_requests_per_second: u32,
    pub providers: RpcProviders,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RpcProviders {
    pub alchemy: ProviderConfig,
    pub infura: ProviderConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderConfig {
    pub http_url: String,
    pub ws_url: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MempoolConfig {
    pub queue_size: usize,
    pub worker_threads: usize,
    pub transaction_timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilterConfig {
    pub min_value_wei: String,
    pub min_gas_price_gwei: u64,
    pub max_gas_price_gwei: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimulationConfig {
    pub enabled: bool,
    pub timeout_ms: u64,
    pub max_parallel_simulations: usize,
    pub fork_block_offset: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfitabilityConfig {
    pub min_profit_wei: String,
    pub min_profit_percentage: f64,
    pub max_gas_cost_percentage: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GasConfig {
    pub max_priority_fee_multiplier: f64,
    pub max_base_fee_multiplier: f64,
    pub absolute_max_gwei: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubmissionConfig {
    pub enabled_paths: Vec<String>,
    pub flashbots_enabled: bool,
    pub flashbots_relay_url: String,
    pub max_submission_attempts: u32,
    pub flashbots: FlashbotsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FlashbotsConfig {
    pub signing_key: String,
    pub min_reputation_score: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletConfig {
    pub private_key: String,
    pub address: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
    pub metrics_port: u16,
    pub health_check_port: u16,
    pub enable_tracing: bool,
}

impl Config {
    /// Load configuration from environment variables and config files
    pub fn load() -> Result<Self, config::ConfigError> {
        let mut builder = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::Environment::default().separator("__"));

        if let Ok(env) = std::env::var("ENVIRONMENT") {
            builder = builder.add_source(
                config::File::with_name(&format!("config/{}", env)).required(false),
            );
        }

        builder.build()?.try_deserialize()
    }
}
