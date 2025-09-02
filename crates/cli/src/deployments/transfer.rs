use anyhow::Result;
use clap::Args;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use slot_core::credentials::Credentials;
use slot_graphql::api::Client;
use slot_graphql::deployments::transfer_deployment::DeploymentService;
use slot_graphql::deployments::{transfer_deployment::*, TransferDeployment};
use slot_graphql::GraphQLQuery;

#[derive(clap::ValueEnum, Clone, Debug, serde::Serialize)]
pub enum Service {
    Katana,
    Torii,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Transfer options")]
pub struct TransferArgs {
    #[arg(help = "The name of the project.")]
    pub project: String,

    #[arg(help = "The name of the service.")]
    pub service: Service,

    #[arg(help = "The name of the team.")]
    pub team: String,

    #[arg(help = "Force Transfer without confirmation", short('f'))]
    pub force: bool,
}

impl TransferArgs {
    pub async fn run(&self) -> Result<()> {
        if !self.force {
            let confirmation = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Please confirm to transfer {} {:?} to {}",
                    &self.project, &self.service, &self.team
                ))
                .default(false)
                .show_default(true)
                .wait_for_newline(true)
                .interact()
                .unwrap();

            if !confirmation {
                return Ok(());
            }
        }

        let service = match &self.service {
            Service::Katana => DeploymentService::katana,
            Service::Torii => DeploymentService::torii,
        };

        let request_body = TransferDeployment::build_query(Variables {
            name: self.project.clone(),
            team: self.team.clone(),
            service,
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let _data: ResponseData = client.query(&request_body).await?;

        println!("Transfer success 🚀");

        Ok(())
    }
}
