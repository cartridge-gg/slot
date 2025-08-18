use anyhow::Result;
use clap::Args;
use serde_json::json;
use slot::merkle::{build_merkle_tree, MerkleClaimData};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, U256};

// Standard ERC721 ABI for ownerOf function
abigen!(
    ERC721,
    r#"[
        function ownerOf(uint256 tokenId) external view returns (address)
    ]"#
);

#[derive(Debug, Args)]
#[command(next_help_heading = "Build merkle drop options")]
pub struct BuildArgs {
    #[arg(long, help = "Contract address to query for token holders")]
    contract_address: String,

    #[arg(
        long,
        help = "Network RPC URL (e.g., https://ethereum-rpc.publicnode.com)"
    )]
    rpc_url: String,

    #[arg(long, help = "Block height to query at (optional, defaults to latest)")]
    block_height: Option<u64>,

    #[arg(long, help = "Starting token ID (inclusive)", default_value = "1")]
    from_id: u64,

    #[arg(long, help = "Ending token ID (inclusive)", default_value = "8000")]
    to_id: u64,

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
}

impl BuildArgs {
    pub async fn run(&self) -> Result<()> {
        println!(
            "🔍 Building merkle tree for contract: {}",
            self.contract_address
        );
        println!("📡 RPC URL: {}", self.rpc_url);
        println!("📊 Token range: {} to {}", self.from_id, self.to_id);

        if let Some(block) = self.block_height {
            println!("📦 Block height: {}", block);
        }

        // Create provider
        let provider = Provider::<Http>::try_from(&self.rpc_url)?;
        let provider = Arc::new(provider);

        // Parse contract address
        let contract_address: Address = self.contract_address.parse()?;

        // Create contract instance
        let contract = ERC721::new(contract_address, provider.clone());

        // Query token holders
        let holders = self
            .query_token_holders(contract, self.from_id, self.to_id)
            .await?;

        if holders.is_empty() {
            return Err(anyhow::anyhow!("No token holders found for contract"));
        }

        println!("✅ Found {} unique holders", holders.len());

        // Convert to merkle claim data format
        let merkle_data: Vec<MerkleClaimData> = holders
            .into_iter()
            .map(|(address, token_ids)| MerkleClaimData {
                address,
                token_ids: token_ids.clone(),
            })
            .collect();

        // Build merkle tree
        println!("🌳 Building merkle tree...");
        let (root, _proofs) = build_merkle_tree(&merkle_data)?;

        println!("✅ Merkle root: 0x{}", hex::encode(&root));

        // Prepare output data in the format expected by the create command
        let claims: Vec<Vec<serde_json::Value>> = merkle_data
            .iter()
            .map(|claim| vec![json!(claim.address), json!(claim.token_ids)])
            .collect();

        // Write to output file - just the claims array
        let output_str = serde_json::to_string_pretty(&claims)?;
        std::fs::write(&self.output, output_str)?;

        println!("✅ Merkle drop data written to: {}", self.output.display());
        println!("\n📋 Summary:");
        println!("  • Total unique holders: {}", merkle_data.len());
        println!("  • Merkle root: 0x{}", hex::encode(&root));
        println!("  • Output file: {}", self.output.display());
        println!("\n📋 Next steps:");
        println!("1. Review the generated snapshot data");
        println!(
            "2. Use 'slot merkle-drops create params --data-file {}' to create the merkle drop",
            self.output.display()
        );

        Ok(())
    }

    async fn query_token_holders(
        &self,
        contract: ERC721<Provider<Http>>,
        from_id: u64,
        to_id: u64,
    ) -> Result<HashMap<String, Vec<i64>>> {
        let mut owners_by_address: HashMap<String, Vec<i64>> = HashMap::new();
        let total_tokens = to_id - from_id + 1;

        println!("📦 Querying {} tokens...", total_tokens);

        for token_id in from_id..=to_id {
            // Progress indicator every 100 tokens
            if token_id % 100 == 0 || token_id == from_id {
                println!(
                    "  Progress: {}/{} ({:.1}%)",
                    token_id - from_id + 1,
                    total_tokens,
                    ((token_id - from_id + 1) as f64 / total_tokens as f64) * 100.0
                );
            }

            // Set up the call with optional block height
            let mut call = contract.owner_of(U256::from(token_id));
            if let Some(block) = self.block_height {
                call = call.block(block);
            }

            // Try to get the owner
            match call.call().await {
                Ok(owner) => {
                    let owner_str = format!("{:?}", owner);
                    owners_by_address
                        .entry(owner_str)
                        .or_insert_with(Vec::new)
                        .push(token_id as i64);
                }
                Err(e) => {
                    // Token might not exist or be burned
                    // This is expected for some token IDs
                    if self.is_token_not_found_error(&e) {
                        // Skip non-existent tokens silently
                        continue;
                    } else {
                        // Log other errors but continue
                        eprintln!("Warning: Error querying token {}: {}", token_id, e);
                    }
                }
            }

            // Add delay between calls to avoid rate limiting
            if self.delay_ms > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
            }
        }

        // Sort token IDs for each owner
        for tokens in owners_by_address.values_mut() {
            tokens.sort();
        }

        Ok(owners_by_address)
    }

    fn is_token_not_found_error(&self, error: &ContractError<Provider<Http>>) -> bool {
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
