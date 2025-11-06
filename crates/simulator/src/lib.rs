//! Transaction simulation engine using REVM
//!
//! This crate provides:
//! - EVM simulation with mainnet forking
//! - Transaction execution and state tracking
//! - Profitability calculation
//! - Gas estimation

pub mod engine;
pub mod fork;
pub mod profitability;

pub use engine::SimulationEngine;
pub use fork::StateFork;
pub use profitability::ProfitabilityCalculator;
