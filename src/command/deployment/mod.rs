use anyhow::Result;
use clap::Subcommand;

use self::{create::CreateArgs, describe::DescribeArgs, list::ListArgs, logs::LogsArgs};

mod create;
mod describe;
mod list;
mod logs;
mod services;

#[derive(Subcommand, Debug)]
pub enum Deployment {
    #[command(about = "Create a new deployment.")]
    Create(CreateArgs),
    #[command(about = "Describe a deployment's configuration.")]
    Describe(DescribeArgs),
    #[command(about = "List all deployments.")]
    List(ListArgs),
    #[command(about = "Fetch logs for a deployment.")]
    Logs(LogsArgs),
}

impl Deployment {
    pub async fn run(&self) -> Result<()> {
        match &self {
            Deployment::Create(args) => args.run().await,
            Deployment::Describe(args) => args.run().await,
            Deployment::List(args) => args.run().await,
            Deployment::Logs(args) => args.run().await,
        }
    }
}
