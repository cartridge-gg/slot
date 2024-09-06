use clap::Args;
use starknet::core::types::Felt;

#[derive(Clone, Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Saya create options")]
pub struct SayaCreateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,

    #[arg(long)]
    #[arg(value_name = "mode")]
    #[arg(help = "The mode to use for the deployment.")]
    pub mode: String,

    #[arg(long)]
    #[arg(value_name = "rpc_url")]
    #[arg(help = "The Katana RPC endpoint.")]
    pub rpc_url: String,

    #[arg(long)]
    #[arg(value_name = "registry")]
    #[arg(help = "Registry address.")]
    pub registry: String,

    #[arg(long)]
    #[arg(value_name = "settlement_contract")]
    #[arg(help = "Settlement contract address.")]
    pub settlement_contract: String,

    #[arg(long)]
    #[arg(value_name = "world")]
    #[arg(help = "World address.")]
    pub world: Felt,

    #[arg(long)]
    #[arg(value_name = "start_block")]
    #[arg(help = "Specify a block to start indexing from.")]
    pub start_block: Option<i64>,

    #[arg(long)]
    #[arg(value_name = "prover_url")]
    #[arg(help = "Prover URL.")]
    pub prover_url: String,

    #[arg(long)]
    #[arg(value_name = "prover_key")]
    #[arg(help = "Store proofs.")]
    pub store_proofs: Option<bool>,

    #[arg(long)]
    #[arg(value_name = "starknet_url")]
    #[arg(help = "Starknet URL.")]
    pub starknet_url: String,

    #[arg(long)]
    #[arg(value_name = "signer_key")]
    #[arg(help = "Signer key.")]
    pub signer_key: String,

    #[arg(long)]
    #[arg(value_name = "signer_address")]
    #[arg(help = "Signer address.")]
    pub signer_address: String,

    #[arg(long)]
    #[arg(value_name = "private_key")]
    #[arg(help = "Private key.")]
    pub private_key: String,

    #[arg(long)]
    #[arg(value_name = "batch_size")]
    #[arg(help = "Batch size.")]
    pub batch_size: Option<i64>,
}

#[derive(Clone, Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Saya update options")]
pub struct SayaUpdateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,
}
