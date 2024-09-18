#![allow(clippy::enum_variant_names)]

use anyhow::Result;
use clap::Args;
use slot::graphql::deployments::fork_deployment::ForkDeploymentForkDeployment::KatanaConfig;
use slot::graphql::deployments::{fork_deployment::*, ForkDeployment};
use slot::graphql::GraphQLQuery;
use slot::{api::Client, credential::Credentials, vars};
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider};
use url::Url;

use super::{services::ForkServiceCommands, Tier};

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
            Tier::Common => DeploymentTier::common,
            Tier::Rare => DeploymentTier::rare,
            Tier::Epic => DeploymentTier::epic,
        };

        let request_body = ForkDeployment::build_query(Variables {
            project: self.project.clone(),
            fork_name: fork_name.clone(),
            fork_block_number,
            tier,
            wait: Some(true),
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let data: ResponseData = client.query(&request_body).await?;
        println!("Fork success 🚀");
        if let KatanaConfig(config) = data.fork_deployment {
            println!("\nEndpoints:");
            println!("  RPC: {}", config.rpc);
            println!(
                "\nStream logs with `slot deployments logs {} katana -f`",
                fork_name
            );
        }

        Ok(())
    }

    async fn fork_config(&self) -> Result<(String, u64)> {
        match &self.fork_commands {
            ForkServiceCommands::Katana(config) => {
                let block_number = if let Some(block_number) = config.fork_block_number {
                    block_number
                } else {
                    let url = vars::get_cartridge_api_url();
                    // Workaround to get latest block number. Perhaps Katana could default to latest if none is supplied
                    let rpc_client = JsonRpcClient::new(HttpTransport::new(Url::parse(&format!(
                        "{url}/x/{}/katana",
                        self.project
                    ))?));
                    rpc_client.block_number().await?
                };

                Ok((config.fork_name.clone(), block_number))
            }
        }
    }
}
