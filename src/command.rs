pub mod auth;
pub mod deployments;

use anyhow::Result;
use clap::Subcommand;

use auth::Auth;
use deployments::Deployments;

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(subcommand)]
    #[command(about = "Manage auth credentials for the Slot CLI.")]
    Auth(Auth),
    #[command(subcommand)]
    #[command(about = "Manage Slot deployments.")]
    Deployments(Deployments),
}

impl Command {
    pub async fn run(&self) -> Result<()> {
        match &self {
            Command::Auth(cmd) => cmd.run().await,
            Command::Deployments(cmd) => cmd.run().await,
        }
    }
}
