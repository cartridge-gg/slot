use anyhow::Result;
use clap::Subcommand;

// Import the structs defined in the subcommand files
use self::build::BuildArgs;
use self::create::CreateArgs;
mod build;
mod create;

#[derive(Subcommand, Debug)]
#[command(next_help_heading = "Merkle drops options")]
pub enum MerkleDropsCmd {
    #[command(about = "Build a merkle tree from onchain token holders.", aliases = ["b"])]
    Build(BuildArgs),
    #[command(about = "Create a new merkle drop.", aliases = ["c"])]
    Create(CreateArgs),
}

impl MerkleDropsCmd {
    pub async fn run(&self) -> Result<()> {
        match self {
            Self::Build(args) => args.run().await,
            Self::Create(args) => args.run().await,
        }
    }
}
