use anyhow::Result;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
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
    #[command(subcommand)]
    command: CreateSubcommand,
}

#[derive(Subcommand, Debug)]
enum CreateSubcommand {
    #[command(about = "Create merkle drop from individual parameters")]
    Params(CreateFromParamsArgs),

    #[command(about = "Create merkle drop from a JSON file")]
    Json(CreateFromJsonArgs),

    #[command(about = "Create merkle drop from a preset")]
    Preset(CreateFromPresetArgs),
}

#[derive(Debug, Args)]
struct CreateFromParamsArgs {
    #[arg(long, help = "Unique name for the merkle drop.")]
    name: String,

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

#[derive(Debug, Args)]
struct CreateFromJsonArgs {
    #[arg(
        long = "json-file",
        help = "Path to a JSON file containing merkle drop configuration and data (e.g., output from 'slot merkle-drops build')."
    )]
    file: PathBuf,
}

#[derive(Debug, Args)]
struct CreateFromPresetArgs {
    #[arg(
        long,
        help = "The project/preset to use. https://github.com/cartridge-gg/presets/tree/main/configs"
    )]
    project: String,

    #[arg(long, help = "The merkle drop name from the preset to create.")]
    name: String,

    #[arg(
        long,
        help = "Network (e.g., SN_MAIN, ETH) to use from preset.",
        default_value = "SN_MAIN"
    )]
    network: String,
}

use slot::preset::{load_preset_config, load_preset_merkle_data, MerkleDropConfig};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MerkleDropJsonConfig {
    pub name: String,
    pub config: MerkleDropConfig,
    pub data: Vec<[serde_json::Value; 2]>,
}

// Structure for JSON output from 'slot merkle-drops build' command
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MerkleDropBuildOutput {
    pub name: String,
    pub network: String,
    pub description: String,
    pub claim_contract: String,
    pub entrypoint: String,
    pub merkle_root: String,
    pub snapshot: Vec<Vec<serde_json::Value>>,
}

