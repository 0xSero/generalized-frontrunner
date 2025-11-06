//! Transaction analysis and address extraction

use ethers::abi::{decode, ParamType, Token};
use ethers::types::{Address, Bytes, Transaction};
use std::collections::HashSet;
use tracing::{debug, warn};
use types::{DecodedInput, Parameter, ParsedTransaction, TransactionType, Result};

/// Transaction analyzer for extracting addresses and decoding calldata
pub struct TransactionAnalyzer {
    /// Known function selectors for common DEX operations
    known_selectors: std::collections::HashMap<[u8; 4], String>,
}

impl TransactionAnalyzer {
    /// Create a new transaction analyzer
    pub fn new() -> Self {
        let mut known_selectors = std::collections::HashMap::new();

        // Uniswap V2 selectors
        known_selectors.insert([0x38, 0xed, 0x17, 0x39], "swapExactTokensForTokens".to_string());
        known_selectors.insert([0x7f, 0xf3, 0x6a, 0xb5], "swapExactETHForTokens".to_string());
        known_selectors.insert([0x18, 0xcb, 0xaf, 0xe5], "swapExactTokensForETH".to_string());

        // Uniswap V3 selectors
        known_selectors.insert([0x41, 0x4b, 0xf3, 0x89], "exactInputSingle".to_string());
        known_selectors.insert([0xc0, 0x4b, 0x8d, 0x59], "exactInput".to_string());

        // ERC20 selectors
        known_selectors.insert([0xa9, 0x05, 0x9c, 0xbb], "transfer".to_string());
        known_selectors.insert([0x23, 0xb8, 0x72, 0xdd], "transferFrom".to_string());
        known_selectors.insert([0x09, 0x5e, 0xa7, 0xb3], "approve".to_string());

        // ERC721 selectors
        known_selectors.insert([0x42, 0x84, 0x2e, 0x0e], "safeTransferFrom".to_string());
        known_selectors.insert([0xb8, 0x8d, 0x4f, 0xde], "safeTransferFrom".to_string()); // with data

        Self { known_selectors }
    }

    /// Parse a transaction and extract relevant information
    pub fn parse(&self, tx: Transaction) -> Result<ParsedTransaction> {
        let mut frontrunner_tx = types::Transaction::new(tx.clone());

        // Extract function selector
        let function_selector = self.extract_function_selector(&tx);

        // Classify transaction type
        let transaction_type = self.classify_transaction(&tx);

        // Extract addresses from calldata
        let addresses = self.extract_addresses_from_calldata(&tx);

        // Try to decode input
        let decoded_input = self.try_decode_input(&tx);

        Ok(ParsedTransaction {
            transaction: frontrunner_tx,
            function_selector,
            decoded_input,
            contract_addresses: addresses.contract_addresses,
            token_addresses: addresses.token_addresses,
            transaction_type,
        })
    }

    /// Extract function selector (first 4 bytes of calldata)
    fn extract_function_selector(&self, tx: &Transaction) -> Option<[u8; 4]> {
        if tx.input.0.len() >= 4 {
            let mut selector = [0u8; 4];
            selector.copy_from_slice(&tx.input.0[..4]);
            Some(selector)
        } else {
            None
        }
    }

    /// Classify the transaction type
    fn classify_transaction(&self, tx: &Transaction) -> TransactionType {
        // Contract deployment
        if tx.to.is_none() {
            return TransactionType::ContractDeployment;
        }

        // Simple ETH transfer (no data)
        if tx.input.0.is_empty() {
            return TransactionType::Unknown;
        }

        // Check function selector
        if let Some(selector) = self.extract_function_selector(tx) {
            if let Some(function_name) = self.known_selectors.get(&selector) {
                return match function_name.as_str() {
                    name if name.contains("swap") => TransactionType::DexSwap,
                    "transfer" | "transferFrom" => TransactionType::TokenTransfer,
                    name if name.contains("safeTransferFrom") => TransactionType::NftTransfer,
                    _ => TransactionType::ContractInteraction,
                };
            }
        }

        TransactionType::ContractInteraction
    }

