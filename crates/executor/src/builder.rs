//! Transaction building and signing

use ethers::prelude::*;
use ethers::signers::{LocalWallet, Signer};
use std::sync::Arc;
use tracing::{debug, info};
use types::{Error, Result};

/// Transaction builder for creating and signing transactions
pub struct TransactionBuilder {
    wallet: Arc<LocalWallet>,
    chain_id: u64,
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new(wallet: Arc<LocalWallet>, chain_id: u64) -> Self {
        Self { wallet, chain_id }
    }

    /// Build an EIP-1559 transaction
    pub async fn build_eip1559(
        &self,
        to: Address,
        data: Bytes,
        value: U256,
        gas_limit: U256,
        max_fee_per_gas: U256,
        max_priority_fee_per_gas: U256,
        nonce: U256,
    ) -> Result<TypedTransaction> {
        let tx = Eip1559TransactionRequest::new()
            .to(to)
            .data(data)
            .value(value)
            .gas(gas_limit)
            .max_fee_per_gas(max_fee_per_gas)
            .max_priority_fee_per_gas(max_priority_fee_per_gas)
            .nonce(nonce)
            .chain_id(self.chain_id);

        Ok(TypedTransaction::Eip1559(tx))
    }

    /// Build a legacy transaction
    pub async fn build_legacy(
        &self,
        to: Address,
        data: Bytes,
        value: U256,
        gas_limit: U256,
        gas_price: U256,
        nonce: U256,
    ) -> Result<TypedTransaction> {
        let tx = TransactionRequest::new()
            .to(to)
            .data(data)
            .value(value)
            .gas(gas_limit)
            .gas_price(gas_price)
            .nonce(nonce)
            .chain_id(self.chain_id);

        Ok(TypedTransaction::Legacy(tx))
    }

    /// Sign a transaction
    pub async fn sign(&self, tx: &TypedTransaction) -> Result<Signature> {
        let signature = self
            .wallet
            .sign_transaction(tx)
            .await
            .map_err(|e| Error::Execution(format!("Failed to sign transaction: {}", e)))?;

        debug!(
            r = ?signature.r,
            s = ?signature.s,
            v = signature.v,
            "Transaction signed"
        );

        Ok(signature)
    }

    /// Build and sign a transaction
    pub async fn build_and_sign(
        &self,
        to: Address,
        data: Bytes,
        value: U256,
        gas_limit: U256,
        max_fee_per_gas: U256,
        max_priority_fee_per_gas: U256,
        nonce: U256,
    ) -> Result<Bytes> {
        let tx = self
            .build_eip1559(
                to,
                data,
                value,
                gas_limit,
                max_fee_per_gas,
                max_priority_fee_per_gas,
                nonce,
            )
            .await?;

        let signature = self.sign(&tx).await?;

        // Encode signed transaction
        let signed_tx = tx.rlp_signed(&signature);

        info!(
            to = ?to,
            value = %value,
            gas_limit = %gas_limit,
            max_fee = %max_fee_per_gas,
            "Transaction built and signed"
        );

        Ok(signed_tx)
    }

    /// Calculate optimal gas prices for frontrunning
    pub fn calculate_frontrun_gas(
        &self,
        base_fee: U256,
        target_priority_fee: U256,
        priority_multiplier: f64,
        max_fee_multiplier: f64,
    ) -> (U256, U256) {
        // Increase priority fee to frontrun
        let multiplier_bp = (priority_multiplier * 10000.0) as u128;
        let new_priority_fee = target_priority_fee
            .checked_mul(U256::from(multiplier_bp))
            .unwrap_or(target_priority_fee)
            / U256::from(10000u64);

        // Calculate max fee
        let max_fee_multiplier_bp = (max_fee_multiplier * 10000.0) as u128;
        let new_max_fee = (base_fee + new_priority_fee)
            .checked_mul(U256::from(max_fee_multiplier_bp))
            .unwrap_or(base_fee + new_priority_fee)
            / U256::from(10000u64);

        (new_max_fee, new_priority_fee)
    }

    /// Get the wallet address
    pub fn address(&self) -> Address {
        self.wallet.address()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_builder_creation() {
        let wallet = LocalWallet::new(&mut rand::thread_rng());
        let builder = TransactionBuilder::new(Arc::new(wallet.clone()), 1);

        assert_eq!(builder.address(), wallet.address());
    }

    #[test]
    fn test_gas_calculation() {
        let wallet = LocalWallet::new(&mut rand::thread_rng());
        let builder = TransactionBuilder::new(Arc::new(wallet), 1);

        let base_fee = U256::from(50_000_000_000u64); // 50 gwei
        let priority_fee = U256::from(2_000_000_000u64); // 2 gwei

        let (max_fee, new_priority) = builder.calculate_frontrun_gas(
            base_fee,
            priority_fee,
            1.2,  // 20% higher priority
            1.5,  // 50% higher max fee
        );

        // Priority should be 2.4 gwei
        assert_eq!(new_priority, U256::from(2_400_000_000u64));

        // Max fee should be (50 + 2.4) * 1.5 = 78.6 gwei
        assert_eq!(max_fee, U256::from(78_600_000_000u64));
    }
}
