use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

// use tokio::selectV
use anyhow::Result;
use clap::Args;
use slot::credential::Credentials;
use slot::graphql::deployments::deployment_logs::DeploymentService;
use slot::graphql::{deployments::deployment_logs::*, GraphQLQuery};
use slot::{api::Client, graphql::deployments::DeploymentLogs};

use super::services::Service;

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

    #[arg(short, long = "container")]
    #[arg(help = "Filter logs by container name.")]
    pub container: Option<String>,
}

impl LogsArgs {
    pub async fn run(&self) -> Result<()> {
        let reader = LogReader::new(self.service.clone(), self.project.clone());

        if self.follow {
            reader
                .stream(self.since.clone(), self.container.clone())
                .await?;
        } else {
            let logs = reader
                .query(self.since.clone(), self.limit, self.container.clone())
                .await?;
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
        container: Option<String>,
    ) -> Result<DeploymentLogsDeploymentLogs> {
        let service = match self.service {
            Service::Katana => DeploymentService::katana,
            Service::Torii => DeploymentService::torii,
        };

        let request_body = DeploymentLogs::build_query(Variables {
            project: self.project.clone(),
            service,
            since,
            limit: Some(limit),
            container,
        });

        let data: ResponseData = self.client.query(&request_body).await?;

        let logs = data.deployment.map(|deployment| deployment.logs).unwrap();

        Ok(logs)
    }

    pub async fn stream(&self, since: Option<String>, container: Option<String>) -> Result<()> {
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");

        let mut logs = self.query(since, 1, container.clone()).await?;
        let mut printed_logs = HashSet::new();

        let mut since = logs.until;
        while running.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(1000)).await;
            logs = self
                .query(Some(since.clone()), 25, container.clone())
                .await?;

            if !printed_logs.contains(&logs.content) {
                println!("{}", logs.content);
                printed_logs.insert(logs.content.clone()); // Add the log to the buffer
            }

            since = logs.until
        }

        Ok(())
    }
}