    /// Extract addresses from calldata
    fn extract_addresses_from_calldata(&self, tx: &Transaction) -> ExtractedAddresses {
        let mut contract_addresses = Vec::new();
        let mut token_addresses = Vec::new();
        let mut seen = HashSet::new();

        // Add the 'to' address if present
        if let Some(to) = tx.to {
            contract_addresses.push(to);
            seen.insert(to);
        }

        // Scan calldata for address-like patterns
        let data = &tx.input.0;

        // Addresses in calldata are typically 32-byte aligned (12 zero bytes + 20 address bytes)
        for i in 0..data.len().saturating_sub(31) {
            // Check for 12 zero bytes followed by 20 bytes (address pattern)
            if data[i..i + 12] == [0u8; 12] {
                let mut addr_bytes = [0u8; 20];
                addr_bytes.copy_from_slice(&data[i + 12..i + 32]);
                let addr = Address::from(addr_bytes);

                // Skip zero address and already seen addresses
                if addr.is_zero() || seen.contains(&addr) {
                    continue;
                }

                seen.insert(addr);

                // Try to determine if it's a token or contract address
                // This is a heuristic - in production you'd query the chain
                token_addresses.push(addr);
            }
        }

        // Also look for PUSH20 opcodes in potential bytecode
        for i in 0..data.len().saturating_sub(20) {
            if data[i] == 0x73 { // PUSH20 opcode
                let mut addr_bytes = [0u8; 20];
                addr_bytes.copy_from_slice(&data[i + 1..i + 21]);
                let addr = Address::from(addr_bytes);

                if !addr.is_zero() && !seen.contains(&addr) {
                    seen.insert(addr);
                    contract_addresses.push(addr);
                }
            }
        }

        ExtractedAddresses {
            contract_addresses,
            token_addresses,
        }
    }

    /// Try to decode transaction input
    fn try_decode_input(&self, tx: &Transaction) -> Option<DecodedInput> {
        let selector = self.extract_function_selector(tx)?;
        let function_name = self.known_selectors.get(&selector)?;

        // For now, return basic info
        // In production, you'd have full ABI decoding
        Some(DecodedInput {
            function_name: function_name.clone(),
            parameters: vec![],
        })
    }
}

impl Default for TransactionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct ExtractedAddresses {
    contract_addresses: Vec<Address>,
    token_addresses: Vec<Address>,
}

/// Address replacer for frontrunning
pub struct AddressReplacer {
    our_address: Address,
}

impl AddressReplacer {
    /// Create a new address replacer
    pub fn new(our_address: Address) -> Self {
        Self { our_address }
    }

    /// Replace addresses in transaction calldata
    pub fn replace_addresses(&self, original_data: &Bytes, target_addresses: &[Address]) -> Bytes {
        let mut modified_data = original_data.0.clone();

        for target_addr in target_addresses {
            // Replace 32-byte aligned addresses (12 zeros + 20 address bytes)
            let target_pattern = self.to_32_byte_pattern(target_addr);
            let replacement_pattern = self.to_32_byte_pattern(&self.our_address);

            modified_data = self.replace_pattern(&modified_data, &target_pattern, &replacement_pattern);

            // Also replace PUSH20 patterns
            let push20_target = self.to_push20_pattern(target_addr);
            let push20_replacement = self.to_push20_pattern(&self.our_address);

            modified_data = self.replace_pattern(&modified_data, &push20_target, &push20_replacement);
        }

        Bytes::from(modified_data)
    }

    /// Convert address to 32-byte calldata pattern
    fn to_32_byte_pattern(&self, addr: &Address) -> Vec<u8> {
        let mut pattern = vec![0u8; 12];
        pattern.extend_from_slice(addr.as_bytes());
        pattern
    }

    /// Convert address to PUSH20 pattern (0x73 + 20 bytes)
    fn to_push20_pattern(&self, addr: &Address) -> Vec<u8> {
        let mut pattern = vec![0x73];
        pattern.extend_from_slice(addr.as_bytes());
        pattern
    }

