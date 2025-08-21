use anyhow::Result;
use clap::Args;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

use super::snapshot::SnapshotData;

// Rewards configuration for processing
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RewardsConfig {
    pub contracts: HashMap<String, Vec<u64>>, // contract_address -> [token_a_per_nft, token_b_per_nft, ...]
}

// Final processed data structure (ready for create command)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessedData {
    pub name: String,
    pub network: String,
    pub description: String,
    pub claim_contract: String,
    pub entrypoint: String,
    pub snapshot: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Process rewards options")]
pub struct ProcessArgs {
    #[arg(long, help = "Name for the merkle drop")]
    name: String,

    #[arg(long, help = "Description of the merkle drop")]
    description: String,

    #[arg(long, help = "Claim contract address for the merkle drop")]
    claim_contract: String,

    #[arg(long, help = "Entrypoint address for claiming")]
    entrypoint: String,

    #[arg(
        long,
        help = "Comma-separated list of snapshot files to process",
        value_delimiter = ','
    )]
    snapshots: Vec<PathBuf>,

    #[arg(
        long,
        help = "Path to JSON file with reward amounts per contract"
    )]
    rewards_config: PathBuf,

    #[arg(
        long,
        help = "Output file path for the processed data",
        default_value = "processed_rewards.json"
    )]
    output: PathBuf,
}

impl ProcessArgs {
    pub async fn run(&self) -> Result<()> {
        println!("Processing rewards for merkle drop: {}", self.name);
        println!("Loading {} snapshot files...", self.snapshots.len());

        // Load rewards configuration
        let rewards_config = self.load_rewards_config()?;
        
        // Determine number of reward tokens from first contract
        let num_reward_tokens = rewards_config.contracts.values()
            .next()
            .map(|rewards| rewards.len())
            .unwrap_or(1);
        
        // Validate all contracts have same number of reward tokens
        for (contract_addr, rewards) in &rewards_config.contracts {
            if rewards.len() != num_reward_tokens {
                return Err(anyhow::anyhow!(
                    "All contracts must have the same number of reward tokens. Contract {} has {} tokens, expected {}",
                    contract_addr, rewards.len(), num_reward_tokens
                ));
            }
        }

        println!(
            "Rewards config loaded: {} contracts with {} reward token(s)",
            rewards_config.contracts.len(),
            num_reward_tokens
        );

        // Load all snapshots
        let snapshots = self.load_snapshots().await?;
        println!("Loaded snapshots from {} contracts", snapshots.len());

        // Validate network consistency
        self.validate_network_consistency(&snapshots)?;
        let network = snapshots[0].network.clone();

        // Calculate rewards for each holder
        let rewards_by_holder = self.calculate_rewards(&snapshots, &rewards_config).await?;

        if rewards_by_holder.is_empty() {
            return Err(anyhow::anyhow!("No holders found with rewards"));
        }

        // Calculate total tokens being distributed
        let mut total_tokens_distributed: Vec<u64> = vec![0; num_reward_tokens];
        for rewards in rewards_by_holder.values() {
            for (token_index, &amount) in rewards.iter().enumerate() {
                total_tokens_distributed[token_index] += amount as u64;
            }
        }

        // Convert to snapshot format expected by create command
        let snapshot: Vec<Vec<serde_json::Value>> = rewards_by_holder
            .into_iter()
            .map(|(address, rewards)| {
                vec![json!(address), json!(rewards)]
            })
            .collect();

        // Create processed data structure
        let processed_data = ProcessedData {
            name: self.name.clone(),
            network,
            description: self.description.clone(),
            claim_contract: self.claim_contract.clone(),
            entrypoint: self.entrypoint.clone(),
            snapshot,
        };

        // Write to output file
        let output_str = serde_json::to_string_pretty(&processed_data)?;
        std::fs::write(&self.output, output_str)?;

        println!("Processed rewards written to: {}", self.output.display());
        println!("\nProcessing Summary:");
        println!("  Name: {}", self.name);
        println!("  Network: {}", processed_data.network);
        println!("  Total Holders: {}", processed_data.snapshot.len());
        println!("  Reward Tokens: {}", num_reward_tokens);
        
        println!("\nTotal tokens being distributed:");
        for (token_index, &total) in total_tokens_distributed.iter().enumerate() {
            println!("  Token {}: {} total", token_index + 1, total);
        }
        
        println!("  Output File: {}", self.output.display());
        
        println!("\nNext steps:");
        println!("1. Review the processed reward data");
        println!("2. Use 'slot merkle-drops create json --file {}' to create the merkle drop", self.output.display());

        Ok(())
    }

