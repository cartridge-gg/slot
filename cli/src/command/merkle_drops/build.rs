use anyhow::Result;
use clap::Args;
use serde::{Deserialize, Serialize};
use serde_json::json;
use slot::merkle::{build_merkle_tree, MerkleClaimData};
use starknet::core::types::Felt;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, U256};
use futures::stream::{self, StreamExt};

// Standard ERC721 ABI for ownerOf function
abigen!(
    ERC721,
    r#"[
        function ownerOf(uint256 tokenId) external view returns (address)
    ]"#
);

// Contract configuration for multi-contract mode
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContractConfig {
    pub address: String,
    pub from_id: u64,
    pub to_id: u64,
}

// Precalculated rewards configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RewardsConfig {
    pub contracts: HashMap<String, Vec<u64>>, // contract_address -> [token_a_per_nft, token_b_per_nft, ...]
}

// Build mode enum
#[derive(Debug, Clone)]
enum BuildMode {
    OnChain,
    Precalculated,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Build merkle drop options")]
pub struct BuildArgs {
    #[arg(long, help = "Name for the merkle drop")]
    name: String,

    // Single contract mode (backward compatibility)
    #[arg(long, help = "Contract address to query for token holders (single contract mode)")]
    contract_address: Option<String>,

    #[arg(
        long,
        help = "Network RPC URL (e.g., https://ethereum-rpc.publicnode.com)"
    )]
    rpc_url: String,

    #[arg(long, help = "Network name (e.g., ETH, BASE)", default_value = "ETH")]
    network: String,

    #[arg(long, help = "Description of the merkle drop")]
    description: String,

    #[arg(long, help = "Claim contract address for the merkle drop")]
    claim_contract: String,

    #[arg(long, help = "Entrypoint address for claiming")]
    entrypoint: String,

    #[arg(
        long,
        help = "Block height to query at (required for deterministic snapshots)"
    )]
    block_height: Option<u64>,

    // Single contract mode token range (backward compatibility)
    #[arg(long, help = "Starting token ID (inclusive)", default_value = "1")]
    from_id: u64,

    #[arg(long, help = "Ending token ID (inclusive)", default_value = "8000")]
    to_id: u64,

    // Multi-contract mode
    #[arg(
        long,
        help = "Path to JSON file with contract configurations [{\"address\": \"0x...\", \"from_id\": 1, \"to_id\": 1000}, ...]"
    )]
    contracts_config: Option<PathBuf>,

    // Precalculated rewards mode
    #[arg(
        long,
        help = "Use precalculated token amounts instead of querying on-chain"
    )]
    use_precalculated: bool,

    #[arg(
        long,
        help = "Path to JSON file with rewards config: {\"contracts\": {\"0x...\": reward_per_token}}"
    )]
    rewards_config: Option<PathBuf>,

    #[arg(
        long,
        help = "Output file path for the merkle drop JSON data",
        default_value = "merkle_drop.json"
    )]
    output: PathBuf,

    #[arg(
        long,
        help = "Delay between RPC calls in milliseconds",
        default_value = "10"
    )]
    delay_ms: u64,

    #[arg(long, help = "Number of concurrent RPC requests", default_value = "10")]
    concurrency: usize,
}

