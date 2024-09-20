use anyhow::Result;
use clap::Args;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use slot::graphql::deployments::{delete_deployment::*, DeleteDeployment};
use slot::graphql::GraphQLQuery;
use slot::{api::Client, credential::Credentials};

#[derive(clap::ValueEnum, Clone, Debug, serde::Serialize)]
pub enum Service {
    Katana,
    Torii,
    Saya,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Delete options")]
pub struct DeleteArgs {
    #[arg(help = "The name of the project.")]
    pub project: String,

    #[arg(help = "The name of the service.")]
    pub service: Service,

    #[arg(help = "Force delete without confirmation", short('f'))]
    pub force: bool,
}

impl DeleteArgs {
    pub async fn run(&self) -> Result<()> {
        if !self.force {
            let confirmation = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Please confirm to delete {} {:?}",
                    &self.project, &self.service
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
            Service::Saya => DeploymentService::saya,
        };

        let request_body = DeleteDeployment::build_query(Variables {
            project: self.project.clone(),
            service,
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let _data: ResponseData = client.query(&request_body).await?;

        println!("Delete success 🚀");

        Ok(())
    }
}
