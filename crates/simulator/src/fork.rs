//! State forking for simulation

use ethers::prelude::*;
use revm::{
    primitives::{AccountInfo, Address as RevmAddress, Bytecode, B256, U256 as RevmU256},
    Database, DatabaseRef,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error};
use types::{Error, Result};

/// State fork that uses an Ethereum provider as the backend
pub struct StateFork {
    provider: Arc<Provider<Http>>,
    block_number: u64,
    // Cache for frequently accessed data
    account_cache: HashMap<RevmAddress, AccountInfo>,
    storage_cache: HashMap<(RevmAddress, RevmU256), RevmU256>,
}

impl StateFork {
    /// Create a new state fork at a specific block
    pub async fn new(provider: Arc<Provider<Http>>, block_number: u64) -> Result<Self> {
        debug!(block_number = block_number, "Creating state fork");

        Ok(Self {
            provider,
            block_number,
            account_cache: HashMap::new(),
            storage_cache: HashMap::new(),
        })
    }

    /// Create a fork at the latest block
    pub async fn new_latest(provider: Arc<Provider<Http>>) -> Result<Self> {
        let block_number = provider
            .get_block_number()
            .await
            .map_err(|e| Error::Provider(e.to_string()))?
            .as_u64();

        Self::new(provider, block_number).await
    }

    /// Convert ethers Address to revm Address
    fn to_revm_address(addr: ethers::types::Address) -> RevmAddress {
        RevmAddress::from_slice(addr.as_bytes())
    }

    /// Convert revm Address to ethers Address
    fn to_ethers_address(addr: RevmAddress) -> ethers::types::Address {
        ethers::types::Address::from_slice(addr.as_slice())
    }

    /// Convert ethers U256 to revm U256
    fn to_revm_u256(value: ethers::types::U256) -> RevmU256 {
        let mut bytes = [0u8; 32];
        value.to_big_endian(&mut bytes);
        RevmU256::from_be_bytes(bytes)
    }

    /// Convert revm U256 to ethers U256
    fn to_ethers_u256(value: RevmU256) -> ethers::types::U256 {
        ethers::types::U256::from_big_endian(&value.to_be_bytes::<32>())
    }

    /// Fetch account info from the provider
    async fn fetch_account_info(&self, address: RevmAddress) -> Result<AccountInfo> {
        let ethers_addr = Self::to_ethers_address(address);
        let block_id = Some(BlockId::Number(self.block_number.into()));

        // Fetch account data in parallel
        let (balance, nonce, code) = tokio::try_join!(
            self.provider.get_balance(ethers_addr, block_id),
            self.provider.get_transaction_count(ethers_addr, block_id),
            self.provider.get_code(ethers_addr, block_id),
        )
        .map_err(|e| Error::Provider(e.to_string()))?;

        let code_hash = if code.is_empty() {
            B256::ZERO
        } else {
            B256::from_slice(&ethers::utils::keccak256(&code))
        };

        Ok(AccountInfo {
            balance: Self::to_revm_u256(balance),
            nonce: nonce.as_u64(),
            code_hash,
            code: Some(Bytecode::new_raw(code.0.into())),
        })
    }

    /// Fetch storage value from the provider
    async fn fetch_storage(
        &self,
        address: RevmAddress,
        index: RevmU256,
    ) -> Result<RevmU256> {
        let ethers_addr = Self::to_ethers_address(address);
        let ethers_index = Self::to_ethers_u256(index);
        let block_id = Some(BlockId::Number(self.block_number.into()));

        let value = self
            .provider
            .get_storage_at(ethers_addr, ethers_index, block_id)
            .await
            .map_err(|e| Error::Provider(e.to_string()))?;

        Ok(Self::to_revm_u256(value))
    }
}

// Implement REVM Database trait
impl Database for StateFork {
    type Error = Error;

    fn basic(&mut self, address: RevmAddress) -> Result<Option<AccountInfo>> {
        // Check cache first
        if let Some(account) = self.account_cache.get(&address) {
            return Ok(Some(account.clone()));
        }

        // Fetch from provider (blocking)
        let account = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { self.fetch_account_info(address).await })
        })?;

        // Cache the result
        self.account_cache.insert(address, account.clone());

        Ok(Some(account))
    }

    fn code_by_hash(&mut self, _code_hash: B256) -> Result<Bytecode> {
        // This is typically cached in AccountInfo
        Ok(Bytecode::default())
    }

    fn storage(&mut self, address: RevmAddress, index: RevmU256) -> Result<RevmU256> {
        let key = (address, index);

        // Check cache first
        if let Some(value) = self.storage_cache.get(&key) {
            return Ok(*value);
        }

        // Fetch from provider (blocking)
        let value = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { self.fetch_storage(address, index).await })
        })?;

        // Cache the result
        self.storage_cache.insert(key, value);

        Ok(value)
    }

    fn block_hash(&mut self, number: RevmU256) -> Result<B256> {
        let block_num = number.to::<u64>();

        // Fetch block hash (blocking)
        let hash = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let block = self
                    .provider
                    .get_block(block_num)
                    .await
                    .map_err(|e| Error::Provider(e.to_string()))?;

                Ok::<_, Error>(
                    block
                        .and_then(|b| b.hash)
                        .ok_or_else(|| Error::Provider("Block not found".to_string()))?,
                )
            })
        })?;

        Ok(B256::from_slice(hash.as_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_conversion() {
        let ethers_addr = ethers::types::Address::random();
        let revm_addr = StateFork::to_revm_address(ethers_addr);
        let converted_back = StateFork::to_ethers_address(revm_addr);
        assert_eq!(ethers_addr, converted_back);
    }

    #[test]
    fn test_u256_conversion() {
        let ethers_value = ethers::types::U256::from(123456789u64);
        let revm_value = StateFork::to_revm_u256(ethers_value);
        let converted_back = StateFork::to_ethers_u256(revm_value);
        assert_eq!(ethers_value, converted_back);
    }
}
