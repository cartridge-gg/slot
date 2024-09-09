use std::path::PathBuf;

use clap::Args;
use katana_primitives::genesis;
use katana_primitives::genesis::json::GenesisJson;

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Katana create options")]
pub struct KatanaCreateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,

    #[arg(long, short, value_name = "block_time")]
    #[arg(help = "Block time.")]
    pub block_time: Option<i64>,

    #[arg(long, value_name = "fork_rpc_url")]
    #[arg(help = "Fork RPC URL.")]
    pub fork_rpc_url: Option<String>,

    #[arg(long, value_name = "fork_block_number")]
    #[arg(help = "Fork block number.")]
    pub fork_block_number: Option<u64>,

    #[arg(long, short, value_name = "seed")]
    #[arg(help = "Seed.")]
    pub seed: Option<String>,

    #[arg(long, short, value_name = "accounts", default_value = "10")]
    #[arg(help = "Accounts.")]
    pub accounts: i64,

    #[arg(long, value_name = "invoke_max_steps")]
    #[arg(help = "Invoke Max Steps.")]
    pub invoke_max_steps: Option<u64>,

    #[arg(long, value_name = "validate_max_steps")]
    #[arg(help = "Validate Max Steps.")]
    pub validate_max_steps: Option<u64>,

    #[arg(long, value_name = "disable_fee")]
    #[arg(help = "Disable Fee.")]
    pub disable_fee: Option<bool>,

    #[arg(long, value_name = "gas_price")]
    #[arg(help = "Gas Price.")]
    pub gas_price: Option<u64>,

    #[arg(long, value_name = "PATH")]
    #[arg(help = "Path to a Katana genesis file.")]
    #[arg(value_parser = genesis_value_parser)]
    pub genesis: Option<String>,

    #[arg(long)]
    #[arg(help = "Enable Katana dev mode for specific endpoints.")]
    pub dev: bool,
}

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Katana update options")]
pub struct KatanaUpdateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,

    #[arg(long, short, value_name = "block_time")]
    #[arg(help = "Block time.")]
    pub block_time: Option<i64>,

    #[arg(long, short, value_name = "fork_rpc_url")]
    #[arg(help = "Fork RPC URL.")]
    pub fork_rpc_url: Option<String>,

    #[arg(long, short, value_name = "fork_block_number")]
    #[arg(help = "Fork Block Number.")]
    pub fork_block_number: Option<u64>,

    #[arg(long, value_name = "invoke_max_steps")]
    #[arg(help = "Invoke Max Steps.")]
    pub invoke_max_steps: Option<u64>,

    #[arg(long, value_name = "validate_max_steps")]
    #[arg(help = "Validate Max Steps.")]
    pub validate_max_steps: Option<u64>,

    #[arg(long, value_name = "disable_fee")]
    #[arg(help = "Disable Fee.")]
    pub disable_fee: Option<bool>,

    #[arg(long, value_name = "gas_price")]
    #[arg(help = "Gas Price.")]
    pub gas_price: Option<u64>,

    #[arg(long)]
    #[arg(help = "Enable Katana dev mode for specific endpoints.")]
    pub dev: bool,
}

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Katana fork options")]
pub struct KatanaForkArgs {
    #[arg(long, value_name = "fork_name")]
    #[arg(help = "Specify the fork name")]
    pub fork_name: String,
    #[arg(long, value_name = "fork_block_number")]
    #[arg(help = "Specify block number to fork. (latests if not provided)")]
    pub fork_block_number: Option<u64>,
}

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Katana account options")]
pub struct KatanaAccountArgs {}

fn genesis_value_parser(path: &str) -> Result<String, anyhow::Error> {
    let path = PathBuf::from(shellexpand::full(path)?.into_owned());
    let genesis = GenesisJson::load(path)?;
    let encoded = genesis::json::to_base64(genesis)?;
    Ok(String::from_utf8(encoded)?)
}
