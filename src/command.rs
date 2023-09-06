pub mod auth;
pub mod deployment;

use anyhow::Result;
use clap::Subcommand;

use auth::Auth;
use deployment::Deployment;

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(subcommand)]
    #[command(about = "Manage auth credentials for the Slot CLI.")]
    Auth(Auth),
    #[command(subcommand)]
    #[command(about = "Manage Slot deployments.")]
    Deployment(Deployment),
}

impl Command {
    pub async fn run(&self) -> Result<()> {
        match &self {
            Command::Auth(cmd) => cmd.run(),
            Command::Deployment(cmd) => cmd.run().await,
        }
    }
}
