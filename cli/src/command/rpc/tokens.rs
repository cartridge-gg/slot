use anyhow::Result;
use clap::{Args, Subcommand};

use self::create::CreateArgs;
use self::delete::DeleteArgs;
use self::list::ListArgs;

mod create;
mod delete;
mod list;

/// Command group for managing RPC tokens
#[derive(Debug, Args)]
pub struct TokensCmd {
    #[command(subcommand)]
    command: TokensSubcommand,
}

// Enum defining the specific token actions
#[derive(Subcommand, Debug)]
enum TokensSubcommand {
    #[command(about = "Create a new RPC token.", alias = "c")]
    Create(CreateArgs),

    #[command(about = "Delete an RPC token.", alias = "d")]
    Delete(DeleteArgs),

    #[command(about = "List RPC tokens.", alias = "l")]
    List(ListArgs),
}

impl TokensCmd {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            TokensSubcommand::Create(args) => args.run().await,
            TokensSubcommand::Delete(args) => args.run().await,
            TokensSubcommand::List(args) => args.run().await,
        }
    }
}
