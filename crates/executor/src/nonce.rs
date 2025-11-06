//! Nonce management system

use ethers::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use types::{Error, Result};

/// Nonce manager for tracking and managing transaction nonces
pub struct NonceManager {
    provider: Arc<Provider<Http>>,
    address: Address,
    /// Current nonce (in-memory cache)
    current_nonce: Arc<RwLock<U256>>,
    /// Pending nonces (awaiting confirmation)
    pending_nonces: Arc<RwLock<HashMap<U256, PendingNonce>>>,
}

#[derive(Debug, Clone)]
struct PendingNonce {
    tx_hash: H256,
    timestamp: std::time::Instant,
}

impl NonceManager {
    /// Create a new nonce manager
    pub async fn new(provider: Arc<Provider<Http>>, address: Address) -> Result<Self> {
        // Fetch initial nonce from the network
        let initial_nonce = provider
            .get_transaction_count(address, None)
            .await
            .map_err(|e| Error::Provider(format!("Failed to get initial nonce: {}", e)))?;

        info!(
            address = ?address,
            nonce = %initial_nonce,
            "Nonce manager initialized"
        );

        Ok(Self {
            provider,
            address,
            current_nonce: Arc::new(RwLock::new(initial_nonce)),
            pending_nonces: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get the next available nonce
    pub async fn get_next_nonce(&self) -> Result<U256> {
        let mut nonce = self.current_nonce.write().await;
        let next = *nonce;
        *nonce += U256::one();

        debug!(nonce = %next, "Allocated nonce");

        // Track as pending
        let mut pending = self.pending_nonces.write().await;
        pending.insert(
            next,
            PendingNonce {
                tx_hash: H256::zero(), // Will be updated when tx is submitted
                timestamp: std::time::Instant::now(),
            },
        );

        Ok(next)
    }

    /// Mark a nonce as confirmed
    pub async fn confirm_nonce(&self, nonce: U256, tx_hash: H256) {
        let mut pending = self.pending_nonces.write().await;

        if let Some(pending_tx) = pending.remove(&nonce) {
            let duration = pending_tx.timestamp.elapsed();
            info!(
                nonce = %nonce,
                tx_hash = ?tx_hash,
                duration_ms = duration.as_millis(),
                "Nonce confirmed"
            );
        }

        // Clean up old pending nonces (older than 5 minutes)
        let cutoff = std::time::Instant::now() - std::time::Duration::from_secs(300);
        pending.retain(|_, v| v.timestamp > cutoff);
    }

    /// Mark a nonce as failed and release it
    pub async fn release_nonce(&self, nonce: U256) {
        warn!(nonce = %nonce, "Releasing failed nonce");

        let mut pending = self.pending_nonces.write().await;
        pending.remove(&nonce);

        // Reset current nonce if this was the last allocated
        let mut current = self.current_nonce.write().await;
        if nonce == *current - U256::one() {
            *current = nonce;
        }
    }

    /// Sync nonce with the network (call periodically or after errors)
    pub async fn sync_with_network(&self) -> Result<()> {
        let network_nonce = self
            .provider
            .get_transaction_count(self.address, None)
            .await
            .map_err(|e| Error::Provider(format!("Failed to sync nonce: {}", e)))?;

        let mut current = self.current_nonce.write().await;
        let old_nonce = *current;

        // Update to network nonce if it's higher
        if network_nonce > *current {
            *current = network_nonce;
            info!(
                old_nonce = %old_nonce,
                new_nonce = %network_nonce,
                "Nonce synced with network"
            );
        }

        // Clear outdated pending nonces
        let mut pending = self.pending_nonces.write().await;
        pending.retain(|n, _| *n >= network_nonce);

        Ok(())
    }

    /// Get current nonce (without incrementing)
    pub async fn get_current_nonce(&self) -> U256 {
        *self.current_nonce.read().await
    }

    /// Get number of pending transactions
    pub async fn pending_count(&self) -> usize {
        self.pending_nonces.read().await.len()
    }

    /// Update the transaction hash for a pending nonce
    pub async fn update_tx_hash(&self, nonce: U256, tx_hash: H256) {
        let mut pending = self.pending_nonces.write().await;
        if let Some(pending_tx) = pending.get_mut(&nonce) {
            pending_tx.tx_hash = tx_hash;
            debug!(nonce = %nonce, tx_hash = ?tx_hash, "Updated nonce tx_hash");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nonce_allocation() {
        // This test would require a mock provider
        // Skipping for now, but structure is in place
    }

    #[test]
    fn test_pending_nonce() {
        let pending = PendingNonce {
            tx_hash: H256::zero(),
            timestamp: std::time::Instant::now(),
        };

        assert_eq!(pending.tx_hash, H256::zero());
    }
}