    /// Replace pattern in byte array
    fn replace_pattern(&self, data: &[u8], pattern: &[u8], replacement: &[u8]) -> Vec<u8> {
        let mut result = data.to_vec();
        let pattern_len = pattern.len();

        if pattern_len == 0 || pattern_len != replacement.len() {
            return result;
        }

        let mut i = 0;
        while i <= result.len().saturating_sub(pattern_len) {
            if &result[i..i + pattern_len] == pattern {
                result.splice(i..i + pattern_len, replacement.iter().copied());
                i += pattern_len;
            } else {
                i += 1;
            }
        }

        result
    }

    /// Extract addresses that can be replaced for frontrunning
    pub fn identify_replaceable_addresses(
        &self,
        parsed_tx: &ParsedTransaction,
        sender: Address,
    ) -> Vec<Address> {
        let mut replaceable = Vec::new();

        // The sender's address is typically replaceable
        replaceable.push(sender);

        // In DEX swaps, recipient addresses are often replaceable
        match parsed_tx.transaction_type {
            TransactionType::DexSwap => {
                // The sender is the one doing the swap
                // We want to replace them with our address
            }
            TransactionType::NftMint => {
                // Replace the minter's address
            }
            _ => {}
        }

        replaceable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_selector_extraction() {
        let analyzer = TransactionAnalyzer::new();

        let mut tx = Transaction {
            hash: ethers::types::H256::zero(),
            nonce: ethers::types::U64::zero(),
            block_hash: None,
            block_number: None,
            transaction_index: None,
            from: Address::zero(),
            to: Some(Address::zero()),
            value: ethers::types::U256::zero(),
            gas_price: Some(ethers::types::U256::zero()),
            gas: ethers::types::U256::from(21000),
            input: Bytes::from(vec![0xa9, 0x05, 0x9c, 0xbb]), // transfer selector
            v: ethers::types::U64::zero(),
            r: ethers::types::U256::zero(),
            s: ethers::types::U256::zero(),
            transaction_type: None,
            access_list: None,
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            chain_id: None,
            other: Default::default(),
        };

        let selector = analyzer.extract_function_selector(&tx);
        assert_eq!(selector, Some([0xa9, 0x05, 0x9c, 0xbb]));
    }

    #[test]
    fn test_address_replacement() {
        let our_address = Address::random();
        let target_address = Address::random();
        let replacer = AddressReplacer::new(our_address);

        // Create calldata with a 32-byte aligned address
        let mut data = vec![0u8; 16]; // Some prefix
        data.extend_from_slice(&[0u8; 12]); // Padding
        data.extend_from_slice(target_address.as_bytes()); // Target address
        data.extend_from_slice(&[0u8; 16]); // Some suffix

        let original = Bytes::from(data);
        let modified = replacer.replace_addresses(&original, &[target_address]);

        // Check that our address is now in the data
        let modified_str = hex::encode(&modified.0);
        let our_addr_str = hex::encode(our_address.as_bytes());

        assert!(modified_str.contains(&our_addr_str));
    }

    #[test]
    fn test_transaction_classification() {
        let analyzer = TransactionAnalyzer::new();

        // Test DEX swap classification
        let mut tx = Transaction {
            hash: ethers::types::H256::zero(),
            nonce: ethers::types::U64::zero(),
            block_hash: None,
            block_number: None,
            transaction_index: None,
            from: Address::zero(),
            to: Some(Address::zero()),
            value: ethers::types::U256::zero(),
            gas_price: Some(ethers::types::U256::zero()),
            gas: ethers::types::U256::from(21000),
            input: Bytes::from(vec![0x38, 0xed, 0x17, 0x39]), // swapExactTokensForTokens
            v: ethers::types::U64::zero(),
            r: ethers::types::U256::zero(),
            s: ethers::types::U256::zero(),
            transaction_type: None,
            access_list: None,
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            chain_id: None,
            other: Default::default(),
        };

        let tx_type = analyzer.classify_transaction(&tx);
        assert_eq!(tx_type, TransactionType::DexSwap);
    }
}
