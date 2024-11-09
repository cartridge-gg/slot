use clap::Args;
use starknet::core::types::Felt;

#[derive(Clone, Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Torii create options")]
pub struct ToriiCreateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,

    #[arg(long)]
    #[arg(value_name = "rpc")]
    #[arg(help = "The Starknet RPC endpoint.")]
    pub rpc: Option<String>,

    #[arg(long)]
    #[arg(value_name = "world")]
    #[arg(help = "World address.")]
    pub world: Option<Felt>,

    #[arg(long)]
    #[arg(help = "A config file ")]
    #[arg(value_name = "config-file")]
    pub config_file: Option<String>,

    #[arg(short, long)]
    #[arg(value_name = "contracts")]
    #[arg(help = "Contract addresses to index")]
    pub contracts: Option<String>,

    #[arg(short, long)]
    #[arg(help = "Specify a block to start indexing from.")]
    pub start_block: Option<u64>,

    #[arg(long)]
    #[arg(help = "Enable indexing pending blocks.")]
    pub index_pending: Option<bool>,

    #[arg(long)]
    #[arg(help = "Polling interval in milliseconds.")]
    pub polling_interval: Option<u64>,

    #[arg(long)]
    #[arg(value_name = "index_transactions")]
    #[arg(help = "Whether or not to index world transactions")]
    pub index_transactions: Option<bool>,

    #[arg(long)]
    #[arg(value_name = "index_raw_events")]
    #[arg(help = "Whether or not to index raw events")]
    pub index_raw_events: Option<bool>,
}

#[derive(Clone, Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Torii update options")]
pub struct ToriiUpdateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,
}
