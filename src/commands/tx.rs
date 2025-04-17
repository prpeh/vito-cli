use anyhow::{Result, Context, bail};
use std::str::FromStr;
use ethers::{
    providers::{Provider, Http},
    types::{Address, H256},
    contract::{abigen},
    middleware::Middleware,
};
use std::sync::Arc;
use crate::config::{get_safe_tx_pool_address, get_network_name, DEFAULT_MAINNET_RPC};

// Generate contract bindings using the exact ABI from the contract at commit 3658aca34ee38cba8e5bb9ed90927c270df8584d
abigen!(
    SafeTxPool,
    r#"[
        function getTxDetails(bytes32 txHash) external view returns (address safe, address to, uint256 value, bytes data, uint8 operation, address proposer, uint256 nonce)
        function getSignatures(bytes32 txHash) external view returns (bytes[] memory)
        function getPendingTxHashes(address safe) external view returns (bytes32[] memory)
        function hasSignedTx(bytes32 txHash, address signer) external view returns (bool)
    ]"#
);

// Define a struct to hold transaction data in a more user-friendly format
#[derive(serde::Serialize)]
struct TransactionData {
    hash: String,
    safe: String,
    to: String,
    value: String,
    data: String,
    operation: u8,
    proposer: String,
    nonce: String,
    signatures: Vec<String>,
}

