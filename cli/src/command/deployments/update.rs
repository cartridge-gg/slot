#![allow(clippy::enum_variant_names)]

use super::services::UpdateServiceCommands;
use crate::command::deployments::Tier;
use anyhow::Result;
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::deployments::update_deployment::{self, UpdateServiceInput};
use slot::graphql::deployments::{update_deployment::*, UpdateDeployment};
use slot::graphql::GraphQLQuery;

#[derive(Debug, Args)]
#[command(next_help_heading = "Update options")]
pub struct UpdateArgs {
    #[arg(help = "The name of the project.")]
    pub project: String,

    #[arg(short, long)]
    #[arg(value_name = "tier")]
    #[arg(help = "Deployment tier.")]
    pub tier: Option<Tier>,

    #[arg(long)]
    #[arg(help = "Enable observability for monitoring and metrics.")]
    pub observability: Option<bool>,

    #[command(subcommand)]
    update_commands: UpdateServiceCommands,
}

impl UpdateArgs {
    pub async fn run(&self) -> Result<()> {
        let service = match &self.update_commands {
            UpdateServiceCommands::Katana(args) => {
                let config = if let Some(config) = args.config.clone() {
                    // Read the raw config file content
                    let service_config = std::fs::read_to_string(&config)?;
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
                    // Read the raw config file content
                    let service_config = std::fs::read_to_string(&config)?;
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
            Some(Tier::Pro) => Some(DeploymentTier::pro),
            Some(Tier::Epic) => Some(DeploymentTier::epic),
            Some(Tier::Legendary) => Some(DeploymentTier::legendary),
            Some(Tier::Insane) => Some(DeploymentTier::insane), // deprecated tier, kept for backwards compatibility
        };

        let request_body = UpdateDeployment::build_query(Variables {
            project: self.project.clone(),
            tier,
            service,
            wait: Some(true),
            observability: self.observability,
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let response: update_deployment::ResponseData = client.query(&request_body).await?;

        let service = match &self.update_commands {
            UpdateServiceCommands::Katana(_) => "katana",
            UpdateServiceCommands::Torii(_) => "torii",
        };

        println!("Update success 🚀");

        // Display observability secret if present
        if let Some(observability_secret) = &response.update_deployment.observability_secret {
            super::print_observability_secret(observability_secret, &self.project, service);
        }

        // Show replica sync note for torii deployments with replicas > 1
        if let UpdateServiceCommands::Torii(args) = &self.update_commands {
            if let Some(replicas) = args.replicas {
                if replicas > 1 {
                    println!("\n⚠️  Note: Deployment will not be active until fully synced due to replica mode.");
                    println!("   This may take several minutes depending on the database size.");
                }
            }
        }

        println!(
            "\nStream logs with `slot deployments logs {} {service} -f`",
            self.project
        );

        Ok(())
    }
}
