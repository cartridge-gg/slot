use anyhow::Result;
use clap::{Args, Subcommand};

use self::add::AddArgs;
use self::list::ListArgs;
use self::remove::RemoveArgs;

mod add;
mod list;
mod remove;

/// Command group for managing origin whitelist
#[derive(Debug, Args)]
pub struct WhitelistCmd {
    #[command(subcommand)]
    command: WhitelistSubcommand,
}

// Enum defining the specific whitelist actions
#[derive(Subcommand, Debug)]
enum WhitelistSubcommand {
    #[command(about = "Add origin to whitelist.", alias = "a")]
    Add(AddArgs),

    #[command(about = "Remove origin from whitelist.", alias = "r")]
    Remove(RemoveArgs),

    #[command(about = "List whitelisted origins.", alias = "l")]
    List(ListArgs),
}

impl WhitelistCmd {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            WhitelistSubcommand::Add(args) => args.run().await,
            WhitelistSubcommand::Remove(args) => args.run().await,
            WhitelistSubcommand::List(args) => args.run().await,
        }
    }
}
