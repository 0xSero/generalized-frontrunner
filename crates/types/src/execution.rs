//! Execution and submission types

use ethers::types::{H256, U256};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SubmissionPath {
    PublicMempool,
    Flashbots,
    PrivateRelay(String),
}

impl std::fmt::Display for SubmissionPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PublicMempool => write!(f, "public"),
            Self::Flashbots => write!(f, "flashbots"),
            Self::PrivateRelay(name) => write!(f, "relay:{}", name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub id: Uuid,
    pub simulation_id: Uuid,
    pub submitted_tx_hash: H256,
    pub submission_path: SubmissionPath,
    pub block_number: Option<u64>,
    pub gas_used: Option<u64>,
    pub actual_gas_price: Option<U256>,
    pub actual_profit: Option<U256>,
    pub success: bool,
    pub inclusion_time_ms: Option<u64>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl ExecutionResult {
    pub fn new(
        simulation_id: Uuid,
        submitted_tx_hash: H256,
        submission_path: SubmissionPath,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            simulation_id,
            submitted_tx_hash,
            submission_path,
            block_number: None,
            gas_used: None,
            actual_gas_price: None,
            actual_profit: None,
            success: false,
            inclusion_time_ms: None,
            error_message: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_confirmation(
        mut self,
        block_number: u64,
        gas_used: u64,
        gas_price: U256,
        profit: U256,
        inclusion_time_ms: u64,
    ) -> Self {
        self.block_number = Some(block_number);
        self.gas_used = Some(gas_used);
        self.actual_gas_price = Some(gas_price);
        self.actual_profit = Some(profit);
        self.inclusion_time_ms = Some(inclusion_time_ms);
        self.success = true;
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error_message = Some(error);
        self.success = false;
        self
    }
}
