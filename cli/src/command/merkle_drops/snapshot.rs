use anyhow::Result;
use clap::Args;
use serde::{Deserialize, Serialize};
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

// Snapshot data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SnapshotData {
    pub contract_address: String,
    pub network: String,
    pub block_height: u64,
    pub snapshot: HashMap<String, Vec<i64>>,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Snapshot options")]
pub struct SnapshotArgs {
    #[arg(long, help = "Contract address to query for token holders")]
    contract_address: String,

    #[arg(
        long,
        help = "Network RPC URL (e.g., https://ethereum-rpc.publicnode.com)"
    )]
    rpc_url: String,

    #[arg(long, help = "Network name (e.g., ETH, BASE)", default_value = "ETH")]
    network: String,

    #[arg(
        long,
        help = "Block height to query at (required for deterministic snapshots)"
    )]
    block_height: u64,

    #[arg(long, help = "Starting token ID (inclusive)", default_value = "1")]
    from_id: u64,

    #[arg(long, help = "Ending token ID (inclusive)", default_value = "8000")]
    to_id: u64,

    #[arg(
        long,
        help = "Output file path for the snapshot data",
        default_value = "snapshot.json"
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

impl SnapshotArgs {
    pub async fn run(&self) -> Result<()> {
        println!(
            "Creating snapshot for contract: {}",
            self.contract_address
        );
        println!("RPC URL: {}", self.rpc_url);
        println!("Network: {}", self.network);
        println!("Token range: {} to {}", self.from_id, self.to_id);
        println!("Block height: {}", self.block_height);
        println!("Concurrency: {} parallel requests", self.concurrency);

        // Create provider
        let provider = Provider::<Http>::try_from(&self.rpc_url)?;
        let provider = Arc::new(provider);

        // Parse contract address
        let contract_address: Address = self.contract_address.parse()
            .map_err(|e| anyhow::anyhow!("Invalid contract address '{}': {}", self.contract_address, e))?;

        // Create contract instance
        let contract = ERC721::new(contract_address, provider.clone());

        // Query token holders
        let holders = self
            .query_token_holders(contract, self.from_id, self.to_id)
            .await?;

        if holders.is_empty() {
            return Err(anyhow::anyhow!("No token holders found for contract"));
        }

        println!("Found {} unique holders", holders.len());

        // Create snapshot data structure
        let snapshot_data = SnapshotData {
            contract_address: self.contract_address.clone(),
            network: self.network.clone(),
            block_height: self.block_height,
            snapshot: holders,
        };

        // Write to output file
        let output_str = serde_json::to_string_pretty(&snapshot_data)?;
        std::fs::write(&self.output, output_str)?;

        println!("Snapshot written to: {}", self.output.display());
        println!("\nSnapshot Summary:");
        println!("  Contract: {}", self.contract_address);
        println!("  Network: {}", self.network);
        println!("  Block Height: {}", self.block_height);
        println!("  Total Holders: {}", snapshot_data.snapshot.len());
        println!("  Output File: {}", self.output.display());
        
        println!("\nNext steps:");
        println!("1. Review the snapshot data");
        println!("2. Use 'slot merkle-drops process' to calculate rewards from this snapshot");

        Ok(())
    }

    async fn query_token_holders(
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
                        .block(self.block_height);

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
                            if !Self::is_token_not_found_error(&e) {
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

    fn is_token_not_found_error(error: &ContractError<Provider<Http>>) -> bool {
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