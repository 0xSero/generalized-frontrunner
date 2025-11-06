//! Mempool monitoring service

use ethers::prelude::*;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use types::{Error, Result};

use crate::filter::TransactionFilter;
use crate::provider::ProviderManager;

/// Mempool monitor that subscribes to pending transactions
pub struct MempoolMonitor {
    provider_manager: Arc<ProviderManager>,
    filter: TransactionFilter,
    tx_sender: mpsc::UnboundedSender<Transaction>,
}

impl MempoolMonitor {
    /// Create a new mempool monitor
    pub fn new(
        provider_manager: Arc<ProviderManager>,
        filter: TransactionFilter,
    ) -> (Self, mpsc::UnboundedReceiver<Transaction>) {
        let (tx_sender, tx_receiver) = mpsc::unbounded_channel();

        let monitor = Self {
            provider_manager,
            filter,
            tx_sender,
        };

        (monitor, tx_receiver)
    }

    /// Start monitoring the mempool
    pub async fn start(&self) -> Result<()> {
        info!("Starting mempool monitor");

        loop {
            match self.run_monitor_loop().await {
                Ok(_) => {
                    warn!("Monitor loop exited normally, restarting...");
                }
                Err(e) => {
                    error!(error = %e, "Monitor loop error, restarting...");
                }
            }

            // Wait before reconnecting
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    /// Run the monitoring loop
    async fn run_monitor_loop(&self) -> Result<()> {
        let provider = self.provider_manager.get_provider().await?;

        info!("Subscribing to pending transactions");

        // Subscribe to pending transactions
        let mut stream = match provider.subscribe_pending_txs().await {
            Ok(stream) => stream,
            Err(e) => {
                return Err(Error::Provider(format!(
                    "Failed to subscribe to pending transactions: {}",
                    e
                )));
            }
        };

        let mut tx_count = 0u64;
        let mut filtered_count = 0u64;
        let start_time = std::time::Instant::now();

        // Process incoming transaction hashes
        while let Some(tx_hash) = stream.next().await {
            tx_count += 1;

            // Fetch full transaction details
            match provider.get_transaction(tx_hash).await {
                Ok(Some(tx)) => {
                    // Apply filters
                    if self.filter.should_process(&tx) {
                        // Send to processing queue
                        if let Err(e) = self.tx_sender.send(tx.clone()) {
                            error!(error = %e, "Failed to send transaction to queue");
                            return Err(Error::Internal(
                                "Transaction queue receiver dropped".to_string()
                            ));
                        }

                        debug!(
                            tx_hash = ?tx.hash,
                            from = ?tx.from,
                            to = ?tx.to,
                            value = %tx.value,
                            "Transaction queued for processing"
                        );
                    } else {
                        filtered_count += 1;
                    }
                }
                Ok(None) => {
                    debug!(tx_hash = ?tx_hash, "Transaction not found");
                }
                Err(e) => {
                    warn!(
                        tx_hash = ?tx_hash,
                        error = %e,
                        "Failed to fetch transaction details"
                    );
                }
            }

            // Log stats every 1000 transactions
            if tx_count % 1000 == 0 {
                let elapsed = start_time.elapsed();
                let tx_per_sec = tx_count as f64 / elapsed.as_secs_f64();
                let pass_rate = ((tx_count - filtered_count) as f64 / tx_count as f64) * 100.0;

                info!(
                    tx_count = tx_count,
                    filtered_count = filtered_count,
                    tx_per_sec = format!("{:.2}", tx_per_sec),
                    pass_rate = format!("{:.2}%", pass_rate),
                    "Mempool monitor stats"
                );
            }
        }

        warn!("Pending transaction stream ended");
        Ok(())
    }

    /// Get statistics about the monitor
    pub fn get_stats(&self) -> MonitorStats {
        MonitorStats {
            queue_depth: 0, // TODO: Track queue depth
        }
    }
}

#[derive(Debug, Clone)]
pub struct MonitorStats {
    pub queue_depth: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::EthereumRpcProvider;

    #[tokio::test]
    async fn test_monitor_creation() {
        let primary = Box::new(EthereumRpcProvider::new(
            "wss://eth-mainnet.g.alchemy.com/v2/test".to_string(),
            3,
        ));
        let provider_manager = Arc::new(ProviderManager::new(primary, None));
        let filter = TransactionFilter::default();

        let (monitor, _rx) = MempoolMonitor::new(provider_manager, filter);
        let stats = monitor.get_stats();

        assert_eq!(stats.queue_depth, 0);
    }
}
