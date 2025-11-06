//! Shared types and data structures for the frontrunner system

pub mod config;
pub mod transaction;
pub mod simulation;
pub mod execution;
pub mod error;

pub use config::Config;
pub use error::{Error, Result};
pub use transaction::{Transaction, TransactionStatus};
pub use simulation::{SimulationResult, ProfitabilityAnalysis};
pub use execution::{SubmissionPath, ExecutionResult};
