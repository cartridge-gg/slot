use clap::Args;
use starknet::core::types::FieldElement;

#[derive(Clone, Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Torii options")]
pub struct ToriiArgs {
    #[arg(long)]
    #[arg(value_name = "rpc")]
    #[arg(help = "The Starknet RPC endpoint.")]
    pub rpc: String,
    #[arg(long)]
    #[arg(value_name = "world")]
    #[arg(help = "World address.")]
    pub world: FieldElement,
    #[arg(short, long)]
    #[arg(help = "Specify a block to start indexing from.")]
    pub start_block: i64,
}
