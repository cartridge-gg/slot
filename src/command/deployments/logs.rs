use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::Result;
use clap::Args;
use graphql_client::{GraphQLQuery, Response};
use tokio::time::sleep;

use crate::{api::ApiClient, command::deployments::logs::deployment_logs::DeploymentService};

use self::deployment_logs::{ResponseData, Variables};

use super::services::Service;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/command/deployments/logs.graphql",
    response_derives = "Debug"
)]
pub struct DeploymentLogs;

type Time = String;

#[derive(Debug, Args)]
#[command(next_help_heading = "Deployment logs options")]
pub struct LogsArgs {
    #[arg(help = "The project of the deployment.")]
    pub project: String,

    #[arg(help = "The name of the deployment service.")]
    pub service: Service,

    #[arg(short, long = "since")]
    #[arg(help = "Display logs after this RFC3339 timestamp.")]
    pub since: Option<String>,

    #[arg(short, long = "limit", default_value = "25")]
    #[arg(help = "Display only the most recent `n` lines of logs.")]
    pub limit: i64,

    #[arg(short, long = "follow", default_value = "false")]
    #[arg(help = "Stream service logs.")]
    pub follow: bool,
}

impl LogsArgs {
    pub async fn run(&self) -> Result<()> {
        let reader = LogReader::new(self.service.clone(), self.project.clone());

        if self.follow {
            reader.stream(self.since.clone()).await?;
        } else {
            reader.query(self.since.clone(), self.limit).await?;
        }

        Ok(())
    }
}

pub struct LogReader {
    client: ApiClient,
    service: Service,
    project: String,
}

impl LogReader {
    pub fn new(service: Service, project: String) -> Self {
        LogReader {
            client: ApiClient::new(),
            service,
            project,
        }
    }

    pub async fn query(&self, since: Option<String>, limit: i64) -> Result<String> {
        let service = match self.service {
            Service::Katana => DeploymentService::katana,
            Service::Torii => DeploymentService::torii,
        };

        let request_body = DeploymentLogs::build_query(Variables {
            project: self.project.clone(),
            service,
            since,
            limit: Some(limit),
        });

        let res: Response<ResponseData> = self.client.post(&request_body).await?;
        if let Some(errors) = res.errors {
            let error_message = errors
                .into_iter()
                .map(|err| err.message)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(anyhow::anyhow!(error_message));
        }

        let logs = res
            .data
            .and_then(|data| data.deployment)
            .map(|deployment| deployment.logs)
            .unwrap();

        println!("{}", logs.content);

        Ok(logs.until)
    }

    pub async fn stream(&self, since: Option<String>) -> Result<()> {
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");

        let mut since = self.query(since, 25).await?;
        while running.load(Ordering::SeqCst) {
            println!("{since}");
            sleep(Duration::from_millis(1000)).await;
            since = self.query(Some(since.clone()), 25).await?;
        }

        Ok(())
    }
}
