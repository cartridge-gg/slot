use anyhow::Result;
use clap::Args;
use graphql_client::{GraphQLQuery, Response};

use crate::api::ApiClient;

use super::services::Service;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/command/deployments/logs.graphql",
    response_derives = "Debug"
)]
pub struct DeploymentLogs;

#[derive(Debug, Args)]
#[command(next_help_heading = "Deployment logs options")]
pub struct LogsArgs {
    #[arg(help = "The project of the deployment.")]
    pub project: String,

    #[arg(help = "The name of the deployment service.")]
    pub service: Service,

    #[arg(short, long = "limit", default_value = "25")]
    #[arg(help = "Display only the most recent `n` lines of logs.")]
    pub limit: i64,
}

impl LogsArgs {
    pub async fn run(&self) -> Result<()> {
        let service = match &self.service {
            Service::Katana => deployment_logs::DeploymentService::katana,
            Service::Torii => deployment_logs::DeploymentService::torii,
        };
        let request_body = DeploymentLogs::build_query(deployment_logs::Variables {
            project: self.project.clone(),
            service,
            cursor: "".to_string(),
            limit: self.limit,
        });

        let client = ApiClient::new();
        let res: Response<deployment_logs::ResponseData> = client.post(&request_body).await?;
        if let Some(errors) = res.errors {
            for err in errors {
                println!("Error: {}", err.message);
            }
            return Ok(());
        }

        let entries = res
            .data
            .and_then(|data| data.deployment)
            .map(|deployment| deployment.logs.entries);

        for e in entries.unwrap().iter() {
            if e.trim() == "{}" {
                println!("\n");
            } else {
                println!("{}", e);
            }
        }

        Ok(())
    }
}
