#![allow(clippy::enum_variant_names)]

use anyhow::Result;
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::deployments::create_deployment::CreateDeploymentCreateDeployment::{
    KatanaConfig, SayaConfig, ToriiConfig,
};
use slot::graphql::deployments::create_deployment::*;
use slot::graphql::deployments::CreateDeployment;
use slot::graphql::GraphQLQuery;

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

    #[command(subcommand)]
    create_commands: CreateServiceCommands,
}

impl CreateArgs {
    pub async fn run(&self) -> Result<()> {
        let service = match &self.create_commands {
            CreateServiceCommands::Katana(config) => CreateServiceInput {
                type_: DeploymentService::katana,
                version: config.version.clone(),
                config: Some(CreateServiceConfigInput {
                    katana: Some(CreateKatanaConfigInput {
                        block_time: config.block_time,
                        fork_rpc_url: config.fork_rpc_url.clone(),
                        fork_block_number: config.fork_block_number,
                        seed: Some(match &config.seed {
                            Some(seed) => seed.clone(),
                            None => rand::random::<u64>().to_string(),
                        }),
                        accounts: Some(config.accounts),
                        disable_fee: config.disable_fee,
                        gas_price: config.gas_price,
                        invoke_max_steps: config.invoke_max_steps,
                        validate_max_steps: config.validate_max_steps,
                        genesis: config.genesis.clone(),
                        dev: config.dev.then_some(true),
                    }),
                    torii: None,
                    saya: None,
                }),
            },
            CreateServiceCommands::Torii(config) => CreateServiceInput {
                type_: DeploymentService::torii,
                version: config.version.clone(),
                config: Some(CreateServiceConfigInput {
                    katana: None,
                    torii: Some(CreateToriiConfigInput {
                        rpc: Some(config.rpc.clone().unwrap_or("".to_string())),
                        world: format!("{:#x}", config.world),
                        contracts: config.contracts.clone(),
                        start_block: config.start_block,
                        index_pending: config.index_pending,
                        polling_interval: config.polling_interval,
                        index_transactions: config.index_transactions,
                        index_raw_events: config.index_raw_events,
                    }),
                    saya: None,
                }),
            },
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

        let data: ResponseData = client.query(&request_body).await?;

        println!("Deployment success 🚀");

        match data.create_deployment {
            SayaConfig(config) => {
                println!("\nConfiguration:");
                println!("  RPC URL: {}", config.rpc_url);
            }
            ToriiConfig(config) => {
                println!("\nConfiguration:");
                println!("  World: {}", config.world);
                println!("  RPC: {}", config.rpc);
                if let Some(contracts) = config.contracts {
                    println!("  Contracts: {}", contracts);
                }
                if let Some(start_block) = config.start_block {
                    println!("  Start Block: {}", start_block);
                }
                if let Some(index_pending) = config.index_pending {
                    println!("  Index Pending: {}", index_pending);
                }
                if let Some(index_raw_events) = config.index_raw_events {
                    println!("  Index Raw Events: {}", index_raw_events);
                }
                if let Some(index_transactions) = config.index_transactions {
                    println!("  Index Transactions: {}", index_transactions);
                }
                println!("\nEndpoints:");
                println!("  GRAPHQL: {}", config.graphql);
                println!("  GRPC: {}", config.grpc);
            }
            KatanaConfig(config) => {
                println!("\nEndpoints:");
                println!("  RPC: {}", config.rpc);
            }
        }

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
