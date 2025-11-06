//! Profitability calculation for simulations

use ethers::types::U256;
use tracing::{debug, info};
use types::{ProfitabilityAnalysis, Result, SimulationResult};

/// Calculator for determining transaction profitability
pub struct ProfitabilityCalculator {
    min_profit: U256,
    min_profit_percentage: f64,
    max_gas_cost_percentage: f64,
}

impl ProfitabilityCalculator {
    /// Create a new profitability calculator
    pub fn new(
        min_profit: U256,
        min_profit_percentage: f64,
        max_gas_cost_percentage: f64,
    ) -> Self {
        Self {
            min_profit,
            min_profit_percentage,
            max_gas_cost_percentage,
        }
    }

    /// Calculate profitability from a simulation result
    pub fn calculate(
        &self,
        simulation: &SimulationResult,
        expected_revenue: U256,
        gas_price: U256,
    ) -> Result<ProfitabilityAnalysis> {
        let gas_used = simulation
            .gas_used
            .ok_or_else(|| types::Error::Simulation("No gas usage data".to_string()))?;

        let analysis = ProfitabilityAnalysis::calculate(
            expected_revenue,
            gas_used,
            gas_price,
            self.min_profit,
            self.max_gas_cost_percentage,
        );

        if analysis.is_profitable {
            info!(
                simulation_id = %simulation.id,
                revenue = %analysis.expected_revenue,
                cost = %analysis.expected_cost,
                profit = %analysis.expected_profit,
                profit_pct = format!("{:.2}%", analysis.profit_percentage),
                "Transaction is profitable"
            );
        } else {
            debug!(
                simulation_id = %simulation.id,
                revenue = %analysis.expected_revenue,
                cost = %analysis.expected_cost,
                profit = %analysis.expected_profit,
                "Transaction is not profitable"
            );
        }

        Ok(analysis)
    }

    /// Estimate revenue from state changes
    pub fn estimate_revenue_from_state_changes(
        &self,
        simulation: &SimulationResult,
    ) -> U256 {
        // Sum up all positive balance changes for our address
        simulation
            .state_changes
            .iter()
            .filter(|change| change.balance_delta > 0)
            .map(|change| U256::from(change.balance_delta as u128))
            .fold(U256::zero(), |acc, val| acc + val)
    }

    /// Check if a transaction meets minimum thresholds
    pub fn meets_thresholds(&self, analysis: &ProfitabilityAnalysis) -> bool {
        analysis.is_profitable
            && analysis.expected_profit >= self.min_profit
            && analysis.profit_percentage >= self.min_profit_percentage
            && analysis.gas_cost_percentage <= self.max_gas_cost_percentage
    }

    /// Calculate optimal gas price for frontrunning
    pub fn calculate_optimal_gas_price(
        &self,
        target_gas_price: U256,
        max_multiplier: f64,
    ) -> U256 {
        // Increase gas price by multiplier to ensure frontrunning
        let multiplier_basis_points = (max_multiplier * 10000.0) as u128;
        let increased = target_gas_price
            .checked_mul(U256::from(multiplier_basis_points))
            .unwrap_or(target_gas_price)
            / U256::from(10000u64);

        increased
    }

    /// Estimate if frontrunning will be profitable after gas increase
    pub fn is_frontrun_profitable(
        &self,
        revenue: U256,
        gas_used: u64,
        target_gas_price: U256,
        gas_multiplier: f64,
    ) -> bool {
        let frontrun_gas_price = self.calculate_optimal_gas_price(target_gas_price, gas_multiplier);
        let cost = U256::from(gas_used) * frontrun_gas_price;

        if revenue <= cost {
            return false;
        }

        let profit = revenue - cost;
        profit >= self.min_profit
    }
}

impl Default for ProfitabilityCalculator {
    fn default() -> Self {
        Self {
            min_profit: U256::from(50_000_000_000_000_000u64), // 0.05 ETH
            min_profit_percentage: 5.0,
            max_gas_cost_percentage: 70.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_profitable_calculation() {
        let calculator = ProfitabilityCalculator::default();

        let revenue = U256::from(1_000_000_000_000_000_000u64); // 1 ETH
        let gas_used = 100_000;
        let gas_price = U256::from(50_000_000_000u64); // 50 gwei

        let analysis = ProfitabilityAnalysis::calculate(
            revenue,
            gas_used,
            gas_price,
            calculator.min_profit,
            calculator.max_gas_cost_percentage,
        );

        assert!(analysis.is_profitable);
        assert!(calculator.meets_thresholds(&analysis));
    }

    #[test]
    fn test_unprofitable_high_gas() {
        let calculator = ProfitabilityCalculator::default();

        let revenue = U256::from(100_000_000_000_000_000u64); // 0.1 ETH
        let gas_used = 500_000;
        let gas_price = U256::from(200_000_000_000u64); // 200 gwei

        let analysis = ProfitabilityAnalysis::calculate(
            revenue,
            gas_used,
            gas_price,
            calculator.min_profit,
            calculator.max_gas_cost_percentage,
        );

        // Gas cost would be 0.1 ETH (100%), not profitable
        assert!(!analysis.is_profitable);
    }

    #[test]
    fn test_gas_price_increase() {
        let calculator = ProfitabilityCalculator::default();
        let base_gas_price = U256::from(50_000_000_000u64); // 50 gwei

        let increased = calculator.calculate_optimal_gas_price(base_gas_price, 1.2);

        // Should be 60 gwei (20% increase)
        assert_eq!(increased, U256::from(60_000_000_000u64));
    }
}