    fn load_rewards_config(&self) -> Result<RewardsConfig> {
        let content = std::fs::read_to_string(&self.rewards_config)
            .map_err(|e| anyhow::anyhow!("Failed to read rewards config file: {}", e))?;
        
        let rewards_config: RewardsConfig = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse rewards config JSON: {}", e))?;
        
        Ok(rewards_config)
    }

    async fn load_snapshots(&self) -> Result<Vec<SnapshotData>> {
        let mut snapshots = Vec::new();
        
        for snapshot_file in &self.snapshots {
            println!("  Loading snapshot: {}", snapshot_file.display());
            
            let content = std::fs::read_to_string(snapshot_file)
                .map_err(|e| anyhow::anyhow!("Failed to read snapshot file {}: {}", snapshot_file.display(), e))?;
            
            let snapshot: SnapshotData = serde_json::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse snapshot JSON from {}: {}", snapshot_file.display(), e))?;
            
            println!("    Contract: {} ({} holders)", snapshot.contract_address, snapshot.snapshot.len());
            snapshots.push(snapshot);
        }
        
        Ok(snapshots)
    }

    fn validate_network_consistency(&self, snapshots: &[SnapshotData]) -> Result<()> {
        if snapshots.is_empty() {
            return Err(anyhow::anyhow!("No snapshots provided"));
        }
        
        let first_network = &snapshots[0].network;
        
        for snapshot in snapshots {
            if snapshot.network != *first_network {
                return Err(anyhow::anyhow!(
                    "Network mismatch: snapshot for contract {} is on network '{}', expected '{}'",
                    snapshot.contract_address,
                    snapshot.network,
                    first_network
                ));
            }
        }
        
        Ok(())
    }

    async fn calculate_rewards(
        &self,
        snapshots: &[SnapshotData],
        rewards_config: &RewardsConfig,
    ) -> Result<HashMap<String, Vec<i64>>> {
        println!("\nCalculating rewards across {} snapshots...", snapshots.len());
        
        // Determine number of reward tokens
        let num_reward_tokens = rewards_config.contracts.values()
            .next()
            .map(|rewards| rewards.len())
            .unwrap_or(1);

        let mut rewards_by_holder: HashMap<String, Vec<i64>> = HashMap::new();

        // Process each snapshot
        for snapshot in snapshots {
            let contract_address = &snapshot.contract_address;
            
            if let Some(reward_per_token_list) = rewards_config.contracts.get(contract_address) {
                println!(
                    "  Processing contract {}: {} holders",
                    contract_address,
                    snapshot.snapshot.len()
                );
                
                // Calculate rewards for each holder in this snapshot
                for (holder_address, token_ids) in &snapshot.snapshot {
                    let tokens_held = token_ids.len() as u64;
                    
                    if tokens_held > 0 {
                        // Initialize holder's rewards if not exists
                        let holder_rewards = rewards_by_holder
                            .entry(holder_address.clone())
                            .or_insert_with(|| vec![0; num_reward_tokens]);
                        
                        // Add rewards for each token type
                        for (token_index, &reward_per_token) in reward_per_token_list.iter().enumerate() {
                            let contract_reward = tokens_held * reward_per_token;
                            holder_rewards[token_index] += contract_reward as i64;
                        }
                        
                        println!(
                            "    {}: {} NFTs → rewards: {:?}",
                            holder_address,
                            tokens_held,
                            reward_per_token_list.iter()
                                .map(|&r| tokens_held * r)
                                .collect::<Vec<_>>()
                        );
                    }
                }
            } else {
                println!(
                    "Warning: No reward configuration found for contract {}", 
                    contract_address
                );
            }
        }
        
        // Remove holders with zero rewards
        rewards_by_holder.retain(|_, rewards| {
            rewards.iter().any(|&reward| reward > 0)
        });

        println!(
            "\nReward calculation complete: {} holders eligible for rewards",
            rewards_by_holder.len()
        );

        Ok(rewards_by_holder)
    }
}