impl BuildArgs {
    pub async fn run(&self) -> Result<()> {
        // Validate arguments and determine build mode
        let build_mode = self.validate_and_get_mode()?;
        
        // Get final holder rewards based on mode
        let holders = match build_mode {
            BuildMode::OnChain => self.convert_onchain_to_flat().await?,
            BuildMode::Precalculated => self.calculate_precalculated_rewards().await?,
        };

        if holders.is_empty() {
            return Err(anyhow::anyhow!("No token holders found"));
        }

        println!("Found {} unique holders", holders.len());

        // Convert to merkle claim data format
        let merkle_data: Vec<MerkleClaimData> = holders
            .into_iter()
            .map(|(address, amounts)| {
                // Convert amounts to Felt array
                let data: Vec<Felt> = amounts.iter().map(|id| Felt::from(*id as u64)).collect();
                MerkleClaimData { address, data }
            })
            .collect();

        // Build merkle tree
        println!("Building merkle tree...");
        let (root, _proofs) = build_merkle_tree(&merkle_data)?;

        println!("Merkle root: 0x{}", hex::encode(&root));

        // Prepare snapshot data
        let snapshot: Vec<Vec<serde_json::Value>> = merkle_data
            .iter()
            .map(|claim| {
                // Convert Felt data back to numeric values for JSON output
                let amounts: Vec<i64> = claim
                    .data
                    .iter()
                    .map(|f| {
                        // Convert Felt to u64 then to i64
                        // This assumes amounts fit in i64 range
                        let bytes = f.to_bytes_be();
                        let mut value = 0u64;
                        for (i, &byte) in bytes.iter().rev().enumerate().take(8) {
                            value |= (byte as u64) << (i * 8);
                        }
                        value as i64
                    })
                    .collect();
                vec![json!(claim.address), json!(amounts)]
            })
            .collect();

        // Build the complete output with metadata
        let output_data = json!({
            "name": self.name,
            "network": self.network,
            "description": self.description,
            "claim_contract": self.claim_contract,
            "entrypoint": self.entrypoint,
            "merkle_root": format!("0x{}", hex::encode(&root)),
            "snapshot": snapshot
        });

        // Write to output file
        let output_str = serde_json::to_string_pretty(&output_data)?;
        std::fs::write(&self.output, output_str)?;

        println!("Merkle drop data written to: {}", self.output.display());
        println!("\nSummary:");
        println!("  Total unique holders: {}", merkle_data.len());
        println!("  Merkle root: 0x{}", hex::encode(&root));
        println!("  Output file: {}", self.output.display());
        println!("\nNext steps:");
        println!("1. Review the generated snapshot data");
        println!(
            "2. Use 'slot merkle-drops create json --file {}' to create the merkle drop",
            self.output.display()
        );

        Ok(())
    }
    
    // Validate arguments and determine the build mode
    fn validate_and_get_mode(&self) -> Result<BuildMode> {
        if self.use_precalculated {
            // Precalculated mode validation
            if self.rewards_config.is_none() {
                return Err(anyhow::anyhow!(
                    "--rewards-config is required when using --use-precalculated"
                ));
            }
            
            // In precalculated mode, we still need contracts to query holders
            if self.contract_address.is_none() && self.contracts_config.is_none() {
                return Err(anyhow::anyhow!(
                    "Either --contract-address or --contracts-config is required even in precalculated mode to determine eligible holders"
                ));
            }
            
            if self.block_height.is_none() {
                return Err(anyhow::anyhow!(
                    "--block-height is required for deterministic snapshots"
                ));
            }
            
            Ok(BuildMode::Precalculated)
        } else {
            // On-chain mode validation
            if self.contract_address.is_none() && self.contracts_config.is_none() {
                return Err(anyhow::anyhow!(
                    "Either --contract-address or --contracts-config is required"
                ));
            }
            
            if self.contract_address.is_some() && self.contracts_config.is_some() {
                return Err(anyhow::anyhow!(
                    "Cannot use both --contract-address and --contracts-config simultaneously"
                ));
            }
            
            if self.block_height.is_none() {
                return Err(anyhow::anyhow!(
                    "--block-height is required for deterministic snapshots"
                ));
            }
            
            Ok(BuildMode::OnChain)
        }
    }
    
    // Parse contract configurations from file or single contract
    fn get_contract_configs(&self) -> Result<Vec<ContractConfig>> {
        if let Some(config_path) = &self.contracts_config {
            // Multi-contract mode from file
            let content = std::fs::read_to_string(config_path)
                .map_err(|e| anyhow::anyhow!("Failed to read contracts config file: {}", e))?;
            let configs: Vec<ContractConfig> = serde_json::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse contracts config JSON: {}", e))?;
            
            if configs.is_empty() {
                return Err(anyhow::anyhow!("Contracts config file must contain at least one contract"));
            }
            
            Ok(configs)
        } else if let Some(address) = &self.contract_address {
            // Single contract mode (backward compatibility)
            Ok(vec![ContractConfig {
                address: address.clone(),
                from_id: self.from_id,
                to_id: self.to_id,
            }])
        } else {
            Err(anyhow::anyhow!("No contract configuration provided"))
        }
    }

