//! EVM simulation engine

use ethers::prelude::*;
use revm::{
    primitives::{ExecutionResult, Output, TransactTo, TxEnv},
    Evm,
};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info};
use types::{Error, Result, SimulationResult, Transaction as FrontrunnerTx};

use crate::fork::StateFork;

/// Simulation engine for executing transactions
pub struct SimulationEngine {
    provider: Arc<Provider<Http>>,
    timeout_ms: u64,
}

impl SimulationEngine {
    /// Create a new simulation engine
    pub fn new(provider: Arc<Provider<Http>>, timeout_ms: u64) -> Self {
        Self {
            provider,
            timeout_ms,
        }
    }

    /// Simulate a transaction
    pub async fn simulate(&self, tx: &FrontrunnerTx) -> Result<SimulationResult> {
        let start_time = Instant::now();
        let mut result = SimulationResult::new(tx.hash);

        // Create state fork at latest block
        let mut fork = StateFork::new_latest(self.provider.clone())
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to create state fork");
                e
            })?;

        // Build EVM instance
        let mut evm = Evm::builder()
            .with_db(&mut fork)
            .build();

        // Configure transaction environment
        let tx_env = self.build_tx_env(tx)?;
        evm.tx_mut().clone_from(&tx_env);

        // Execute transaction
        debug!(tx_hash = ?tx.hash, "Executing simulation");

        let execution_result = match evm.transact() {
            Ok(result) => result,
            Err(e) => {
                let error_msg = format!("Simulation execution error: {:?}", e);
                error!(error = %error_msg, tx_hash = ?tx.hash);
                return Ok(result.with_error(error_msg));
            }
        };

        // Process execution result
        match execution_result.result {
            ExecutionResult::Success {
                gas_used,
                gas_refunded,
                output,
                ..
            } => {
                let net_gas_used = gas_used - gas_refunded;

                info!(
                    tx_hash = ?tx.hash,
                    gas_used = net_gas_used,
                    "Simulation succeeded"
                );

                result.gas_used = Some(net_gas_used);
                result.success = true;

                // Extract return data if available
                if let Output::Call(data) = output {
                    debug!(
                        tx_hash = ?tx.hash,
                        return_data_len = data.len(),
                        "Call returned data"
                    );
                }
            }
            ExecutionResult::Revert { gas_used, output } => {
                let error_msg = format!("Transaction reverted: {:?}", output);
                error!(
                    tx_hash = ?tx.hash,
                    gas_used = gas_used,
                    error = %error_msg
                );
                return Ok(result.with_error(error_msg));
            }
            ExecutionResult::Halt { reason, gas_used } => {
                let error_msg = format!("Transaction halted: {:?}", reason);
                error!(
                    tx_hash = ?tx.hash,
                    gas_used = gas_used,
                    error = %error_msg
                );
                return Ok(result.with_error(error_msg));
            }
        }

        // Calculate simulation duration
        result.simulation_duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(result)
    }

    /// Build REVM transaction environment from our transaction type
    fn build_tx_env(&self, tx: &FrontrunnerTx) -> Result<TxEnv> {
        let mut tx_env = TxEnv::default();

        // Set caller
        tx_env.caller = self.convert_address(tx.from);

        // Set transaction target
        tx_env.transact_to = if let Some(to) = tx.to {
            TransactTo::Call(self.convert_address(to))
        } else {
            TransactTo::Create
        };

        // Set value
        tx_env.value = self.convert_u256(tx.value);

        // Set data
        tx_env.data = tx.data.0.clone().into();

        // Set gas limit
        tx_env.gas_limit = tx.gas_limit.as_u64();

        // Set gas price (EIP-1559 or legacy)
        if let (Some(max_fee), Some(priority_fee)) =
            (tx.max_fee_per_gas, tx.max_priority_fee_per_gas)
        {
            tx_env.gas_price = self.convert_u256(max_fee);
            tx_env.gas_priority_fee = Some(self.convert_u256(priority_fee));
        } else if let Some(gas_price) = tx.gas_price {
            tx_env.gas_price = self.convert_u256(gas_price);
        }

        // Set nonce
        tx_env.nonce = Some(tx.nonce.as_u64());

        Ok(tx_env)
    }

    /// Convert ethers Address to revm Address
    fn convert_address(&self, addr: Address) -> revm::primitives::Address {
        revm::primitives::Address::from_slice(addr.as_bytes())
    }

    /// Convert ethers U256 to revm U256
    fn convert_u256(&self, value: U256) -> revm::primitives::U256 {
        let mut bytes = [0u8; 32];
        value.to_big_endian(&mut bytes);
        revm::primitives::U256::from_be_bytes(bytes)
    }

    /// Simulate multiple transactions in parallel
    pub async fn simulate_batch(
        &self,
        transactions: Vec<FrontrunnerTx>,
    ) -> Result<Vec<SimulationResult>> {
        let tasks: Vec<_> = transactions
            .into_iter()
            .map(|tx| {
                let engine = Self::new(self.provider.clone(), self.timeout_ms);
                tokio::spawn(async move { engine.simulate(&tx).await })
            })
            .collect();

        let results = futures::future::join_all(tasks).await;

        let mut simulation_results = Vec::new();
        for result in results {
            match result {
                Ok(Ok(sim_result)) => simulation_results.push(sim_result),
                Ok(Err(e)) => {
                    error!(error = %e, "Simulation failed");
                }
                Err(e) => {
                    error!(error = %e, "Task join error");
                }
            }
        }

        Ok(simulation_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_conversion() {
        let engine = SimulationEngine::new(
            Arc::new(Provider::<Http>::try_from("http://localhost:8545").unwrap()),
            100,
        );

        let addr = Address::random();
        let converted = engine.convert_address(addr);
        assert_eq!(addr.as_bytes(), converted.as_slice());
    }
}
