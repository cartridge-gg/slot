use clap::Args;
use starknet::core::types::FieldElement;

#[derive(Clone, Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Torii create options")]
pub struct ToriiCreateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,

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

#[derive(Clone, Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Torii update options")]
pub struct ToriiUpdateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,
}