    // Query all contracts based on configuration
    async fn query_all_contracts(&self) -> Result<HashMap<String, HashMap<String, Vec<i64>>>> {
        let contract_configs = self.get_contract_configs()?;
        let mut all_holders: HashMap<String, HashMap<String, Vec<i64>>> = HashMap::new();
        
        println!("Querying {} contract(s)...", contract_configs.len());
        
        for (i, config) in contract_configs.iter().enumerate() {
            println!("\n[{}/{}] Processing contract: {}", i + 1, contract_configs.len(), config.address);
            println!("  Token range: {} to {}", config.from_id, config.to_id);
            
            // Create provider for this contract
            let provider = Provider::<Http>::try_from(&self.rpc_url)?;
            let provider = Arc::new(provider);
            
            // Parse contract address
            let contract_address: Address = config.address.parse()
                .map_err(|e| anyhow::anyhow!("Invalid contract address '{}': {}", config.address, e))?;
            
            // Create contract instance
            let contract = ERC721::new(contract_address, provider.clone());
            
            // Query holders for this contract
            let contract_holders = self
                .query_single_contract_holders(contract, config.from_id, config.to_id)
                .await?;
            
            println!("  Found {} holders for this contract", contract_holders.len());
            
            // Store holders by contract address
            for (address, token_ids) in contract_holders {
                all_holders.entry(address)
                    .or_insert_with(HashMap::new)
                    .insert(config.address.clone(), token_ids);
            }
        }
        
        Ok(all_holders)
    }
    
    // Convert on-chain data to flat format for compatibility
    async fn convert_onchain_to_flat(&self) -> Result<HashMap<String, Vec<i64>>> {
        let holders_by_contract = self.query_all_contracts().await?;
        let mut flat_holders: HashMap<String, Vec<i64>> = HashMap::new();
        
        // Flatten the data structure - combine all token IDs across contracts
        for (holder_address, contracts) in holders_by_contract {
            let mut all_tokens = Vec::new();
            for token_ids in contracts.values() {
                all_tokens.extend(token_ids);
            }
            all_tokens.sort();
            
            if !all_tokens.is_empty() {
                flat_holders.insert(holder_address, all_tokens);
            }
        }
        
        Ok(flat_holders)
    }
    
    async fn query_single_contract_holders(
        &self,
        contract: ERC721<Provider<Http>>,
        from_id: u64,
        to_id: u64,
    ) -> Result<HashMap<String, Vec<i64>>> {
        let owners_by_address = Arc::new(Mutex::new(HashMap::<String, Vec<i64>>::new()));
        let total_tokens = to_id - from_id + 1;
        let processed = Arc::new(Mutex::new(0u64));

        println!("Querying {} tokens...", total_tokens);

        // Create a stream of token IDs
        let token_ids: Vec<u64> = (from_id..=to_id).collect();

        // Process token IDs in parallel with controlled concurrency
        let contract = Arc::new(contract);
        let delay_ms = self.delay_ms;

        stream::iter(token_ids)
            .map(|token_id| {
                let contract = contract.clone();
                let owners_by_address = owners_by_address.clone();
                let processed = processed.clone();

                async move {
                    // Add delay between calls to avoid rate limiting
                    if delay_ms > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    }

                    // Set up the call with required block height
                    let call = contract
                        .owner_of(U256::from(token_id))
                        .block(self.block_height.unwrap()); // Safe unwrap due to validation

                    // Try to get the owner
                    match call.call().await {
                        Ok(owner) => {
                            let owner_str = format!("{:?}", owner);
                            let mut owners = owners_by_address.lock().await;
                            owners
                                .entry(owner_str)
                                .or_insert_with(Vec::new)
                                .push(token_id as i64);
                        }
                        Err(e) => {
                            // Token might not exist or be burned
                            // This is expected for some token IDs
                            if !Self::is_token_not_found_error_static(&e) {
                                // Log other errors but continue
                                eprintln!("Warning: Error querying token {}: {}", token_id, e);
                            }
                        }
                    }

                    // Update progress
                    let mut count = processed.lock().await;
                    *count += 1;

                    // Progress indicator every 100 tokens or at milestones
                    if *count % 100 == 0 || *count == 1 || *count == total_tokens {
                        println!(
                            "  Progress: {}/{} ({:.1}%)",
                            *count,
                            total_tokens,
                            (*count as f64 / total_tokens as f64) * 100.0
                        );
                    }
                }
            })
            .buffer_unordered(self.concurrency)
            .collect::<Vec<_>>()
            .await;

        // Get the final results and sort token IDs for each owner
        let mut owners = owners_by_address.lock().await;
        for tokens in owners.values_mut() {
            tokens.sort();
        }

