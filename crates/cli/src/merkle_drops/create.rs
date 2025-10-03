use anyhow::Result;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use slot_core::credentials::Credentials;
use slot_graphql::api::Client;
use slot_graphql::merkle_drop::create_merkle_drop::{self, MerkleDropNetwork};
use slot_graphql::merkle_drop::CreateMerkleDrop;
use slot_graphql::GraphQLQuery;
use starknet::core::types::Felt;
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

    #[arg(
        long,
        help = "Network (ETHEREUM, STARKNET, ARBITRUM, OPTIMISM, POLYGON, BASE)."
    )]
    network: String,

    #[arg(long, help = "Contract address.")]
    contract: String,

    #[arg(long, help = "Entrypoint address.")]
    entrypoint: String,

    #[arg(long, help = "Salt for the merkle drop.")]
    salt: String,

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

use slot_core::preset::{load_preset_config, load_preset_merkle_data, MerkleDropConfig};

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

        // Validate that data is a direct array
        let merkle_array = merkle_data.as_array().ok_or_else(|| {
            anyhow::anyhow!("Data file must be a JSON array of [address, [data]] entries")
        })?;

        Self::validate_merkle_data(merkle_array)?;

        // Convert JSON data to structured claims
        let claims = Self::convert_to_claims(merkle_array)?;

        // Create the merkle drop
        let config = MerkleDropConfig {
            description: args.description.clone(),
            network: args.network.clone(),
            contract: args.contract.clone(),
            entrypoint: args.entrypoint.clone(),
            salt: args.salt.clone(),
        };

        Self::create_merkle_drop(&args.name, &config, &claims).await
    }

    async fn run_from_json(args: &CreateFromJsonArgs) -> Result<()> {
        // Read the JSON file
        let file_content = fs::read_to_string(&args.file)
            .map_err(|e| anyhow::anyhow!("Failed to read JSON file: {}", e))?;

        let json_data: Value = serde_json::from_str(&file_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON file: {}", e))?;

        // Get the root object
        let root = json_data
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("JSON file must contain a root object"))?;

        // Extract required fields
        let name = root
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'name' field"))?;

        let network_str = root
            .get("network")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'network' field"))?
            .to_uppercase();

        let contract = root
            .get("claim_contract")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'claim_contract' field"))?;

        let entrypoint = root
            .get("entrypoint")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'entrypoint' field"))?;

        let salt = root
            .get("salt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'contract_address' field"))?;

        let description = root
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Get the merkle array from the snapshot field
        let merkle_array = root
            .get("snapshot")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'snapshot' array field"))?;

        Self::validate_merkle_data(merkle_array)?;

        // Convert JSON data to structured claims
        let claims = Self::convert_to_claims(merkle_array)?;

        // Create the merkle drop
        let config = MerkleDropConfig {
            description,
            network: network_str,
            contract: contract.to_string(),
            entrypoint: entrypoint.to_string(),
            salt: salt.to_string(),
        };

        Self::create_merkle_drop(name, &config, &claims).await
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
                    "Entry {} must have exactly 2 elements: [address, [data]]",
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
        merkle_array
            .iter()
            .map(|entry| {
                let entry_array = entry
                    .as_array()
                    .ok_or_else(|| anyhow::anyhow!("Each entry must be an array"))?;

                let address_str = entry_array[0]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Address must be a string"))?;
                let address = Felt::from_hex(address_str)
                    .unwrap_or_else(|_| Felt::from_dec_str(address_str).unwrap());

                let data_array = entry_array[1]
                    .as_array()
                    .ok_or_else(|| anyhow::anyhow!("Data must be an array"))?;

                let data: Result<Vec<Felt>> = data_array
                    .iter()
                    .map(|id| {
                        let id_str = match id {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => {
                                // Convert number to hex string format
                                if let Some(n_u128) = n.as_u128() {
                                    format!("0x{:x}", n_u128)
                                } else {
                                    return Err(anyhow::anyhow!(
                                        "Claim data number must be less than 128 bits"
                                    ));
                                }
                            }
                            _ => {
                                return Err(anyhow::anyhow!(
                                    "Claim data must be a string or number"
                                ))
                            }
                        };

                        // Try hex first, then decimal
                        Felt::from_hex(&id_str)
                            .or_else(|_| Felt::from_dec_str(&id_str))
                            .map_err(|_| anyhow::anyhow!("Failed to parse claim data: {}", id_str))
                    })
                    .collect();

                Ok(create_merkle_drop::MerkleClaimInput {
                    address,
                    data: data?,
                })
            })
            .collect()
    }

    // Helper method to create merkle drop via GraphQL
    async fn create_merkle_drop(
        key: &str,
        config: &MerkleDropConfig,
        claims: &[create_merkle_drop::MerkleClaimInput],
    ) -> Result<()> {
        let credentials = Credentials::load()?;

        // Convert network string to enum
        let network = match config.network.to_uppercase().as_str() {
            "ETHEREUM" => MerkleDropNetwork::ETHEREUM,
            "STARKNET" => MerkleDropNetwork::STARKNET,
            "ARBITRUM" => MerkleDropNetwork::ARBITRUM,
            "OPTIMISM" => MerkleDropNetwork::OPTIMISM,
            "POLYGON" => MerkleDropNetwork::POLYGON,
            "BASE" => MerkleDropNetwork::BASE,
            _ => return Err(anyhow::anyhow!("Invalid network: {}", config.network)),
        };

        // Prepare GraphQL variables
        let variables = create_merkle_drop::Variables {
            key: key.to_string(),
            network,
            description: config.description.clone(),
            contract: Felt::from_hex(&config.contract)
                .unwrap_or_else(|_| Felt::from_dec_str(&config.contract).unwrap()),
            entrypoint: config.entrypoint.clone(),
            salt: config.salt.clone(),
            claims: claims
                .iter()
                .map(|claim| create_merkle_drop::MerkleClaimInput {
                    address: claim.address,
                    data: claim.data.clone(),
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
                println!("  • Network: {:?}", data.create_merkle_drop.network);
                println!("  • Claim Contract: {}", data.create_merkle_drop.contract);
                println!("  • Entrypoint: {}", data.create_merkle_drop.entrypoint);
                println!("  • Salt: {}", config.salt);

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

                    println!("\n🌳 Merkle Details:");
                    println!("  • Entries: {}", claims.len());

                    println!("\n📄 Data file validation: ✅ Passed");
                    println!("  • File format: Valid JSON array");
                    println!("  • Entry format: All entries have [address, [data]] structure");

                    std::result::Result::Ok(())
                } else {
                    // Some other API error - propagate it
                    Err(err.into())
                }
            }
        }
    }
}
