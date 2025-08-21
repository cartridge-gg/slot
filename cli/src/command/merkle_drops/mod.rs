use anyhow::Result;
use clap::Subcommand;

// Import the structs defined in the subcommand files
use self::create::CreateArgs;
mod create;

#[derive(Subcommand, Debug)]
#[command(next_help_heading = "Merkle drops options")]
pub enum MerkleDropsCmd {
    #[command(about = "Create a new merkle drop.", aliases = ["c"])]
    Create(CreateArgs),
}

impl MerkleDropsCmd {
    pub async fn run(&self) -> Result<()> {
        match self {
            Self::Create(args) => args.run().await,
        }
    }
}
