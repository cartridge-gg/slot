use anyhow::Result;
use clap::Subcommand;

use self::{create::CreateArgs, logs::LogsArgs};

mod configs;
mod create;
mod logs;

#[derive(Subcommand, Debug)]
pub enum Deployment {
    #[command(about = "Create a new deployment.")]
    Create(CreateArgs),
    #[command(about = "Fetch logs for a deployment.")]
    Logs(LogsArgs),
}

impl Deployment {
    pub async fn run(&self) -> Result<()> {
        match &self {
            Deployment::Create(args) => args.run().await,
            Deployment::Logs(args) => args.run().await,
        }
    }
}
