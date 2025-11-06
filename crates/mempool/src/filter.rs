//! Transaction filtering logic

use ethers::types::{Transaction, U256};
use tracing::debug;
use types::Result;

/// Transaction filter configuration and logic
#[derive(Debug, Clone)]
pub struct TransactionFilter {
    pub min_value: U256,
    pub min_gas_price: U256,
    pub max_gas_price: U256,
}

impl TransactionFilter {
    /// Create a new transaction filter
    pub fn new(min_value: U256, min_gas_price: U256, max_gas_price: U256) -> Self {
        Self {
            min_value,
            min_gas_price,
            max_gas_price,
        }
    }

    /// Check if a transaction passes all filters
    pub fn should_process(&self, tx: &Transaction) -> bool {
        // Filter by value
        if tx.value < self.min_value {
            debug!(
                tx_hash = ?tx.hash,
                value = %tx.value,
                min_value = %self.min_value,
                "Transaction filtered: value too low"
            );
            return false;
        }

        // Get effective gas price
        let gas_price = tx.gas_price.unwrap_or(
            tx.max_fee_per_gas.unwrap_or_default()
        );

        // Filter by gas price (too low)
        if gas_price < self.min_gas_price {
            debug!(
                tx_hash = ?tx.hash,
                gas_price = %gas_price,
                min_gas_price = %self.min_gas_price,
                "Transaction filtered: gas price too low"
            );
            return false;
        }

        // Filter by gas price (too high - indicates spam or mistakes)
        if gas_price > self.max_gas_price {
            debug!(
                tx_hash = ?tx.hash,
                gas_price = %gas_price,
                max_gas_price = %self.max_gas_price,
                "Transaction filtered: gas price too high"
            );
            return false;
        }

        // Transaction passed all filters
        true
    }

    /// Check if transaction is a contract interaction
    pub fn is_contract_interaction(tx: &Transaction) -> bool {
        tx.to.is_some() && !tx.input.0.is_empty()
    }

    /// Check if transaction is a contract deployment
    pub fn is_contract_deployment(tx: &Transaction) -> bool {
        tx.to.is_none() && !tx.input.0.is_empty()
    }

    /// Extract function selector from transaction data
    pub fn extract_function_selector(tx: &Transaction) -> Option<[u8; 4]> {
        if tx.input.0.len() >= 4 {
            let mut selector = [0u8; 4];
            selector.copy_from_slice(&tx.input.0[..4]);
            Some(selector)
        } else {
            None
        }
    }
}

impl Default for TransactionFilter {
    fn default() -> Self {
        Self {
            min_value: U256::from(100_000_000_000_000_000u64), // 0.1 ETH
            min_gas_price: U256::from(10_000_000_000u64),       // 10 gwei
            max_gas_price: U256::from(500_000_000_000u64),      // 500 gwei
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::types::{Address, Bytes, H256, U64};

    fn create_test_transaction(value: u64, gas_price: u64) -> Transaction {
        Transaction {
            hash: H256::random(),
            nonce: U64::zero(),
            block_hash: None,
            block_number: None,
            transaction_index: None,
            from: Address::random(),
            to: Some(Address::random()),
            value: U256::from(value),
            gas_price: Some(U256::from(gas_price)),
            gas: U256::from(21000),
            input: Bytes::default(),
            v: U64::zero(),
            r: U256::zero(),
            s: U256::zero(),
            transaction_type: None,
            access_list: None,
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            chain_id: None,
            other: Default::default(),
        }
    }

    #[test]
    fn test_filter_low_value() {
        let filter = TransactionFilter::default();
        let tx = create_test_transaction(
            10_000_000_000_000_000,   // 0.01 ETH (below 0.1 ETH threshold)
            50_000_000_000,            // 50 gwei
        );

        assert!(!filter.should_process(&tx));
    }

    #[test]
    fn test_filter_pass() {
        let filter = TransactionFilter::default();
        let tx = create_test_transaction(
            500_000_000_000_000_000,  // 0.5 ETH
            50_000_000_000,            // 50 gwei
        );

        assert!(filter.should_process(&tx));
    }

    #[test]
    fn test_filter_gas_too_high() {
        let filter = TransactionFilter::default();
        let tx = create_test_transaction(
            500_000_000_000_000_000,  // 0.5 ETH
            1_000_000_000_000,         // 1000 gwei (above 500 gwei threshold)
        );

        assert!(!filter.should_process(&tx));
    }
}
