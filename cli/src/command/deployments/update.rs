#![allow(clippy::enum_variant_names)]

use super::services::UpdateServiceCommands;
use crate::command::deployments::Tier;
use anyhow::Result;
use clap::Args;
use katana_cli::file::NodeArgsConfig;
use katana_cli::NodeArgs;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::deployments::update_deployment::{self, UpdateServiceInput};
use slot::graphql::deployments::{update_deployment::*, UpdateDeployment};
use slot::graphql::GraphQLQuery;
use torii_cli::args::ToriiArgs;

#[derive(Debug, Args)]
#[command(next_help_heading = "Update options")]
pub struct UpdateArgs {
    #[arg(help = "The name of the project.")]
    pub project: String,

    #[arg(short, long)]
    #[arg(value_name = "tier")]
    #[arg(help = "Deployment tier.")]
    pub tier: Option<Tier>,

    #[command(subcommand)]
    update_commands: UpdateServiceCommands,
}

impl UpdateArgs {
    pub async fn run(&self) -> Result<()> {
        let service = match &self.update_commands {
            UpdateServiceCommands::Katana(args) => {
                let config = if let Some(config) = args.config.clone() {
                    let node_args = NodeArgs {
                        config: Some(config),
                        ..Default::default()
                    };

                    let service_config = toml::to_string(&NodeArgsConfig::try_from(node_args)?)?;

                    Some(slot::read::base64_encode_string(&service_config))
                } else {
                    None
                };

                UpdateServiceInput {
                    type_: DeploymentService::katana,
                    version: None,
                    config,
                    torii: None,
                }
            }
            UpdateServiceCommands::Torii(args) => {
                let config = if let Some(config) = args.config.clone() {
                    let torii_args = ToriiArgs {
                        config: Some(config),
                        ..Default::default()
                    };

                    let service_config = toml::to_string(&torii_args.with_config_file()?)?;
                    Some(slot::read::base64_encode_string(&service_config))
                } else {
                    None
                };

                UpdateServiceInput {
                    type_: DeploymentService::torii,
                    version: args.version.clone(),
                    config,
                    torii: Some(ToriiUpdateInput {
                        replicas: args.replicas,
                    }),
                }
            }
        };

        let tier = match &self.tier {
            None => None,
            Some(Tier::Basic) => Some(DeploymentTier::basic),
            Some(Tier::Common) => Some(DeploymentTier::common),
            Some(Tier::Epic) => Some(DeploymentTier::epic),
            Some(Tier::Legendary) => Some(DeploymentTier::legendary),
            Some(Tier::Insane) => Some(DeploymentTier::insane),
        };

        let request_body = UpdateDeployment::build_query(Variables {
            project: self.project.clone(),
            tier,
            service,
            wait: Some(true),
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let _: update_deployment::ResponseData = client.query(&request_body).await?;

        println!("Update success 🚀");

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
