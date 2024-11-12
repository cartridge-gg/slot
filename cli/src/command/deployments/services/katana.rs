use std::path::PathBuf;

use clap::Args;
use katana_cli::NodeArgs;
use katana_primitives::genesis;
use katana_primitives::genesis::json::GenesisJson;

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Katana create options")]
pub struct KatanaCreateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,

    #[command(flatten)]
    pub node_args: NodeArgs,
}

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Katana update options")]
pub struct KatanaUpdateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,

    #[command(flatten)]
    pub node_args: NodeArgs,
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
