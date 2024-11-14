#![allow(clippy::enum_variant_names)]

use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use katana_cli::file::NodeArgsConfig;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::deployments::create_deployment::*;
use slot::graphql::deployments::CreateDeployment;
use slot::graphql::GraphQLQuery;
use torii_cli::args::ToriiArgsConfig;

use super::{services::CreateServiceCommands, Tier};

#[derive(Debug, Args)]
#[command(next_help_heading = "Create options")]
pub struct CreateArgs {
    #[arg(help = "The name of the project.")]
    pub project: String,

    #[arg(short, long, default_value = "basic")]
    #[arg(value_name = "tier")]
    #[arg(help = "Deployment tier.")]
    pub tier: Tier,

    #[arg(short, long)]
    #[arg(help = "The list of regions to deploy to.")]
    #[arg(value_name = "regions")]
    #[arg(value_delimiter = ',')]
    pub regions: Option<Vec<String>>,

    #[arg(long)]
    #[arg(help = "Writes the service configuration to a file and exits without deploying.")]
    #[arg(global = true)]
    pub output_service_config: Option<PathBuf>,

    #[command(subcommand)]
    create_commands: CreateServiceCommands,
}

impl CreateArgs {
    pub async fn run(&self) -> Result<()> {
        let service = match &self.create_commands {
            CreateServiceCommands::Katana(config) => {
                let service_config =
                    toml::to_string(&NodeArgsConfig::try_from(config.node_args.clone())?)?;

                if let Some(path) = &self.output_service_config {
                    std::fs::write(path, &service_config)?;
                    println!("Service config written to {}", path.display());
                    return Ok(());
                }

                CreateServiceInput {
                    type_: DeploymentService::katana,
                    version: config.version.clone(),
                    config: Some(CreateServiceConfigInput {
                        katana: Some(CreateKatanaConfigInput {
                            config_file: Some(slot::read::base64_encode_string(&service_config)),
                            // TODO: those must be changed on the server side to pull the schema correctly from the infra.
                            block_time: None,
                            accounts: None,
                            dev: None,
                            fork_rpc_url: None,
                            fork_block_number: None,
                            seed: None,
                            invoke_max_steps: None,
                            validate_max_steps: None,
                            disable_fee: None,
                            gas_price: None,
                            genesis: None,
                        }),
                        torii: None,
                        saya: None,
                    }),
                }
            }
            CreateServiceCommands::Torii(config) => {
                let service_config =
                    toml::to_string(&ToriiArgsConfig::try_from(config.torii_args.clone())?)?;

                if let Some(path) = &self.output_service_config {
                    std::fs::write(path, &service_config)?;
                    println!("Service config written to {}", path.display());
                    return Ok(());
                }

                CreateServiceInput {
                    type_: DeploymentService::torii,
                    version: config.version.clone(),
                    config: Some(CreateServiceConfigInput {
                        katana: None,
                        torii: Some(CreateToriiConfigInput {
                            config_file: Some(slot::read::base64_encode_string(&service_config)),
                            // TODO: those must be changed on the server side to pull the schema correctly from the infra.
                            rpc: None,
                            world: None,
                            contracts: None,
                            start_block: None,
                            index_pending: None,
                            polling_interval: None,
                            index_transactions: None,
                            index_raw_events: None,
                        }),
                        saya: None,
                    }),
                }
            }
            CreateServiceCommands::Saya(config) => CreateServiceInput {
                type_: DeploymentService::saya,
                version: config.version.clone(),
                config: Some(CreateServiceConfigInput {
                    katana: None,
                    torii: None,
                    saya: Some(CreateSayaConfigInput {
                        mode: config.mode.clone(),
                        rpc_url: config.rpc_url.clone(),
                        registry: config.registry.clone(),
                        settlement_contract: config.settlement_contract.clone(),
                        world: config.world.clone().to_string(),
                        prover_url: config.prover_url.clone(),
                        store_proofs: config.store_proofs.unwrap_or(false),
                        starknet_url: config.starknet_url.clone(),
                        signer_key: config.signer_key.clone(),
                        signer_address: config.signer_address.clone(),
                        private_key: config.private_key.clone(),
                        batch_size: config.batch_size.unwrap_or(1),
                        start_block: config.start_block.unwrap_or(0),
                    }),
                }),
            },
        };

        let tier = match &self.tier {
            Tier::Basic => DeploymentTier::basic,
            Tier::Common => DeploymentTier::common,
            Tier::Rare => DeploymentTier::rare,
            Tier::Epic => DeploymentTier::epic,
        };

        let request_body = CreateDeployment::build_query(Variables {
            project: self.project.clone(),
            tier,
            service,
            wait: Some(true),
            regions: self.regions.clone(),
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let _: ResponseData = client.query(&request_body).await?;

        println!("Deployment success 🚀");

        let service = match &self.create_commands {
            CreateServiceCommands::Katana(_) => "katana",
            CreateServiceCommands::Torii(_) => "torii",
            CreateServiceCommands::Saya(_) => "saya",
        };

        println!(
            "\nStream logs with `slot deployments logs {} {service} -f`",
            self.project
        );

        Ok(())
    }
}
