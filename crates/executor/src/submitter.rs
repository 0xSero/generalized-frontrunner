//! Transaction submission strategies

use ethers::prelude::*;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use types::{Error, ExecutionResult, Result, SubmissionPath};
use uuid::Uuid;

/// Transaction submission strategy
#[derive(Debug, Clone)]
pub enum SubmissionStrategy {
    /// Submit to public mempool only
    PublicOnly,
    /// Submit to Flashbots only
    FlashbotsOnly,
    /// Submit to both in parallel
    Parallel,
    /// Try Flashbots first, fallback to public
    FlashbotsWithFallback,
}

/// Transaction submitter
pub struct TransactionSubmitter {
    provider: Arc<Provider<Http>>,
    strategy: SubmissionStrategy,
}

impl TransactionSubmitter {
    /// Create a new transaction submitter
    pub fn new(provider: Arc<Provider<Http>>, strategy: SubmissionStrategy) -> Self {
        Self { provider, strategy }
    }

    /// Submit a raw signed transaction to public mempool
    pub async fn submit_to_public(&self, raw_tx: Bytes) -> Result<H256> {
        debug!(tx_len = raw_tx.len(), "Submitting transaction to public mempool");

        let pending_tx = self
            .provider
            .send_raw_transaction(raw_tx)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to submit to public mempool");
                Error::Execution(format!("Public submission failed: {}", e))
            })?;

        let tx_hash = *pending_tx;

        info!(tx_hash = ?tx_hash, "Transaction submitted to public mempool");

        Ok(tx_hash)
    }

    /// Wait for transaction confirmation
    pub async fn wait_for_confirmation(
        &self,
        tx_hash: H256,
        confirmations: usize,
    ) -> Result<Option<TransactionReceipt>> {
        debug!(
            tx_hash = ?tx_hash,
            confirmations = confirmations,
            "Waiting for transaction confirmation"
        );

        let receipt = self
            .provider
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(|e| Error::Provider(format!("Failed to get receipt: {}", e)))?;

        if let Some(receipt) = &receipt {
            info!(
                tx_hash = ?tx_hash,
                block_number = ?receipt.block_number,
                status = ?receipt.status,
                gas_used = ?receipt.gas_used,
                "Transaction confirmed"
            );
        }

        Ok(receipt)
    }

    /// Check if transaction was successful
    pub fn is_successful(receipt: &TransactionReceipt) -> bool {
        receipt.status == Some(U64::from(1))
    }

    /// Submit with retry logic
    pub async fn submit_with_retry(
        &self,
        raw_tx: Bytes,
        max_attempts: u32,
    ) -> Result<H256> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < max_attempts {
            attempts += 1;

            match self.submit_to_public(raw_tx.clone()).await {
                Ok(tx_hash) => return Ok(tx_hash),
                Err(e) => {
                    warn!(
                        attempt = attempts,
                        max_attempts = max_attempts,
                        error = %e,
                        "Submission attempt failed"
                    );
                    last_error = Some(e);

                    if attempts < max_attempts {
                        // Exponential backoff
                        let delay = std::time::Duration::from_millis(2u64.pow(attempts) * 1000);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::Execution("All attempts failed".to_string())))
    }

    /// Monitor transaction inclusion
    pub async fn monitor_inclusion(
        &self,
        tx_hash: H256,
        timeout_secs: u64,
    ) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        loop {
            if start_time.elapsed() > timeout {
                return Err(Error::Timeout(timeout_secs * 1000));
            }

            match self.provider.get_transaction_receipt(tx_hash).await {
                Ok(Some(receipt)) => {
                    let inclusion_time_ms = start_time.elapsed().as_millis() as u64;
                    let success = Self::is_successful(&receipt);

                    let mut result = ExecutionResult::new(
                        Uuid::new_v4(), // TODO: Link to actual simulation ID
                        tx_hash,
                        SubmissionPath::PublicMempool,
                    );

                    if success {
                        result = result.with_confirmation(
                            receipt.block_number.unwrap_or_default().as_u64(),
                            receipt.gas_used.unwrap_or_default().as_u64(),
                            receipt.effective_gas_price.unwrap_or_default(),
                            U256::zero(), // TODO: Calculate actual profit
                            inclusion_time_ms,
                        );
                    } else {
                        result = result.with_error("Transaction reverted".to_string());
                    }

                    return Ok(result);
                }
                Ok(None) => {
                    // Not yet included, wait and retry
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
                Err(e) => {
                    warn!(error = %e, "Error checking transaction receipt");
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submission_strategy() {
        let strategy = SubmissionStrategy::Parallel;
        match strategy {
            SubmissionStrategy::Parallel => {}
            _ => panic!("Expected Parallel strategy"),
        }
    }
}
