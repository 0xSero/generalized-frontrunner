//! Complete orchestrator implementation with all features

use anyhow::Result;
use ethers::prelude::*;
use ethers::signers::LocalWallet;
use mempool::{AddressReplacer, MempoolMonitor, RpcProvider, TransactionAnalyzer, TransactionFilter};
use simulator::{ProfitabilityCalculator, SimulationEngine};
use executor::{NonceManager, TransactionBuilder, TransactionSubmitter, SubmissionStrategy};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use types::Config;

use crate::database::Database;

/// Complete orchestrator with all features integrated
pub struct CompleteOrchestrator {
    config: Config,
    db: Database,
}

impl CompleteOrchestrator {
    /// Create a new complete orchestrator
    pub async fn new(config: Config, db_pool: sqlx::PgPool) -> Result<Self> {
        let db = Database::new(db_pool);

        Ok(Self { config, db })
    }

    /// Run the complete orchestrator
    pub async fn run(self) -> Result<()> {
        info!("Complete Orchestrator starting with all features");

        // Initialize RPC providers
        let http_provider = Arc::new(
            Provider::<Http>::try_from(&self.config.rpc.providers.alchemy.http_url)?
        );

        // Get chain ID
        let chain_id = http_provider
            .get_chainid()
            .await?
            .as_u64();

        info!(chain_id = chain_id, "Connected to chain");

        let ws_url = self.config.rpc.providers.alchemy.ws_url.clone();
        let primary_provider: Box<dyn RpcProvider> = Box::new(
            mempool::provider::EthereumRpcProvider::new(
                ws_url,
                self.config.rpc.max_reconnect_attempts,
            )
        );

        let provider_manager = Arc::new(mempool::provider::ProviderManager::new(
            primary_provider,
            None,
        ));

        provider_manager.connect_all().await?;

        // Initialize wallet and nonce manager
        let wallet = Arc::new(self.load_wallet()?);
        let our_address = wallet.address();

        info!(address = ?our_address, "Wallet loaded");

        let nonce_manager = Arc::new(
            NonceManager::new(http_provider.clone(), our_address).await?
        );

        // Initialize components
        let filter = self.create_transaction_filter()?;
        let (mempool_monitor, mut tx_receiver) = MempoolMonitor::new(
            provider_manager.clone(),
            filter,
        );

        let transaction_analyzer = Arc::new(TransactionAnalyzer::new());
        let address_replacer = Arc::new(AddressReplacer::new(our_address));

        let simulation_engine = Arc::new(SimulationEngine::new(
            http_provider.clone(),
            self.config.simulation.timeout_ms,
        ));

        let profitability_calculator = Arc::new(self.create_profitability_calculator()?);

        let transaction_builder = Arc::new(TransactionBuilder::new(wallet.clone(), chain_id));

        let transaction_submitter = Arc::new(TransactionSubmitter::new(
            http_provider.clone(),
            SubmissionStrategy::PublicOnly,
        ));

        // Spawn mempool monitor
        let monitor_handle = tokio::spawn(async move {
            if let Err(e) = mempool_monitor.start().await {
                error!(error = %e, "Mempool monitor error");
            }
        });

        // Spawn nonce sync task (every 30 seconds)
        let nonce_manager_clone = nonce_manager.clone();
        let nonce_sync_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(e) = nonce_manager_clone.sync_with_network().await {
                    warn!(error = %e, "Failed to sync nonce");
                }
            }
        });

        // Main processing loop
        let mut stats = ProcessingStats::default();
        info!("Starting complete transaction processing loop");

        while let Some(tx) = tx_receiver.recv().await {
            stats.processed += 1;

            // Parse transaction
            let parsed_tx = match transaction_analyzer.parse(tx.clone()) {
                Ok(parsed) => parsed,
                Err(e) => {
                    warn!(tx_hash = ?tx.hash, error = %e, "Failed to parse transaction");
                    continue;
                }
            };

            debug!(
                tx_hash = ?tx.hash,
                tx_type = ?parsed_tx.transaction_type,
                "Parsed transaction"
            );

            // Save to database
            if let Err(e) = self.db.save_transaction(&parsed_tx.transaction).await {
                warn!(error = %e, "Failed to save transaction");
            }

            // Identify replaceable addresses
            let replaceable_addresses = address_replacer
                .identify_replaceable_addresses(&parsed_tx, tx.from);

            if replaceable_addresses.is_empty() {
                debug!(tx_hash = ?tx.hash, "No replaceable addresses found");
                continue;
            }

            // Replace addresses in calldata
            let modified_data = address_replacer.replace_addresses(
                &tx.input,
                &replaceable_addresses,
            );

            // Create modified transaction for simulation
            let nonce = nonce_manager.get_next_nonce().await?;
            let modified_tx = parsed_tx.transaction.create_frontrun_copy(
                our_address,
                modified_data,
                U64::from(nonce.as_u64()),
            );

            // Simulate
            stats.simulated += 1;
            let mut sim_result = match simulation_engine.simulate(&modified_tx).await {
                Ok(result) => result,
                Err(e) => {
                    warn!(tx_hash = ?tx.hash, error = %e, "Simulation failed");
                    nonce_manager.release_nonce(nonce).await;
                    continue;
                }
            };

            // Save simulation result
            if let Err(e) = self.db.save_simulation(&sim_result).await {
                warn!(error = %e, "Failed to save simulation");
            }

            if !sim_result.success {
                debug!(tx_hash = ?tx.hash, "Simulation unsuccessful");
                nonce_manager.release_nonce(nonce).await;
                continue;
            }

            stats.sim_successful += 1;

            // Calculate profitability
            let expected_revenue = profitability_calculator
                .estimate_revenue_from_state_changes(&sim_result);

            let gas_price = tx.gas_price.unwrap_or(
                tx.max_fee_per_gas.unwrap_or_default()
            );

            // Calculate frontrunning gas price
            let base_fee = http_provider.get_gas_price().await?;
            let (max_fee, priority_fee) = transaction_builder.calculate_frontrun_gas(
                base_fee,
                tx.max_priority_fee_per_gas.unwrap_or(U256::from(2_000_000_000u64)),
                self.config.gas.max_priority_fee_multiplier,
                self.config.gas.max_base_fee_multiplier,
            );

            let profitability = match profitability_calculator.calculate(
                &sim_result,
                expected_revenue,
                max_fee,
            ) {
                Ok(analysis) => analysis,
                Err(e) => {
                    warn!(tx_hash = ?tx.hash, error = %e, "Profitability calculation failed");
                    nonce_manager.release_nonce(nonce).await;
                    continue;
                }
            };

            // Update simulation with profitability
            sim_result = sim_result.with_profitability(profitability.clone());

            if !profitability_calculator.meets_thresholds(&profitability) {
                debug!(
                    tx_hash = ?tx.hash,
                    profit = %profitability.expected_profit,
                    "Not profitable enough"
                );
                nonce_manager.release_nonce(nonce).await;
                continue;
            }

            stats.profitable += 1;

            info!(
                tx_hash = ?tx.hash,
                profit = %profitability.expected_profit,
                profit_pct = format!("{:.2}%", profitability.profit_percentage),
                gas_cost_pct = format!("{:.2}%", profitability.gas_cost_percentage),
                "💰 PROFITABLE TRANSACTION FOUND!"
            );

            // Build and sign frontrunning transaction
            let raw_tx = match transaction_builder.build_and_sign(
                modified_tx.to.unwrap(),
                modified_tx.data.clone(),
                modified_tx.value,
                modified_tx.gas_limit,
                max_fee,
                priority_fee,
                nonce,
            ).await {
                Ok(tx) => tx,
                Err(e) => {
                    error!(error = %e, "Failed to build transaction");
                    nonce_manager.release_nonce(nonce).await;
                    continue;
                }
            };

            // Submit transaction
            stats.submitted += 1;
            let submitted_hash = match transaction_submitter
                .submit_with_retry(raw_tx, self.config.submission.max_submission_attempts)
                .await
            {
                Ok(hash) => {
                    info!(tx_hash = ?hash, "Transaction submitted successfully");
                    nonce_manager.update_tx_hash(nonce, hash).await;
                    hash
                }
                Err(e) => {
                    error!(error = %e, "Failed to submit transaction");
                    nonce_manager.release_nonce(nonce).await;
                    continue;
                }
            };

            // Monitor for confirmation (with timeout)
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(60),
                transaction_submitter.monitor_inclusion(submitted_hash, 60)
            ).await {
                Ok(Ok(exec_result)) => {
                    if exec_result.success {
                        stats.confirmed += 1;
                        info!(
                            tx_hash = ?submitted_hash,
                            block = ?exec_result.block_number,
                            "✅ Transaction confirmed!"
                        );
                        nonce_manager.confirm_nonce(nonce, submitted_hash).await;
                    } else {
                        warn!(tx_hash = ?submitted_hash, "Transaction reverted");
                    }

                    // Save execution result
                    if let Err(e) = self.db.save_execution(&exec_result).await {
                        warn!(error = %e, "Failed to save execution result");
                    }
                }
                Ok(Err(e)) => {
                    warn!(error = %e, "Error monitoring transaction");
                }
                Err(_) => {
                    warn!(tx_hash = ?submitted_hash, "Transaction confirmation timeout");
                }
            }

            // Log stats every 100 transactions
            if stats.processed % 100 == 0 {
                self.log_stats(&stats).await;
            }
        }

        warn!("Transaction receiver channel closed");
        monitor_handle.abort();
        nonce_sync_handle.abort();

        Ok(())
    }

    fn create_transaction_filter(&self) -> Result<TransactionFilter> {
        let min_value = U256::from_dec_str(&self.config.filters.min_value_wei)?;
        let min_gas_price = U256::from(self.config.filters.min_gas_price_gwei) * U256::exp10(9);
        let max_gas_price = U256::from(self.config.filters.max_gas_price_gwei) * U256::exp10(9);

        Ok(TransactionFilter::new(min_value, min_gas_price, max_gas_price))
    }

    fn create_profitability_calculator(&self) -> Result<ProfitabilityCalculator> {
        let min_profit = U256::from_dec_str(&self.config.profitability.min_profit_wei)?;

        Ok(ProfitabilityCalculator::new(
            min_profit,
            self.config.profitability.min_profit_percentage,
            self.config.profitability.max_gas_cost_percentage,
        ))
    }

    fn load_wallet(&self) -> Result<LocalWallet> {
        let private_key = &self.config.wallet.private_key;

        let wallet = LocalWallet::from_str(private_key)
            .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;

        Ok(wallet)
    }

    async fn log_stats(&self, stats: &ProcessingStats) {
        let profit_rate = (stats.profitable as f64 / stats.processed as f64) * 100.0;
        let success_rate = if stats.submitted > 0 {
            (stats.confirmed as f64 / stats.submitted as f64) * 100.0
        } else {
            0.0
        };

        info!(
            processed = stats.processed,
            simulated = stats.simulated,
            sim_successful = stats.sim_successful,
            profitable = stats.profitable,
            submitted = stats.submitted,
            confirmed = stats.confirmed,
            profit_rate = format!("{:.2}%", profit_rate),
            success_rate = format!("{:.2}%", success_rate),
            "📊 Processing Statistics"
        );

        // Update database stats
        let today = chrono::Utc::now().naive_utc().date();
        let _ = self.db.update_daily_stats(
            today,
            100,  // Last 100 processed
            stats.simulated as i64,
            stats.sim_successful as i64,
            stats.submitted as i64,
            stats.confirmed as i64,
            "0".to_string(),  // Would need to track actual values
            "0".to_string(),
            "0".to_string(),
        ).await;
    }
}

#[derive(Debug, Default)]
struct ProcessingStats {
    processed: u64,
    simulated: u64,
    sim_successful: u64,
    profitable: u64,
    submitted: u64,
    confirmed: u64,
}