        // Return the owned HashMap
        let result = std::mem::take(&mut *owners);
        Ok(result)
    }
    
    // Calculate precalculated rewards based on token holdings and reward config
    async fn calculate_precalculated_rewards(&self) -> Result<HashMap<String, Vec<i64>>> {
        // Load rewards configuration
        let rewards_config_path = self.rewards_config.as_ref().unwrap(); // Safe unwrap due to validation
        let content = std::fs::read_to_string(rewards_config_path)
            .map_err(|e| anyhow::anyhow!("Failed to read rewards config file: {}", e))?;
        let rewards_config: RewardsConfig = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse rewards config JSON: {}", e))?;
        
        // Determine the number of reward tokens from the first contract's rewards
        let num_reward_tokens = rewards_config.contracts.values()
            .next()
            .map(|rewards| rewards.len())
            .unwrap_or(1);
        
        // Validate that all contracts have the same number of reward tokens
        for (contract_addr, rewards) in &rewards_config.contracts {
            if rewards.len() != num_reward_tokens {
                return Err(anyhow::anyhow!(
                    "All contracts must have the same number of reward tokens. Contract {} has {} tokens, expected {}",
                    contract_addr, rewards.len(), num_reward_tokens
                ));
            }
        }
        
        println!(
            "Loaded rewards config for {} contracts with {} reward token(s)", 
            rewards_config.contracts.len(), 
            num_reward_tokens
        );
        
        // Get token holders organized by contract
        let token_holders = self.query_all_contracts().await?;
        
        println!("\nCalculating precalculated multi-token rewards...");
        
        // Calculate rewards for each holder
        let mut rewards_by_holder: HashMap<String, Vec<i64>> = HashMap::new();
        
        for (holder_address, contracts_data) in token_holders {
            // Initialize total rewards for each token type
            let mut total_rewards: Vec<i64> = vec![0; num_reward_tokens];
            
            // Calculate rewards per contract
            for (contract_address, token_ids) in contracts_data {
                let tokens_in_contract = token_ids.len() as u64;
                    
                if tokens_in_contract > 0 {
                    if let Some(reward_per_token_list) = rewards_config.contracts.get(&contract_address) {
                        // Calculate rewards for each token type
                        for (token_index, &reward_per_token) in reward_per_token_list.iter().enumerate() {
                            let contract_reward = tokens_in_contract * reward_per_token;
                            total_rewards[token_index] += contract_reward as i64;
                        }
                        
                        println!(
                            "  {}: {} NFTs in contract {} → rewards: {:?}",
                            holder_address,
                            tokens_in_contract,
                            contract_address,
                            reward_per_token_list.iter()
                                .map(|&r| tokens_in_contract * r)
                                .collect::<Vec<_>>()
                        );
                    } else {
                        println!(
                            "Warning: No reward configuration found for contract {}", 
                            contract_address
                        );
                    }
                }
            }
            
            // Only include holders with non-zero rewards
            if total_rewards.iter().any(|&reward| reward > 0) {
                rewards_by_holder.insert(holder_address.clone(), total_rewards.clone());
                println!(
                    "  Total rewards for {}: {:?}", 
                    holder_address, 
                    total_rewards
                );
            }
        }
        
        // Calculate total tokens being distributed
        let mut total_tokens_distributed: Vec<u64> = vec![0; num_reward_tokens];
        for rewards in rewards_by_holder.values() {
            for (token_index, &amount) in rewards.iter().enumerate() {
                total_tokens_distributed[token_index] += amount as u64;
            }
        }
        
        println!(
            "\nSummary: {} holders eligible for rewards across {} token types", 
            rewards_by_holder.len(),
            num_reward_tokens
        );
        
        println!("Total tokens being distributed:");
        for (token_index, &total) in total_tokens_distributed.iter().enumerate() {
            println!("  Token {}: {} total", token_index + 1, total);
        }
        
        Ok(rewards_by_holder)
    }

    fn is_token_not_found_error_static(error: &ContractError<Provider<Http>>) -> bool {
        // Check if error indicates token doesn't exist
        // Common ERC721 revert messages for non-existent tokens
        let error_str = error.to_string().to_lowercase();
        error_str.contains("nonexistent token")
            || error_str.contains("invalid token")
            || error_str.contains("token does not exist")
            || error_str.contains("owner query for nonexistent token")
            || error_str.contains("erc721: owner query for nonexistent token")
    }
}