pub async fn execute(safe: Option<String>, rpc: Option<String>, hash: Option<String>, tx_pool: Option<String>) -> Result<()> {
    // Clone the safe value early to avoid ownership issues
    let safe_str = safe.clone().unwrap_or_else(|| "unknown".to_string());

    // Validate and require safe address
    let safe_address = match &safe {
        Some(ref addr) => Address::from_str(addr)
            .context("Invalid Safe wallet address format")?,
        None => bail!("Safe address is required. Use --safe flag or set it globally.")
    };

    // Use the provided RPC or the default mainnet RPC
    let rpc_url = rpc.unwrap_or_else(|| {
        println!("No RPC URL provided, using default Ethereum mainnet RPC");
        DEFAULT_MAINNET_RPC.to_string()
    });

    // Connect to provider
    let provider = Provider::<Http>::try_from(rpc_url.clone())
        .context("Failed to connect to RPC provider")?;
    
    let provider = Arc::new(provider);

    // Get chain ID first to identify the network
    let chain_id = provider.get_chainid().await
        .context("Failed to get chain ID from network")?;
    
    let network_id = chain_id.as_u64();
    let network_name = get_network_name(network_id);
    
    println!("Connected to {} (Chain ID: {})", network_name, network_id);

    // Check if safe exists by attempting to get its code
    let code = provider.get_code(safe_address, None).await
        .context("Failed to query the network")?;
    
    if code.is_empty() {
        bail!("Safe wallet address not found on {}", network_name);
    }

    // Determine transaction pool address
    let tx_pool_address = if let Some(custom_address) = tx_pool {
        // User provided a custom address
        println!("Using custom Safe transaction pool address: {}", custom_address);
        Address::from_str(&custom_address)
            .context("Invalid custom Safe transaction pool address")?
    } else {
        // Get the network-specific transaction pool address
        let tx_pool_address_str = get_safe_tx_pool_address(network_id);
        println!("Using Safe transaction pool at {} for {}", tx_pool_address_str, network_name);
        
        // Parse the address
        Address::from_str(tx_pool_address_str)
            .context("Invalid Safe transaction pool address")?
    };
    
    // Verify that the transaction pool contract exists
    let contract_code = provider.get_code(tx_pool_address, None).await
        .context("Failed to query the network for transaction pool contract")?;
    
    if contract_code.is_empty() {
        bail!("Transaction pool contract not found at {}. Please verify the contract address for {} is correct.", 
              tx_pool_address, network_name);
    }
    
    // Create contract instance
    let contract = SafeTxPool::new(tx_pool_address, provider.clone());
    
    if let Some(tx_hash) = hash {
        // Convert hash string to H256
        let hash = H256::from_str(&tx_hash)
            .context("Invalid transaction hash format")?;
        
        println!("Fetching transaction with hash {} for Safe {}", tx_hash, safe_str);
        
        // Fetch the transaction details from the Safe transaction pool
        let tx_details = match contract.get_tx_details(hash.into()).call().await {
            Ok(details) => details,
            Err(e) => {
                bail!("Failed to fetch transaction details: {}. This could be because the transaction does not exist or the contract interface is incorrect.", e);
            }
        };

        // Check if the transaction exists (proposer address will be zero if not)
        if tx_details.5 == Address::zero() {
            bail!("Transaction not found or has already been executed");
        }
        
        // Fetch the signatures for this transaction
        let signatures = match contract.get_signatures(hash.into()).call().await {
            Ok(sigs) => sigs,
            Err(e) => {
                bail!("Failed to fetch transaction signatures: {}. This could be because the transaction does not exist or the contract interface is incorrect.", e);
            }
        };
        
        // Convert signatures to hex strings
        let signature_strings: Vec<String> = signatures.into_iter()
            .map(|sig| format!("0x{}", hex::encode(sig.to_vec())))
            .collect();
        
        // Create a structured representation of the transaction
        let tx_data = TransactionData {
            hash: format!("0x{}", hex::encode(hash.as_bytes())),
            safe: format!("0x{:x}", tx_details.0),
            to: format!("0x{:x}", tx_details.1),
            value: tx_details.2.to_string(),
            data: format!("0x{}", hex::encode(tx_details.3.to_vec())),
            operation: tx_details.4 as u8,
            proposer: format!("0x{:x}", tx_details.5),
            nonce: tx_details.6.to_string(),
            signatures: signature_strings,
        };
        
        // Convert transaction to JSON and print it
        println!("{}", serde_json::to_string_pretty(&tx_data).unwrap());
    } else {
        println!("Fetching all pending transactions for Safe {}", safe_str);
        
        // Get all pending transaction hashes for the Safe
        // The contract doesn't paginate, just returns all hashes at once
        let all_tx_hashes_raw = match contract.get_pending_tx_hashes(safe_address).call().await {
            Ok(hashes) => hashes,
            Err(e) => {
                bail!("Failed to fetch pending transaction hashes: {}. This could be because the contract interface is incorrect or the contract does not support this function.", e);
            }
        };
        
        // Convert raw bytes32 array to H256 vector
        let all_tx_hashes: Vec<H256> = all_tx_hashes_raw.into_iter()
            .map(|h| H256::from_slice(&h))
            .collect();
        
        if all_tx_hashes.is_empty() {
            println!("No pending transactions found for Safe {}", safe_str);
            return Ok(());
        }
        
        println!("Found {} pending transactions", all_tx_hashes.len());
        
        // Fetch details for each transaction hash
        let mut transactions = Vec::new();
        for tx_hash in all_tx_hashes {
            // Fetch transaction details
            match contract.get_tx_details(tx_hash.into()).call().await {
                Ok(tx_details) => {
                    // Fetch signatures
                    let signatures = match contract.get_signatures(tx_hash.into()).call().await {
                        Ok(sigs) => sigs.into_iter()
                            .map(|sig| format!("0x{}", hex::encode(sig.to_vec())))
                            .collect(),
                        Err(_) => vec![]
                    };
                    
                    // Create transaction data object
                    let tx_data = TransactionData {
                        hash: format!("0x{}", hex::encode(tx_hash.as_bytes())),
                        safe: format!("0x{:x}", tx_details.0),
                        to: format!("0x{:x}", tx_details.1),
                        value: tx_details.2.to_string(),
                        data: format!("0x{}", hex::encode(tx_details.3.to_vec())),
                        operation: tx_details.4 as u8,
                        proposer: format!("0x{:x}", tx_details.5),
                        nonce: tx_details.6.to_string(),
                        signatures,
                    };
                    
                    transactions.push(tx_data);
                },
                Err(e) => {
                    println!("Warning: Failed to fetch details for transaction {}: {}", tx_hash, e);
                }
            }
        }
        
        // Sort transactions by nonce for better readability
        transactions.sort_by(|a, b| {
            let a_nonce = a.nonce.parse::<u64>().unwrap_or(0);
            let b_nonce = b.nonce.parse::<u64>().unwrap_or(0);
            a_nonce.cmp(&b_nonce)
        });
        
        // Convert the list to JSON and print it
        println!("{}", serde_json::to_string_pretty(&transactions).unwrap());
    }

    Ok(())
} 