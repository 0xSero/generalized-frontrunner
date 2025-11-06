//! RPC provider abstraction and connection management

use async_trait::async_trait;
use ethers::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use types::{Error, Result};

/// RPC provider trait for abstracting different providers
#[async_trait]
pub trait RpcProvider: Send + Sync {
    /// Connect to the RPC provider
    async fn connect(&self) -> Result<()>;

    /// Check if the provider is connected
    async fn is_connected(&self) -> bool;

    /// Get the WebSocket provider
    async fn get_provider(&self) -> Result<Arc<Provider<Ws>>>;

    /// Reconnect to the provider
    async fn reconnect(&self) -> Result<()>;
}

/// Ethereum RPC provider with automatic reconnection
pub struct EthereumRpcProvider {
    ws_url: String,
    provider: Arc<RwLock<Option<Arc<Provider<Ws>>>>>,
    max_reconnect_attempts: u32,
    reconnect_delay_ms: u64,
}

impl EthereumRpcProvider {
    /// Create a new RPC provider
    pub fn new(ws_url: String, max_reconnect_attempts: u32) -> Self {
        Self {
            ws_url,
            provider: Arc::new(RwLock::new(None)),
            max_reconnect_attempts,
            reconnect_delay_ms: 1000,
        }
    }

    /// Connect to the WebSocket endpoint with retry logic
    async fn connect_with_retry(&self) -> Result<Arc<Provider<Ws>>> {
        let mut attempts = 0;
        let mut delay = self.reconnect_delay_ms;

        loop {
            attempts += 1;

            match Ws::connect(&self.ws_url).await {
                Ok(ws) => {
                    let provider = Arc::new(Provider::new(ws));

                    // Test the connection
                    match provider.get_block_number().await {
                        Ok(block_number) => {
                            info!(
                                attempts = attempts,
                                block_number = %block_number,
                                "Successfully connected to RPC provider"
                            );
                            return Ok(provider);
                        }
                        Err(e) => {
                            warn!(
                                error = %e,
                                attempt = attempts,
                                "Connection test failed"
                            );
                        }
                    }
                }
                Err(e) => {
                    error!(
                        error = %e,
                        attempt = attempts,
                        max_attempts = self.max_reconnect_attempts,
                        "Failed to connect to RPC provider"
                    );
                }
            }

            if attempts >= self.max_reconnect_attempts {
                return Err(Error::Rpc(format!(
                    "Failed to connect after {} attempts",
                    attempts
                )));
            }

            // Exponential backoff
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            delay = (delay * 2).min(30000); // Cap at 30 seconds
        }
    }
}

#[async_trait]
impl RpcProvider for EthereumRpcProvider {
    async fn connect(&self) -> Result<()> {
        let provider = self.connect_with_retry().await?;
        let mut provider_lock = self.provider.write().await;
        *provider_lock = Some(provider);
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        let provider_lock = self.provider.read().await;
        provider_lock.is_some()
    }

    async fn get_provider(&self) -> Result<Arc<Provider<Ws>>> {
        let provider_lock = self.provider.read().await;
        provider_lock
            .as_ref()
            .cloned()
            .ok_or_else(|| Error::Rpc("Provider not connected".to_string()))
    }

    async fn reconnect(&self) -> Result<()> {
        warn!("Reconnecting to RPC provider...");

        // Clear the existing provider
        {
            let mut provider_lock = self.provider.write().await;
            *provider_lock = None;
        }

        // Connect with retry logic
        self.connect().await
    }
}

/// Provider manager that handles failover between multiple providers
pub struct ProviderManager {
    primary: Box<dyn RpcProvider>,
    backup: Option<Box<dyn RpcProvider>>,
    current_provider: Arc<RwLock<ProviderType>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderType {
    Primary,
    Backup,
}

impl ProviderManager {
    /// Create a new provider manager
    pub fn new(
        primary: Box<dyn RpcProvider>,
        backup: Option<Box<dyn RpcProvider>>,
    ) -> Self {
        Self {
            primary,
            backup,
            current_provider: Arc::new(RwLock::new(ProviderType::Primary)),
        }
    }

    /// Get the active provider
    pub async fn get_provider(&self) -> Result<Arc<Provider<Ws>>> {
        let provider_type = *self.current_provider.read().await;

        match provider_type {
            ProviderType::Primary => {
                if self.primary.is_connected().await {
                    self.primary.get_provider().await
                } else {
                    // Try to reconnect primary
                    match self.primary.reconnect().await {
                        Ok(_) => self.primary.get_provider().await,
                        Err(e) => {
                            error!(error = %e, "Failed to reconnect primary provider");
                            self.failover_to_backup().await
                        }
                    }
                }
            }
            ProviderType::Backup => {
                if let Some(backup) = &self.backup {
                    if backup.is_connected().await {
                        backup.get_provider().await
                    } else {
                        // Try to reconnect backup
                        match backup.reconnect().await {
                            Ok(_) => backup.get_provider().await,
                            Err(e) => {
                                error!(error = %e, "Failed to reconnect backup provider");
                                Err(e)
                            }
                        }
                    }
                } else {
                    Err(Error::Rpc("No backup provider configured".to_string()))
                }
            }
        }
    }

    /// Failover to backup provider
    async fn failover_to_backup(&self) -> Result<Arc<Provider<Ws>>> {
        if let Some(backup) = &self.backup {
            warn!("Failing over to backup provider");

            if !backup.is_connected().await {
                backup.connect().await?;
            }

            let mut current = self.current_provider.write().await;
            *current = ProviderType::Backup;

            backup.get_provider().await
        } else {
            Err(Error::Rpc("No backup provider available for failover".to_string()))
        }
    }

    /// Connect both primary and backup providers
    pub async fn connect_all(&self) -> Result<()> {
        self.primary.connect().await?;

        if let Some(backup) = &self.backup {
            if let Err(e) = backup.connect().await {
                warn!(error = %e, "Failed to connect backup provider");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_creation() {
        let provider = EthereumRpcProvider::new(
            "wss://eth-mainnet.g.alchemy.com/v2/test".to_string(),
            3,
        );
        assert!(!provider.is_connected().await);
    }
}
