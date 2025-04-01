#![allow(clippy::enum_variant_names)]

use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use katana_cli::file::NodeArgsConfig;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::deployments::create_deployment::*;
use slot::graphql::deployments::CreateDeployment;
use slot::graphql::GraphQLQuery;
use torii_cli::args::ToriiArgsConfig;

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

    #[arg(long)]
    #[arg(help = "Writes the service configuration to a file and exits without deploying.")]
    #[arg(global = true)]
    pub output_service_config: Option<PathBuf>,

    #[command(subcommand)]
    create_commands: CreateServiceCommands,
}

impl CreateArgs {
    pub async fn run(&self) -> Result<()> {
        let service = match &self.create_commands {
            CreateServiceCommands::Katana(config) => {
                config.validate()?;

                let service_config =
                    toml::to_string(&NodeArgsConfig::try_from(config.node_args.clone())?)?;

                if let Some(path) = &self.output_service_config {
                    std::fs::write(path, &service_config)?;
                    println!("Service config written to {}", path.display());
                    return Ok(());
                }

                CreateServiceInput {
                    type_: DeploymentService::katana,
                    version: config.version.clone(),
                    config: slot::read::base64_encode_string(&service_config),
                    katana: Some(KatanaCreateInput {
                        provable: Some(config.provable),
                        network: config.network.clone(),
                        saya: Some(config.saya),
                    }),
                }
            }
            CreateServiceCommands::Torii(config) => {
                let service_config =
                    toml::to_string(&ToriiArgsConfig::try_from(config.torii_args.clone())?)?;

                if let Some(path) = &self.output_service_config {
                    std::fs::write(path, &service_config)?;
                    println!("Service config written to {}", path.display());
                    return Ok(());
                }

                CreateServiceInput {
                    type_: DeploymentService::torii,
                    version: config.version.clone(),
                    config: slot::read::base64_encode_string(&service_config),
                    katana: None,
                }
            }
        };

        let tier = match &self.tier {
            Tier::Basic => DeploymentTier::basic,
            Tier::Common => DeploymentTier::common,
            Tier::Epic => DeploymentTier::epic,
            Tier::Legendary => DeploymentTier::legendary,
            Tier::Insane => DeploymentTier::insane,
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

        let service = match &self.create_commands {
            CreateServiceCommands::Katana(_) => "katana",
            CreateServiceCommands::Torii(_) => "torii",
        };

        println!(
            "Deploying {} ...",
            super::service_url(&self.project, service)
        );

        let _: ResponseData = client.query(&request_body).await?;

        println!("\nDeployment success 🚀");
        println!(
            "\nStream logs with `slot deployments logs {} {service} -f`",
            self.project
        );

        Ok(())
    }
}
