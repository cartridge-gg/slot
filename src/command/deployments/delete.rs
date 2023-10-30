#![allow(clippy::enum_variant_names)]

use anyhow::Result;
use clap::Args;
use graphql_client::{GraphQLQuery, Response};

use crate::{
    api::ApiClient,
    command::deployments::delete::delete_deployment::{DeploymentService, Variables},
};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/command/deployments/delete.graphql",
    response_derives = "Debug"
)]
pub struct DeleteDeployment;

#[derive(clap::ValueEnum, Clone, Debug, serde::Serialize)]
pub enum Service {
    Katana,
    Torii,
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
        };

        let request_body = DeleteDeployment::build_query(Variables {
            project: self.project.clone(),
            service,
        });

        let client = ApiClient::new();
        let res: Response<delete_deployment::ResponseData> = client.post(&request_body).await?;
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
