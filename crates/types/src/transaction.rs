//! Transaction types and structures

use ethers::types::{Address, Bytes, H256, U256, U64};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub hash: H256,
    pub from: Address,
    pub to: Option<Address>,
    pub value: U256,
    pub gas_price: Option<U256>,
    pub max_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,
    pub gas_limit: U256,
    pub nonce: U64,
    pub data: Bytes,
    pub status: TransactionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Analyzing,
    Simulated,
    Profitable,
    Submitted,
    Confirmed,
    Failed,
    Rejected,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Analyzing => write!(f, "analyzing"),
            Self::Simulated => write!(f, "simulated"),
            Self::Profitable => write!(f, "profitable"),
            Self::Submitted => write!(f, "submitted"),
            Self::Confirmed => write!(f, "confirmed"),
            Self::Failed => write!(f, "failed"),
            Self::Rejected => write!(f, "rejected"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTransaction {
    pub transaction: Transaction,
    pub function_selector: Option<[u8; 4]>,
    pub decoded_input: Option<DecodedInput>,
    pub contract_addresses: Vec<Address>,
    pub token_addresses: Vec<Address>,
    pub transaction_type: TransactionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedInput {
    pub function_name: String,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionType {
    Unknown,
    DexSwap,
    NftMint,
    NftTransfer,
    TokenTransfer,
    ContractDeployment,
    ContractInteraction,
}

impl Transaction {
    pub fn new(tx: ethers::types::Transaction) -> Self {
        Self {
            id: Uuid::new_v4(),
            hash: tx.hash,
            from: tx.from,
            to: tx.to,
            value: tx.value,
            gas_price: tx.gas_price,
            max_fee_per_gas: tx.max_fee_per_gas,
            max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
            gas_limit: tx.gas,
            nonce: tx.nonce,
            data: tx.input,
            status: TransactionStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn is_eip1559(&self) -> bool {
        self.max_fee_per_gas.is_some() && self.max_priority_fee_per_gas.is_some()
    }

    pub fn effective_gas_price(&self, base_fee: U256) -> U256 {
        if let (Some(max_fee), Some(priority_fee)) = (self.max_fee_per_gas, self.max_priority_fee_per_gas) {
            // EIP-1559
            let max_priority = std::cmp::min(priority_fee, max_fee.saturating_sub(base_fee));
            base_fee + max_priority
        } else {
            // Legacy
            self.gas_price.unwrap_or_default()
        }
    }

    /// Create a modified copy of this transaction for frontrunning
    pub fn create_frontrun_copy(
        &self,
        new_from: Address,
        new_data: Bytes,
        new_nonce: U64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            hash: H256::zero(), // Will be set when signed
            from: new_from,
            to: self.to,
            value: self.value,
            gas_price: self.gas_price,
            max_fee_per_gas: self.max_fee_per_gas,
            max_priority_fee_per_gas: self.max_priority_fee_per_gas,
            gas_limit: self.gas_limit,
            nonce: new_nonce,
            data: new_data,
            status: TransactionStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