impl CreateArgs {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            CreateSubcommand::Params(args) => Self::run_from_params(args).await,
            CreateSubcommand::Json(args) => Self::run_from_json(args).await,
            CreateSubcommand::Preset(args) => Self::run_from_preset(args).await,
        }
    }

    async fn run_from_params(args: &CreateFromParamsArgs) -> Result<()> {
        // Read and validate merkle drop data file
        let data_content = fs::read_to_string(&args.data_file)
            .map_err(|e| anyhow::anyhow!("Failed to read data file: {}", e))?;

        let merkle_data: Value = serde_json::from_str(&data_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON data file: {}", e))?;

        // Validate that data is an array
        let merkle_array = merkle_data
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Data file must contain a JSON array"))?;

        Self::validate_merkle_data(merkle_array)?;

        // Parse args (optional)
        let args_vec: Option<Vec<String>> = args
            .args
            .as_ref()
            .map(|args| args.split(',').map(|s| s.trim().to_string()).collect());

        // Convert JSON data to structured claims
        let claims = Self::convert_to_claims(merkle_array)?;

        // Create the merkle drop
        let config = MerkleDropConfig {
            description: args.description.clone(),
            network: args.network.clone(),
            contract: args.contract.clone(),
            entrypoint: args.entrypoint.clone(),
            args: args_vec,
        };

        Self::create_merkle_drop(&args.name, &config, &claims).await
    }

    async fn run_from_json(args: &CreateFromJsonArgs) -> Result<()> {
        // Read the JSON file
        let file_content = fs::read_to_string(&args.file)
            .map_err(|e| anyhow::anyhow!("Failed to read JSON file: {}", e))?;

        // Try to parse as build output format first (from 'slot merkle-drops build')
        if let Ok(build_output) = serde_json::from_str::<MerkleDropBuildOutput>(&file_content) {
            // This is output from 'slot merkle-drops build' command
            println!("Processing merkle drop from build output...");

            // Convert snapshot to the expected format
            let merkle_array: Vec<Value> = build_output
                .snapshot
                .iter()
                .map(|entry| {
                    if entry.len() == 2 {
                        serde_json::Value::Array(vec![entry[0].clone(), entry[1].clone()])
                    } else {
                        serde_json::Value::Array(entry.clone())
                    }
                })
                .collect();

            // Validate the merkle data
            Self::validate_merkle_data(&merkle_array)?;

            // Convert to claims
            let claims = Self::convert_to_claims(&merkle_array)?;

            // Create MerkleDropConfig from build output
            let config = MerkleDropConfig {
                description: Some(build_output.description),
                network: build_output.network,
                contract: build_output.claim_contract,
                entrypoint: build_output.entrypoint,
                args: None, // Build output doesn't include args
            };

            // Create the merkle drop
            Self::create_merkle_drop(&build_output.name, &config, &claims).await
        } else if let Ok(json_config) = serde_json::from_str::<MerkleDropJsonConfig>(&file_content)
        {
            // This is the old format with explicit config structure
            println!("Processing merkle drop from configuration file...");

            // Validate the merkle data
            let merkle_array = json_config
                .data
                .iter()
                .map(|entry| serde_json::Value::Array(vec![entry[0].clone(), entry[1].clone()]))
                .collect::<Vec<_>>();

            Self::validate_merkle_data(&merkle_array)?;

            // Convert to claims
            let claims = Self::convert_to_claims(&merkle_array)?;

            // Create the merkle drop using the config from JSON
            Self::create_merkle_drop(&json_config.name, &json_config.config, &claims).await
        } else {
            Err(anyhow::anyhow!(
                "Failed to parse JSON file. Expected either output from 'slot merkle-drops build' or a configuration file with 'name', 'config', and 'data' fields."
            ))
        }
    }

    async fn run_from_preset(args: &CreateFromPresetArgs) -> Result<()> {
        // Fetch the preset configuration
        let preset_config = load_preset_config(&args.project).await?;

        // Get the merkle drop configuration for the specified network
        let chain_config = preset_config.chains.get(&args.network).ok_or_else(|| {
            anyhow::anyhow!(
                "Network '{}' not found in preset '{}'",
                args.network,
                args.project
            )
        })?;

        let merkle_config = chain_config
            .merkledrops
            .merkledrops
            .get(&args.name)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Merkle drop '{}' not found in preset '{}' for network '{}'",
                    args.name,
                    args.project,
                    args.network
                )
            })?;

        // Fetch the merkle drop data
        let merkle_data = load_preset_merkle_data(&args.project, &args.name).await?;

        // Validate the merkle data
        Self::validate_merkle_data(&merkle_data)?;

        // Convert to claims
        let claims = Self::convert_to_claims(&merkle_data)?;

        // Create the merkle drop
        Self::create_merkle_drop(&args.name, merkle_config, &claims).await
    }

    // Helper method to validate merkle drop data format
    fn validate_merkle_data(merkle_array: &[Value]) -> Result<()> {
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
        Ok(())
    }

    // Helper method to convert JSON data to structured claims
    fn convert_to_claims(
        merkle_array: &[Value],
    ) -> Result<Vec<create_merkle_drop::MerkleClaimInput>> {
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
        Ok(claims)
    }

    // Helper method to create merkle drop via GraphQL
    async fn create_merkle_drop(
        key: &str,
        config: &MerkleDropConfig,
        claims: &[create_merkle_drop::MerkleClaimInput],
    ) -> Result<()> {
        let credentials = Credentials::load()?;

        // Prepare GraphQL variables
        let variables = create_merkle_drop::Variables {
            key: key.to_string(),
            network: config.network.clone(),
            description: config.description.clone(),
            contract: config.contract.clone(),
            entrypoint: config.entrypoint.clone(),
            args: config.args.clone(),
            claims: claims
                .iter()
                .map(|claim| create_merkle_drop::MerkleClaimInput {
                    address: claim.address.clone(),
                    token_ids: claim.token_ids.clone(),
                })
                .collect(),
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
                println!("  • Name: {}", key);
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
                println!("  • Entries: {}", claims.len());
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
                    println!("  • Name: {}", key);
                    println!(
                        "  • Description: {}",
                        config.description.as_deref().unwrap_or("N/A")
                    );

                    println!("\n🔗 Contract Details:");
                    println!("  • Network: {}", config.network);
                    println!("  • Contract: {}", config.contract);
                    println!("  • Entrypoint: {}", config.entrypoint);
                    println!(
                        "  • Args: {:?}",
                        config.args.as_ref().unwrap_or(&vec!["None".to_string()])
                    );

                    println!("\n🌳 Merkle Details:");
                    println!("  • Entries: {}", claims.len());

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
