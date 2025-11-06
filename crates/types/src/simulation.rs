//! Simulation result types

use ethers::types::{Address, U256, H256};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub id: Uuid,
    pub original_tx_hash: H256,
    pub modified_tx_hash: Option<H256>,
    pub success: bool,
    pub gas_used: Option<u64>,
    pub state_changes: Vec<StateChange>,
    pub profitability: Option<ProfitabilityAnalysis>,
    pub error_message: Option<String>,
    pub simulation_duration_ms: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    pub address: Address,
    pub balance_before: U256,
    pub balance_after: U256,
    pub balance_delta: i128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitabilityAnalysis {
    pub expected_revenue: U256,
    pub expected_cost: U256,
    pub expected_profit: U256,
    pub profit_percentage: f64,
    pub gas_cost_percentage: f64,
    pub is_profitable: bool,
    pub confidence_score: f64,
}

impl SimulationResult {
    pub fn new(original_tx_hash: H256) -> Self {
        Self {
            id: Uuid::new_v4(),
            original_tx_hash,
            modified_tx_hash: None,
            success: false,
            gas_used: None,
            state_changes: Vec::new(),
            profitability: None,
            error_message: None,
            simulation_duration_ms: 0,
            created_at: Utc::now(),
        }
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error_message = Some(error);
        self.success = false;
        self
    }

    pub fn with_profitability(mut self, profitability: ProfitabilityAnalysis) -> Self {
        self.profitability = Some(profitability);
        self.success = true;
        self
    }
}

impl ProfitabilityAnalysis {
    pub fn calculate(
        revenue: U256,
        gas_used: u64,
        gas_price: U256,
        min_profit: U256,
        max_gas_percentage: f64,
    ) -> Self {
        let cost = U256::from(gas_used) * gas_price;
        let profit = if revenue > cost {
            revenue - cost
        } else {
            U256::zero()
        };

        let profit_percentage = if revenue > U256::zero() {
            ((profit.as_u128() as f64) / (revenue.as_u128() as f64)) * 100.0
        } else {
            0.0
        };

        let gas_cost_percentage = if revenue > U256::zero() {
            ((cost.as_u128() as f64) / (revenue.as_u128() as f64)) * 100.0
        } else {
            100.0
        };

        let is_profitable = profit >= min_profit && gas_cost_percentage <= max_gas_percentage;

        // Simple confidence score based on profit margin
        let confidence_score = if is_profitable {
            (profit_percentage / 100.0).min(1.0)
        } else {
            0.0
        };

        Self {
            expected_revenue: revenue,
            expected_cost: cost,
            expected_profit: profit,
            profit_percentage,
            gas_cost_percentage,
            is_profitable,
            confidence_score,
        }
    }
}
