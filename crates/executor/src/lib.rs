//! Transaction execution and submission
//!
//! This crate handles:
//! - Transaction building and signing
//! - Gas price optimization
//! - Multi-path submission (public, Flashbots, private relays)
//! - Transaction monitoring and confirmation
//! - Nonce management

pub mod builder;
pub mod submitter;
pub mod flashbots;
pub mod nonce;

pub use builder::TransactionBuilder;
pub use submitter::{TransactionSubmitter, SubmissionStrategy};
pub use flashbots::FlashbotsSubmitter;
pub use nonce::NonceManager;
