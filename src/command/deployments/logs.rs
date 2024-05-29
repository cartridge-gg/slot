use std::{
    collections::HashSet,
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

use crate::{
    api::Client, command::deployments::logs::deployment_logs::DeploymentService,
    credential::Credentials,
};

use self::deployment_logs::{DeploymentLogsDeploymentLogs, ResponseData, Variables};

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
            let logs = reader.query(self.since.clone(), self.limit).await?;
            println!("{}", logs.content);
        }

        Ok(())
    }
}

pub struct LogReader {
    client: Client,
    service: Service,
    project: String,
}

impl LogReader {
    pub fn new(service: Service, project: String) -> Self {
        let user = Credentials::load().unwrap();
        let client = Client::new_with_token(user.access_token);
        LogReader {
            client,
            service,
            project,
        }
    }

    pub async fn query(
        &self,
        since: Option<String>,
        limit: i64,
    ) -> Result<DeploymentLogsDeploymentLogs> {
        let service = match self.service {
            Service::Katana => DeploymentService::katana,
            Service::Torii => DeploymentService::torii,
            Service::Madara => DeploymentService::madara,
        };

        let request_body = DeploymentLogs::build_query(Variables {
            project: self.project.clone(),
            service,
            since,
            limit: Some(limit),
        });

        let res: Response<ResponseData> = self.client.query(&request_body).await?;
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

        Ok(logs)
    }

    pub async fn stream(&self, since: Option<String>) -> Result<()> {
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");

        let mut logs = self.query(since, 1).await?;
        let mut printed_logs = HashSet::new();

        let mut since = logs.until;
        while running.load(Ordering::SeqCst) {
            sleep(Duration::from_millis(1000)).await;
            logs = self.query(Some(since.clone()), 25).await?;

            if !printed_logs.contains(&logs.content) {
                println!("{}", logs.content);
                printed_logs.insert(logs.content.clone()); // Add the log to the buffer
            }

            since = logs.until
        }

        Ok(())
    }
}
