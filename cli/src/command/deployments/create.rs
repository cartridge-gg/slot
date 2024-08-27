#![allow(clippy::enum_variant_names)]

use anyhow::Result;
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::deployments::create_deployment::CreateDeploymentCreateDeployment::{
    KatanaConfig, MadaraConfig, ToriiConfig,
};
use slot::graphql::deployments::create_deployment::*;
use slot::graphql::deployments::CreateDeployment;
use slot::graphql::{GraphQLQuery, Response};

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
                    }),
                    torii: None,
                    madara: None,
                }),
            },
            CreateServiceCommands::Torii(config) => CreateServiceInput {
                type_: DeploymentService::torii,
                version: config.version.clone(),
                config: Some(CreateServiceConfigInput {
                    katana: None,
                    madara: None,
                    torii: Some(CreateToriiConfigInput {
                        rpc: Some(config.rpc.clone().unwrap_or("".to_string())),
                        world: format!("{:#x}", config.world),
                        start_block: Some(config.start_block.unwrap_or(0)),
                        index_pending: config.index_pending,
                        polling_interval: config.polling_interval,
                    }),
                }),
            },
            CreateServiceCommands::Madara(config) => CreateServiceInput {
                type_: DeploymentService::madara,
                version: config.version.clone(),
                config: Some(CreateServiceConfigInput {
                    katana: None,
                    torii: None,
                    madara: Some(CreateMadaraConfigInput {
                        name: config.name.clone(),
                        base_path: config.base_path.clone(),
                        dev: config.dev.then_some(true),
                        no_grandpa: config.no_grandpa.then_some(true),
                        validator: config.validator.then_some(true),
                        sealing: config.sealing.clone().map(|s| s.to_string()),
                        chain: config.chain.clone().map(|c| c.to_string()),
                    }),
                }),
            },
        };

        let tier = match &self.tier {
            Tier::Basic => DeploymentTier::basic,
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

        let res: Response<ResponseData> = client.query(&request_body).await?;
        if let Some(errors) = res.errors.clone() {
            for err in errors {
                println!("Error: {}", err.message);
            }

            return Ok(());
        }

        if let Some(data) = res.data {
            println!("Deployment success 🚀");
            match data.create_deployment {
                ToriiConfig(config) => {
                    println!("\nConfiguration:");
                    println!("  World: {}", config.world);
                    println!("  RPC: {}", config.rpc);
                    println!("  Start Block: {}", config.start_block.unwrap_or(0));
                    println!("  Index Pending: {}", config.index_pending.unwrap_or(false));
                    println!("\nEndpoints:");
                    println!("  GRAPHQL: {}", config.graphql);
                    println!("  GRPC: {}", config.grpc);
                }
                KatanaConfig(config) => {
                    println!("\nEndpoints:");
                    println!("  RPC: {}", config.rpc);
                }
                MadaraConfig(config) => {
                    println!("\nEndpoints:");
                    println!("  RPC: {}", config.rpc);
                }
            }
        }

        let service = match &self.create_commands {
            CreateServiceCommands::Katana(_) => "katana",
            CreateServiceCommands::Torii(_) => "torii",
            CreateServiceCommands::Madara(_) => "madara",
        };

        println!(
            "\nStream logs with `slot deployments logs {} {service} -f`",
            self.project
        );

        Ok(())
    }
}
