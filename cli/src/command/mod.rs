pub mod auth;
pub mod deployments;
pub mod paymaster;
pub mod paymasters;
pub mod teams;

use anyhow::Result;
use clap::Subcommand;
use slot::version;

use auth::Auth;
use deployments::Deployments;
use paymaster::PaymasterCmd;
use paymasters::PaymastersCmd;
use teams::Teams;

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(subcommand)]
    #[command(about = "Manage auth credentials for the Slot CLI.", aliases = ["a"])]
    Auth(Auth),

    #[command(subcommand)]
    #[command(about = "Manage Slot deployments.", aliases = ["d"])]
    Deployments(Deployments),

    #[command(about = "Manage Slot team.", aliases = ["t"])]
    Teams(Teams),

    #[command(subcommand)]
    #[command(about = "Manage paymasters.", aliases = ["ps"])]
    Paymasters(PaymastersCmd),

    #[command(about = "Operate on a specific paymaster.", aliases = ["p"])]
    Paymaster(PaymasterCmd),
}

impl Command {
    pub async fn run(&self) -> Result<()> {
        // Check for new version and run auto-update if available
        version::check_and_auto_update();

        // Run the actual command
        match &self {
            Command::Auth(cmd) => cmd.run().await,
            Command::Deployments(cmd) => cmd.run().await,
            Command::Teams(cmd) => cmd.run().await,
            Command::Paymasters(cmd) => cmd.run().await,
            Command::Paymaster(cmd) => cmd.run().await,
        }
    }
}
