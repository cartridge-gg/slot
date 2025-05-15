#![allow(clippy::enum_variant_names)]

use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use katana_cli::file::NodeArgsConfig;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::deployments::create_deployment::*;
use slot::graphql::deployments::CreateDeployment;
use slot::graphql::GraphQLQuery;
use torii_cli::args::ToriiArgs;

use super::{services::CreateServiceCommands, Tier};

#[derive(Debug, Args)]
#[command(next_help_heading = "Create options")]
pub struct CreateArgs {
    #[arg(help = "The name of the project.")]
    pub project: String,

    #[arg(long, value_name = "team")]
    #[arg(help = "The name of the team. Defaults to a team named after your username.")]
    pub team: Option<String>,

    #[arg(short, long, default_value = "basic")]
    #[arg(value_name = "tier")]
    #[arg(help = "Deployment tier.")]
    pub tier: Tier,

    #[arg(long)]
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

    #[arg(help = "Force create confirmation", short('f'))]
    pub force: bool,
}

impl CreateArgs {
    pub async fn run(&self) -> Result<()> {
        let tier_pricing = vec![
            (Tier::Basic, "3"),
            (Tier::Common, "5"),
            (Tier::Epic, "15"),
            (Tier::Legendary, "35"),
            (Tier::Insane, "50"),
        ]
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();

        if self.tier != Tier::Basic {
            // billing
            if !self.force {
                let confirmation = Confirm::with_theme(&ColorfulTheme::default())
                  .with_prompt(format!(
                      "You are creating an of kind `{}`, which will cost you around ${} per month (billed daily). Do you want to proceed?",
                      &self.tier,
                      tier_pricing.get(&self.tier).unwrap()
                  ))
                  .default(false)
                  .show_default(true)
                  .wait_for_newline(true)
                  .interact()?;

                if !confirmation {
                    return Ok(());
                }
            }
        }

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
                    version: None,
                    config: slot::read::base64_encode_string(&service_config),
                    katana: Some(KatanaCreateInput {
                        provable: Some(config.provable),
                        network: config.network.clone(),
                        saya: Some(config.saya),
                    }),
                    torii: None,
                }
            }
            CreateServiceCommands::Torii(config) => {
                let service_config =
                    toml::to_string(&ToriiArgs::with_config_file(config.torii_args.clone())?)?;

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
                    torii: Some(ToriiCreateInput {
                        replicas: config.replicas,
                    }),
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
            team: self.team.clone(),
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
