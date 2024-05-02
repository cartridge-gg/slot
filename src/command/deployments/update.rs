#![allow(clippy::enum_variant_names)]

use anyhow::Result;
use clap::Args;
use graphql_client::{GraphQLQuery, Response};

use self::update_deployment::UpdateServiceInput;
use crate::{
    api::ApiClient,
    command::deployments::update::update_deployment::{
        DeploymentService, DeploymentTier,
        UpdateDeploymentUpdateDeployment::{KatanaConfig, MadaraConfig, ToriiConfig},
        UpdateKatanaConfigInput, UpdateServiceConfigInput, Variables,
    },
};

use super::services::UpdateServiceCommands;

type Long = u64;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/command/deployments/update.graphql",
    response_derives = "Debug"
)]
pub struct UpdateDeployment;

#[derive(clap::ValueEnum, Clone, Debug, serde::Serialize)]
pub enum Tier {
    Basic,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Update options")]
pub struct UpdateArgs {
    #[arg(help = "The name of the project.")]
    pub project: String,
    #[arg(short, long, default_value = "basic")]
    #[arg(value_name = "tier")]
    #[arg(help = "Deployment tier.")]
    pub tier: Tier,

    #[command(subcommand)]
    update_commands: UpdateServiceCommands,
}

impl UpdateArgs {
    pub async fn run(&self) -> Result<()> {
        let service = match &self.update_commands {
            UpdateServiceCommands::Katana(config) => UpdateServiceInput {
                type_: DeploymentService::katana,
                version: config.version.clone(),
                config: Some(UpdateServiceConfigInput {
                    katana: Some(UpdateKatanaConfigInput {
                        block_time: config.block_time,
                        fork_rpc_url: config.fork_rpc_url.clone(),
                        fork_block_number: config.fork_block_number,
                        disable_fee: config.disable_fee,
                        gas_price: config.gas_price,
                        invoke_max_steps: config.invoke_max_steps,
                        validate_max_steps: config.validate_max_steps,
                    }),
                }),
            },
            UpdateServiceCommands::Torii(config) => UpdateServiceInput {
                type_: DeploymentService::torii,
                version: config.version.clone(),
                config: Some(UpdateServiceConfigInput { katana: None }),
            },
        };

        let tier = match &self.tier {
            Tier::Basic => DeploymentTier::basic,
        };

        let request_body = UpdateDeployment::build_query(Variables {
            project: self.project.clone(),
            tier,
            service,
            wait: Some(true),
        });

        let client = ApiClient::new();
        let res: Response<update_deployment::ResponseData> = client.post(&request_body).await?;
        if let Some(errors) = res.errors.clone() {
            for err in errors {
                println!("Error: {}", err.message);
            }

            return Ok(());
        }

        if let Some(data) = res.data {
            println!("Update success 🚀");
            match data.update_deployment {
                ToriiConfig(config) => {
                    println!("\nConfiguration:");
                    println!("  World: {}", config.world);
                    println!("  RPC: {}", config.rpc);
                    println!("  Start Block: {}", config.start_block);
                    println!("  Index Pending: {}", config.index_pending);
                    println!("\nEndpoints:");
                    println!("  GRAPHQL: {}", config.graphql);
                    println!("  GRPC: {}", config.grpc);
                }
                KatanaConfig(config) => {
                    println!("\nEndpoints:");
                    println!("  RPC: {}", config.rpc);
                }
                MadaraConfig => {} // TODO: implement
            }
        }

        let service = match &self.update_commands {
            UpdateServiceCommands::Katana(_) => "katana",
            UpdateServiceCommands::Torii(_) => "torii",
        };

        println!(
            "\nStream logs with `slot deployments logs {} {service} -f`",
            self.project
        );

        Ok(())
    }
}
