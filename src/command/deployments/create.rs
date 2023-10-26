#![allow(clippy::enum_variant_names)]

use anyhow::Result;
use clap::Args;
use graphql_client::{GraphQLQuery, Response};

use self::create_deployment::ServiceInput;
use crate::{
    api::ApiClient,
    command::deployments::create::create_deployment::{
        CreateDeploymentCreateDeployment::{KatanaConfig, ToriiConfig},
        DeploymentService, DeploymentTier, KatanaConfigInput, ServiceConfigInput, ToriiConfigInput,
        Variables,
    },
};

use super::services::CreateCommands;

type Long = u64;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/command/deployments/create.graphql",
    response_derives = "Debug"
)]
pub struct CreateDeployment;

#[derive(clap::ValueEnum, Clone, Debug, serde::Serialize)]
pub enum Tier {
    Basic,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Create options")]
pub struct CreateArgs {
    #[arg(help = "The name of the project.")]
    pub project: String,
    #[arg(short, long, default_value = "basic")]
    #[arg(value_name = "tier")]
    #[arg(help = "Deployment tier.")]
    pub tier: Tier,

    #[command(subcommand)]
    create_commands: CreateCommands,
}

impl CreateArgs {
    pub async fn run(&self) -> Result<()> {
        let service = match &self.create_commands {
            CreateCommands::Katana(config) => ServiceInput {
                type_: DeploymentService::katana,
                version: None,
                config: Some(ServiceConfigInput {
                    katana: Some(KatanaConfigInput {
                        block_time: config.block_time,
                        fork_rpc_url: config.fork_rpc_url.clone(),
                        fork_block_number: config.fork_block_number,
                        seed: match &config.seed {
                            Some(seed) => seed.clone(),
                            None => rand::random::<u64>().to_string(),
                        },
                        total_accounts: config.total_accounts,
                    }),
                    torii: None,
                }),
            },
            CreateCommands::Torii(config) => ServiceInput {
                type_: DeploymentService::torii,
                version: None,
                config: Some(ServiceConfigInput {
                    katana: None,
                    torii: Some(ToriiConfigInput {
                        rpc: config.rpc.clone(),
                        world: format!("{:#x}", config.world),
                        start_block: Some(config.start_block),
                    }),
                }),
            },
        };

        let tier = match &self.tier {
            Tier::Basic => DeploymentTier::basic,
        };

        let request_body = CreateDeployment::build_query(Variables {
            project: self.project.clone(),
            tier,
            service,
            wait: Some(true),
        });

        let client = ApiClient::new();
        let res: Response<create_deployment::ResponseData> = client.post(&request_body).await?;
        if let Some(errors) = res.errors.clone() {
            for err in errors {
                println!("Error: {}", err.message);
            }
        }

        if let Some(data) = res.data {
            println!("Deployment success 🚀");
            match data.create_deployment {
                ToriiConfig(config) => {
                    println!("\nConfiguration:");
                    println!("  World: {}", config.world);
                    println!("  RPC: {}", config.rpc);
                    println!("  Start Block: {}", config.start_block);
                    println!("\nEndpoints:");
                    println!("  GRAPHQL: {}", config.graphql);
                    println!("  GRPC: {}", config.grpc);
                }
                KatanaConfig(config) => {
                    println!("\nEndpoints:");
                    println!("  RPC: {}", config.rpc);
                }
            }
        }

        let service = match &self.create_commands {
            CreateCommands::Katana(_) => "katana",
            CreateCommands::Torii(_) => "torii",
        };

        println!(
            "\nStream logs with `slot deployments logs {} {service} -f`",
            self.project
        );

        Ok(())
    }
}
