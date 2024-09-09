#![allow(clippy::enum_variant_names)]

use super::services::UpdateServiceCommands;
use crate::command::deployments::Tier;
use anyhow::Result;
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::deployments::update_deployment::UpdateDeploymentUpdateDeployment::{
    KatanaConfig, MadaraConfig, SayaConfig, ToriiConfig,
};
use slot::graphql::deployments::update_deployment::{
    self, UpdateKatanaConfigInput, UpdateServiceConfigInput, UpdateServiceInput,
};
use slot::graphql::deployments::{update_deployment::*, UpdateDeployment};
use slot::graphql::{GraphQLQuery, Response};

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
            UpdateServiceCommands::Saya(config) => UpdateServiceInput {
                type_: DeploymentService::saya,
                version: config.version.clone(),
                config: Some(UpdateServiceConfigInput { katana: None }),
            },
        };

        let tier = match &self.tier {
            Tier::Basic => DeploymentTier::basic,
            Tier::Common => DeploymentTier::common,
            Tier::Rare => DeploymentTier::rare,
            Tier::Epic => DeploymentTier::epic,
        };

        let request_body = UpdateDeployment::build_query(Variables {
            project: self.project.clone(),
            tier,
            service,
            wait: Some(true),
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let res: Response<update_deployment::ResponseData> = client.query(&request_body).await?;
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
                MadaraConfig => {} // TODO: implement
                SayaConfig(config) => {
                    println!("\nConfiguration:");
                    println!("  RPC URL: {}", config.rpc_url);
                }
            }
        }

        let service = match &self.update_commands {
            UpdateServiceCommands::Katana(_) => "katana",
            UpdateServiceCommands::Torii(_) => "torii",
            UpdateServiceCommands::Saya(_) => "saya",
        };

        println!(
            "\nStream logs with `slot deployments logs {} {service} -f`",
            self.project
        );

        Ok(())
    }
}
