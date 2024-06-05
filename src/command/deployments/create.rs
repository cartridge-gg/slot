#![allow(clippy::enum_variant_names)]

use anyhow::{bail, Result};
use clap::Args;
use graphql_client::{GraphQLQuery, Response};

use crate::{
    api::Client,
    command::deployments::create::create_deployment::{
        CreateDeploymentCreateDeployment::{KatanaConfig, MadaraConfig, ToriiConfig},
        DeploymentTier, Variables,
    },
    credential::Credentials,
};

use super::{
    services::{katana::KatanaCreateArgs, CreateServiceCommands},
    Long, Tier,
};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/command/deployments/create.graphql",
    response_derives = "Debug"
)]
pub struct CreateDeployment;

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
    create_commands: CreateServiceCommands,
}

impl CreateArgs {
    pub async fn run(&self) -> Result<()> {
        if self.create_commands.local() {
            self.create_commands.run_local().await?;
        }

        let service = self.create_commands.service_input();

        let tier = match &self.tier {
            Tier::Basic => DeploymentTier::basic,
        };

        let request_body = CreateDeployment::build_query(Variables {
            project: self.project.clone(),
            tier,
            service,
            wait: Some(true),
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let res: Response<create_deployment::ResponseData> = client.query(&request_body).await?;
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
                    println!("  Start Block: {}", config.start_block);
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
