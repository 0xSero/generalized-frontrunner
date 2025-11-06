//! Main orchestrator that coordinates all components

use anyhow::Result;
use ethers::prelude::*;
use ethers::signers::LocalWallet;
use mempool::{MempoolMonitor, RpcProvider, TransactionFilter};
use simulator::{ProfitabilityCalculator, SimulationEngine};
use executor::{TransactionBuilder, TransactionSubmitter, SubmissionStrategy};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use types::Config;

/// Main orchestrator that coordinates the frontrunner system
pub struct Orchestrator {
    config: Config,
    db_pool: sqlx::PgPool,
}

impl Orchestrator {
    /// Create a new orchestrator
    pub async fn new(config: Config, db_pool: sqlx::PgPool) -> Result<Self> {
        Ok(Self { config, db_pool })
    }

    /// Run the orchestrator (main event loop)
    pub async fn run(self) -> Result<()> {
        info!("Orchestrator starting");

        // Initialize RPC providers
        let http_provider = Arc::new(
            Provider::<Http>::try_from(&self.config.rpc.providers.alchemy.http_url)?
        );

        let ws_url = self.config.rpc.providers.alchemy.ws_url.clone();
        let primary_provider: Box<dyn RpcProvider> = Box::new(
            mempool::provider::EthereumRpcProvider::new(
                ws_url,
                self.config.rpc.max_reconnect_attempts,
            )
        );

        let provider_manager = Arc::new(mempool::provider::ProviderManager::new(
            primary_provider,
            None, // TODO: Add backup provider
        ));

        // Connect providers
        provider_manager.connect_all().await?;

        // Initialize components
        let filter = self.create_transaction_filter()?;
        let (mempool_monitor, mut tx_receiver) = MempoolMonitor::new(
            provider_manager.clone(),
            filter,
        );

        let simulation_engine = Arc::new(SimulationEngine::new(
            http_provider.clone(),
            self.config.simulation.timeout_ms,
        ));

        let profitability_calculator = Arc::new(self.create_profitability_calculator()?);

        let wallet = Arc::new(self.load_wallet()?);
        let transaction_builder = Arc::new(TransactionBuilder::new(wallet.clone(), 1)); // TODO: Get actual chain ID

        let transaction_submitter = Arc::new(TransactionSubmitter::new(
            http_provider.clone(),
            SubmissionStrategy::PublicOnly, // TODO: Make configurable
        ));

        // Spawn mempool monitor
        let monitor_handle = tokio::spawn(async move {
            if let Err(e) = mempool_monitor.start().await {
                error!(error = %e, "Mempool monitor error");
            }
        });

        // Process transactions
        let mut processed_count = 0u64;
        let mut profitable_count = 0u64;
        let mut submitted_count = 0u64;

        info!("Starting transaction processing loop");

        while let Some(tx) = tx_receiver.recv().await {
            processed_count += 1;

            debug!(
                tx_hash = ?tx.hash,
                from = ?tx.from,
                to = ?tx.to,
                value = %tx.value,
                "Processing transaction"
            );

            // Convert to our transaction type
            let frontrunner_tx = types::Transaction::new(tx.clone());

            // Simulate the transaction
            let sim_result = match simulation_engine.simulate(&frontrunner_tx).await {
                Ok(result) => result,
                Err(e) => {
                    warn!(tx_hash = ?tx.hash, error = %e, "Simulation failed");
                    continue;
                }
            };

            if !sim_result.success {
                debug!(tx_hash = ?tx.hash, "Simulation unsuccessful, skipping");
                continue;
            }

            // Calculate profitability
            let expected_revenue = profitability_calculator
                .estimate_revenue_from_state_changes(&sim_result);

            let gas_price = tx.gas_price.unwrap_or(
                tx.max_fee_per_gas.unwrap_or_default()
            );

            let profitability = match profitability_calculator.calculate(
                &sim_result,
                expected_revenue,
                gas_price,
            ) {
                Ok(analysis) => analysis,
                Err(e) => {
                    warn!(tx_hash = ?tx.hash, error = %e, "Profitability calculation failed");
                    continue;
                }
            };

            if !profitability_calculator.meets_thresholds(&profitability) {
                debug!(
                    tx_hash = ?tx.hash,
                    profit = %profitability.expected_profit,
                    "Transaction not profitable enough"
                );
                continue;
            }

            profitable_count += 1;

            info!(
                tx_hash = ?tx.hash,
                profit = %profitability.expected_profit,
                profit_pct = format!("{:.2}%", profitability.profit_percentage),
                "Profitable transaction found!"
            );

            // TODO: Build frontrunning transaction
            // TODO: Submit transaction
            // TODO: Monitor confirmation
            // TODO: Record results in database

            submitted_count += 1;

            // Log stats every 100 transactions
            if processed_count % 100 == 0 {
                info!(
                    processed = processed_count,
                    profitable = profitable_count,
                    submitted = submitted_count,
                    profit_rate = format!("{:.2}%", (profitable_count as f64 / processed_count as f64) * 100.0),
                    "Transaction processing stats"
                );
            }
        }

        warn!("Transaction receiver channel closed");
        monitor_handle.abort();

        Ok(())
    }

    /// Create transaction filter from config
    fn create_transaction_filter(&self) -> Result<TransactionFilter> {
        let min_value = U256::from_dec_str(&self.config.filters.min_value_wei)?;
        let min_gas_price = U256::from(self.config.filters.min_gas_price_gwei) * U256::exp10(9);
        let max_gas_price = U256::from(self.config.filters.max_gas_price_gwei) * U256::exp10(9);

        Ok(TransactionFilter::new(min_value, min_gas_price, max_gas_price))
    }

    /// Create profitability calculator from config
    fn create_profitability_calculator(&self) -> Result<ProfitabilityCalculator> {
        let min_profit = U256::from_dec_str(&self.config.profitability.min_profit_wei)?;

        Ok(ProfitabilityCalculator::new(
            min_profit,
            self.config.profitability.min_profit_percentage,
            self.config.profitability.max_gas_cost_percentage,
        ))
    }

    /// Load wallet from configuration
    fn load_wallet(&self) -> Result<LocalWallet> {
        let private_key = &self.config.wallet.private_key;

        let wallet = LocalWallet::from_str(private_key)
            .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;

        info!(address = ?wallet.address(), "Wallet loaded");

        Ok(wallet)
    }
}
