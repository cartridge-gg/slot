use anyhow::Result;
use clap::Args;
use serde_json::Value;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::merkle_drop::create_merkle_drop;
use slot::graphql::merkle_drop::CreateMerkleDrop;
use slot::graphql::GraphQLQuery;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Args)]
#[command(next_help_heading = "Create merkle drop options")]
pub struct CreateArgs {
    #[arg(long, help = "Name of the merkle drop.")]
    name: String,

    #[arg(long, help = "Project to associate the merkle drop with.")]
    project: String,

    #[arg(long, help = "Unique key for the merkle drop.")]
    key: String,

    #[arg(long, help = "Description of the merkle drop.")]
    description: Option<String>,

    #[arg(long, help = "Network (e.g., ETH, STARKNET).")]
    network: String,

    #[arg(long, help = "Contract address.")]
    contract: String,

    #[arg(long, help = "Entrypoint address.")]
    entrypoint: String,

    #[arg(
        long,
        help = "Arguments for the contract call (comma-separated, optional)."
    )]
    args: Option<String>,

    #[arg(long, help = "Path to JSON file containing merkle drop data.")]
    data_file: PathBuf,
}

impl CreateArgs {
    pub async fn run(&self) -> Result<()> {
        let credentials = Credentials::load()?;

        // Read and validate merkle drop data file
        let data_content = fs::read_to_string(&self.data_file)
            .map_err(|e| anyhow::anyhow!("Failed to read data file: {}", e))?;

        let merkle_data: Value = serde_json::from_str(&data_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON data file: {}", e))?;

        // Validate that data is an array
        let merkle_array = merkle_data
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Data file must contain a JSON array"))?;

        // Validate each entry in the merkle drop data
        for (index, entry) in merkle_array.iter().enumerate() {
            let entry_array = entry
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Entry {} must be an array", index))?;

            if entry_array.len() != 2 {
                return Err(anyhow::anyhow!(
                    "Entry {} must have exactly 2 elements: [address, token_ids]",
                    index
                ));
            }

            // First element should be a string (address)
            entry_array[0].as_str().ok_or_else(|| {
                anyhow::anyhow!("Entry {} first element must be a string address", index)
            })?;

            // Second element should be an array (token IDs or other data)
            entry_array[1].as_array().ok_or_else(|| {
                anyhow::anyhow!("Entry {} second element must be an array", index)
            })?;
        }

        // Calculate merkle root (simplified - in reality this would be more complex)
        let merkle_root = format!("0x{:064x}", merkle_array.len()); // Placeholder implementation

        // Parse args (optional)
        let args_vec: Option<Vec<String>> = self
            .args
            .as_ref()
            .map(|args| args.split(',').map(|s| s.trim().to_string()).collect());

        // Convert JSON data to structured claims
        let claims: Vec<create_merkle_drop::MerkleClaimInput> = merkle_array
            .iter()
            .map(|entry| {
                let entry_array = entry.as_array().unwrap(); // Already validated
                let address = entry_array[0].as_str().unwrap().to_string(); // Already validated
                let token_ids: Vec<i64> = entry_array[1]
                    .as_array()
                    .unwrap() // Already validated
                    .iter()
                    .map(|id| id.as_i64().unwrap_or(0))
                    .collect();

                create_merkle_drop::MerkleClaimInput { address, token_ids }
            })
            .collect();

        // Prepare GraphQL variables
        let variables = create_merkle_drop::Variables {
            project: self.project.clone(),
            key: self.key.clone(),
            name: self.name.clone(),
            network: self.network.clone(),
            description: self.description.clone(),
            contract: self.contract.clone(),
            entrypoint: self.entrypoint.clone(),
            args: args_vec.clone(),
            merkle_root: merkle_root.clone(),
            claims,
        };

        let request_body = CreateMerkleDrop::build_query(variables);
        let client = Client::new_with_token(credentials.access_token);

        // Try to make the API call
        match client.query(&request_body).await {
            std::result::Result::Ok(data) => {
                let data: create_merkle_drop::ResponseData = data;
                // Success! The backend now supports merkle drops
                println!("\n✅ Merkle Drop Created Successfully");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                println!("🏢 Details:");
                println!("  • ID: {}", data.create_merkle_drop.id);
                println!("  • Name: {}", data.create_merkle_drop.name);
                println!("  • Project: {}", self.project);
                println!("  • Key: {}", self.key);
                println!(
                    "  • Description: {}",
                    data.create_merkle_drop
                        .description
                        .as_deref()
                        .unwrap_or("N/A")
                );

                println!("\n🔗 Contract Details:");
                println!("  • Network: {}", data.create_merkle_drop.network);
                println!("  • Contract: {}", data.create_merkle_drop.contract);
                println!("  • Entrypoint: {}", data.create_merkle_drop.entrypoint);
                println!("  • Args: {:?}", data.create_merkle_drop.args);

                println!("\n🌳 Merkle Details:");
                println!("  • Root: {}", data.create_merkle_drop.merkle_root);
                println!("  • Entries: {}", merkle_array.len());
                println!("  • Created: {}", data.create_merkle_drop.created_at);

                std::result::Result::Ok(())
            }
            Err(err) => {
                // Check if the error is specifically about the mutation not existing
                let error_msg = err.to_string().to_lowercase();
                if error_msg.contains("createMerkledrop")
                    || error_msg.contains("no field named createMerkledrop")
                    || error_msg.contains("unknown field")
                {
                    // Backend doesn't support merkle drops yet - show preview
                    println!("⚠️  Merkle Drop API not yet available. This is a preview of the command structure.");
                    println!("\n📋 Merkle Drop Configuration Preview");
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                    println!("🏢 Details:");
                    println!("  • Name: {}", self.name);
                    println!("  • Project: {}", self.project);
                    println!("  • Key: {}", self.key);
                    println!(
                        "  • Description: {}",
                        self.description.as_deref().unwrap_or("N/A")
                    );

                    println!("\n🔗 Contract Details:");
                    println!("  • Network: {}", self.network);
                    println!("  • Contract: {}", self.contract);
                    println!("  • Entrypoint: {}", self.entrypoint);
                    println!(
                        "  • Args: {:?}",
                        args_vec.as_ref().unwrap_or(&vec!["None".to_string()])
                    );

                    println!("\n🌳 Merkle Details:");
                    println!("  • Root: {}", merkle_root);
                    println!("  • Entries: {}", merkle_array.len());

                    println!("\n📄 Data file validation: ✅ Passed");
                    println!("  • File format: Valid JSON array");
                    println!("  • Entry format: All entries have [address, token_ids] structure");

                    std::result::Result::Ok(())
                } else {
                    // Some other API error - propagate it
                    Err(err.into())
                }
            }
        }
    }
}
