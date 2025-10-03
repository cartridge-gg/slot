use self::list::ListArgs;
use anyhow::Result;
use clap::Subcommand;
mod list;

#[derive(Subcommand, Debug)]
#[command(next_help_heading = "Paymasters options")]
pub enum PaymastersCmd {
    #[command(about = "List paymasters for the current user.", aliases = ["ls"])]
    List(ListArgs),
}

impl PaymastersCmd {
    pub async fn run(&self) -> Result<()> {
        match self {
            Self::List(args) => args.run().await,
        }
    }
}
