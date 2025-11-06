//! Mempool monitoring and transaction ingestion service
//!
//! This crate provides functionality to:
//! - Connect to Ethereum RPC providers via WebSocket
//! - Subscribe to pending transactions
//! - Filter transactions based on configurable criteria
//! - Queue transactions for analysis

pub mod provider;
pub mod monitor;
pub mod filter;

pub use monitor::MempoolMonitor;
pub use filter::TransactionFilter;
pub use provider::RpcProvider;
