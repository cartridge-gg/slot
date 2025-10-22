use anyhow::Result;
use clap::Subcommand;

use self::logs::LogsArgs;
use self::tokens::TokensCmd;
use self::whitelist::WhitelistCmd;

mod logs;
mod tokens;
mod whitelist;

/// Command group for managing RPC tokens and configurations
#[derive(Subcommand, Debug)]
pub enum RpcCmd {
    #[command(about = "Manage RPC tokens.", alias = "t")]
    Tokens(TokensCmd),

    #[command(about = "Manage origin whitelist.", alias = "w")]
    Whitelist(WhitelistCmd),

    #[command(about = "View RPC logs.", alias = "l")]
    Logs(LogsArgs),
}

impl RpcCmd {
    // Main entry point for the RPC command group
    pub async fn run(&self) -> Result<()> {
        match &self {
            RpcCmd::Tokens(cmd) => cmd.run().await,
            RpcCmd::Whitelist(cmd) => cmd.run().await,
            RpcCmd::Logs(args) => args.run().await,
        }
    }
}
