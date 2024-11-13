#![allow(clippy::enum_variant_names)]

use super::services::UpdateServiceCommands;
use crate::command::deployments::Tier;
use anyhow::Result;
use clap::Args;
use katana_cli::NodeArgsConfig;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::deployments::update_deployment::UpdateDeploymentUpdateDeployment::{
    KatanaConfig, SayaConfig, ToriiConfig,
};
use slot::graphql::deployments::update_deployment::{
    self, UpdateKatanaConfigInput, UpdateServiceConfigInput, UpdateServiceInput,
    UpdateToriiConfigInput,
};
use slot::graphql::deployments::{update_deployment::*, UpdateDeployment};
use slot::graphql::GraphQLQuery;
use torii_cli::args::ToriiArgsConfig;

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
            UpdateServiceCommands::Katana(config) => {
                let service_config =
                    toml::to_string(&NodeArgsConfig::try_from(config.node_args.clone())?)?;

                UpdateServiceInput {
                    type_: DeploymentService::katana,
                    version: config.version.clone(),
                    config: Some(UpdateServiceConfigInput {
                        torii: None,
                        katana: Some(UpdateKatanaConfigInput {
                            config_file: Some(slot::read::base64_encode_string(&service_config)),
                        }),
                    }),
                }
            }
            UpdateServiceCommands::Torii(config) => {
                let service_config =
                    toml::to_string(&ToriiArgsConfig::try_from(config.torii_args.clone())?)?;

                UpdateServiceInput {
                    type_: DeploymentService::torii,
                    version: config.version.clone(),
                    config: Some(UpdateServiceConfigInput {
                        katana: None,
                        torii: Some(UpdateToriiConfigInput {
                            config_file: Some(slot::read::base64_encode_string(&service_config)),
                        }),
                    }),
                }
            }
            UpdateServiceCommands::Saya(config) => UpdateServiceInput {
                type_: DeploymentService::saya,
                version: config.version.clone(),
                config: Some(UpdateServiceConfigInput {
                    katana: None,
                    torii: None,
                }),
            },
        };

        let tier = match &self.tier {
            Tier::Basic => DeploymentTier::basic,
            Tier::Common => DeploymentTier::common,
            Tier::Rare => DeploymentTier::rare,
            Tier::Epic => DeploymentTier::epic,
        };

        let service_toml = service.get_config_toml()?;

        let request_body = UpdateDeployment::build_query(Variables {
            project: self.project.clone(),
            tier,
            service,
            wait: Some(true),
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let data: update_deployment::ResponseData = client.query(&request_body).await?;

        println!("Update success 🚀");

        let service = match &self.update_commands {
            UpdateServiceCommands::Katana(_) => "katana",
            UpdateServiceCommands::Torii(_) => "torii",
            UpdateServiceCommands::Saya(_) => "saya",
        };

        println!(
            "\nStream logs with `slot deployments logs {} {service} -f`",
            self.project
        );

        match data.update_deployment {
            SayaConfig(_) => {
                super::print_config_file("TODO");
            }
            ToriiConfig(_) => {
                // Currently, the infra returns None for the config file.
                // When hence use the config generated from the CLI:
                if let Some(config) = service_toml {
                    super::print_config_file(&config);
                }
            }
            KatanaConfig(_) => {
                // Once the infra returns the config, print it.
                if let Some(config) = service_toml {
                    super::print_config_file(&config);
                }
            }
        }

        Ok(())
    }
}
