#![allow(clippy::enum_variant_names)]

use anyhow::Result;
use clap::Args;
use graphql_client::{GraphQLQuery, Response};
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider};
use url::Url;

use crate::{
    api::ApiClient,
    command::deployments::fork::fork_deployment::{
        DeploymentTier, ForkDeploymentForkDeployment::KatanaConfig, Variables,
    },
};

use super::{services::ForkServiceCommands, Long, Tier};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/command/deployments/fork.graphql",
    response_derives = "Debug"
)]
pub struct ForkDeployment;

#[derive(Debug, Args)]
#[command(next_help_heading = "Fork options")]
pub struct ForkArgs {
    #[arg(help = "The name of the project to fork.")]
    pub project: String,
    #[arg(short, long, default_value = "basic")]
    #[arg(value_name = "tier")]
    #[arg(help = "Deployment tier.")]
    pub tier: Tier,

    #[command(subcommand)]
    fork_commands: ForkServiceCommands,
}

impl ForkArgs {
    pub async fn run(&self) -> Result<()> {
        let (fork_name, fork_block_number) = self.fork_config().await?;

        let tier = match &self.tier {
            Tier::Basic => DeploymentTier::basic,
        };

        let request_body = ForkDeployment::build_query(Variables {
            project: self.project.clone(),
            fork_name: fork_name.clone(),
            fork_block_number,
            tier,
            wait: Some(true),
        });

        let client = ApiClient::new();
        let res: Response<fork_deployment::ResponseData> = client.post(&request_body).await?;
        if let Some(errors) = res.errors.clone() {
            for err in errors {
                println!("Error: {}", err.message);
            }
        }

        if let Some(data) = res.data {
            println!("Fork success 🚀");
            if let KatanaConfig(config) = data.fork_deployment {
                println!("\nEndpoints:");
                println!("  RPC: {}", config.rpc);
                println!(
                    "\nStream logs with `slot deployments logs {} katana -f`",
                    fork_name
                );
            }
        }

        Ok(())
    }

    async fn fork_config(&self) -> Result<(String, u64)> {
        match &self.fork_commands {
            ForkServiceCommands::Katana(config) => {
                let block_number = if let Some(block_number) = config.fork_block_number {
                    block_number
                } else {
                    // Workaround to get latest block number. Perhaps Katana could default to latest if none is supplied
                    let rpc_client = JsonRpcClient::new(HttpTransport::new(Url::parse(&format!(
                        "https://api.cartridge.gg/x/{}/katana",
                        self.project
                    ))?));
                    rpc_client.block_number().await?
                };

                Ok((config.fork_name.clone(), block_number))
            }
        }
    }
}
