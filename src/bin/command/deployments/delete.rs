use anyhow::Result;
use clap::Args;
use slot::graphql::deployments::{delete_deployment::*, DeleteDeployment};
use slot::graphql::{GraphQLQuery, Response};
use slot::{api::Client, credential::Credentials};

#[derive(clap::ValueEnum, Clone, Debug, serde::Serialize)]
pub enum Service {
    Katana,
    Torii,
    Madara,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Delete options")]
pub struct DeleteArgs {
    #[arg(help = "The name of the project.")]
    pub project: String,

    #[arg(help = "The name of the service.")]
    pub service: Service,
}

impl DeleteArgs {
    pub async fn run(&self) -> Result<()> {
        let service = match &self.service {
            Service::Katana => DeploymentService::katana,
            Service::Torii => DeploymentService::torii,
            Service::Madara => DeploymentService::madara,
        };

        let request_body = DeleteDeployment::build_query(Variables {
            project: self.project.clone(),
            service,
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let res: Response<ResponseData> = client.query(&request_body).await?;
        if let Some(errors) = res.errors.clone() {
            for err in errors {
                println!("Error: {}", err.message);
            }
        }

        if res.data.is_some() {
            println!("Delete success 🚀");
        }

        Ok(())
    }
}
