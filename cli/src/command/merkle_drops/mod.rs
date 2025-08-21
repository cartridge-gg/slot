use anyhow::Result;
use clap::Subcommand;

// Import the structs defined in the subcommand files
use self::create::CreateArgs;
use self::process::ProcessArgs;
use self::snapshot::SnapshotArgs;
mod create;
mod process;
mod snapshot;

#[derive(Subcommand, Debug)]
#[command(next_help_heading = "Merkle drops options")]
pub enum MerkleDropsCmd {
    #[command(about = "Create a snapshot of token holders from a single contract.", aliases = ["s"])]
    Snapshot(SnapshotArgs),
    #[command(about = "Process rewards from multiple snapshots.", aliases = ["p"])]
    Process(ProcessArgs),
    #[command(about = "Create a new merkle drop.", aliases = ["c"])]
    Create(CreateArgs),
}

impl MerkleDropsCmd {
    pub async fn run(&self) -> Result<()> {
        match self {
            Self::Snapshot(args) => args.run().await,
            Self::Process(args) => args.run().await,
            Self::Create(args) => args.run().await,
        }
    }
}
