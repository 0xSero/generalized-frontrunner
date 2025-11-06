//! Error types for the frontrunner system

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Ethereum provider error: {0}")]
    Provider(String),

    #[error("Transaction parsing error: {0}")]
    TransactionParsing(String),

    #[error("Simulation error: {0}")]
    Simulation(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Insufficient profit: expected {expected}, got {actual}")]
    InsufficientProfit { expected: u128, actual: u128 },

    #[error("Gas price too high: {0}")]
    GasPriceTooHigh(u64),

    #[error("Nonce error: {0}")]
    Nonce(String),

    #[error("Timeout error: operation timed out after {0}ms")]
    Timeout(u64),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<ethers::providers::ProviderError> for Error {
    fn from(err: ethers::providers::ProviderError) -> Self {
        Error::Provider(err.to_string())
    }
}
