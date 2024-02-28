use anyhow::Result;
use clap::Subcommand;

use self::{
    create::CreateArgs, delete::DeleteArgs, describe::DescribeArgs, fork::ForkArgs, list::ListArgs,
    logs::LogsArgs, update::UpdateArgs,
};

mod create;
mod delete;
mod describe;
mod fork;
mod list;
mod logs;
mod services;
mod update;

type Long = u64;

#[derive(Subcommand, Debug)]
pub enum Deployments {
    #[command(about = "Create a new deployment.")]
    Create(CreateArgs),
    #[command(about = "Delete a deployment.")]
    Delete(DeleteArgs),
    #[command(about = "Update a deployment.")]
    Update(UpdateArgs),
    #[command(about = "Fork a deployment.")]
    Fork(ForkArgs),
    #[command(about = "Describe a deployment's configuration.")]
    Describe(DescribeArgs),
    #[command(about = "List all deployments.", aliases = ["ls"])]
    List(ListArgs),
    #[command(about = "Fetch logs for a deployment.")]
    Logs(LogsArgs),
}

impl Deployments {
    pub async fn run(&self) -> Result<()> {
        match &self {
            Deployments::Create(args) => args.run().await,
            Deployments::Delete(args) => args.run().await,
            Deployments::Update(args) => args.run().await,
            Deployments::Fork(args) => args.run().await,
            Deployments::Describe(args) => args.run().await,
            Deployments::List(args) => args.run().await,
            Deployments::Logs(args) => args.run().await,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Debug, serde::Serialize)]
pub enum Tier {
    Basic,
}
