use clap::Args;

use super::MachineSpecs;

#[derive(Clone, Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Torii options")]
pub struct Torii {
    #[arg(long)]
    #[arg(value_name = "rpc")]
    #[arg(help = "The Starknet RPC endpoint.")]
    rpc: String,
    #[arg(long)]
    #[arg(value_name = "manifest")]
    #[arg(help = "The manifest.")]
    manifest: String,
    #[command(flatten)]
    requests: MachineSpecs,
    #[arg(long)]
    #[arg(value_name = "storage")]
    #[arg(help = "Amount of storage to request.")]
    storage: u16,
    #[arg(long)]
    #[arg(value_name = "start_block")]
    #[arg(help = "The start block of the indexer.")]
    start_block: u64,
}
