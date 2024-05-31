use std::{io::Write, path::PathBuf, process::Command, str::FromStr};

use anyhow::{Context, Result};
use clap::Args;
use katana_primitives::genesis::{self, json::GenesisJson};
use serde_json::json;
use starknet::core::types::FieldElement;
use tempfile::{tempdir, tempfile};

use crate::{
    command::auth::{info::InfoArgs, Auth},
    credential::Credentials,
};

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

    #[arg(long, short, value_name = "accounts")]
    #[arg(help = "Accounts.")]
    pub accounts: Option<i64>,

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

    #[arg(long, value_name = "local")]
    #[arg(help = "Instantiate the service locally.")]
    pub local: bool,
}

impl KatanaCreateArgs {
    pub async fn execute_local(&self) -> Result<()> {
        // 1. get the user controller address

        let account = Credentials::load()?;

        // TODO: the account type should already be using the proper types
        let controller_address = account.account.unwrap().contract_address.unwrap();
        dbg!(&controller_address);
        let controller_address = FieldElement::from_str(&controller_address)?;

        dbg!(&controller_address);

        // 2. inject the controller class into the genesis

        // 2.1. if user pass a genesis file, inject the controller class into the file
        // genesis.add_controller_account(controller_address);
        // 2.2. if user doesn't pass a genesis file, build a default genesis with the controller class in it

        let mut genesis = if let Some(ref json) = self.genesis {
            serde_json::from_str::<GenesisJson>(&json)?
        } else {
            let default_genesis = json!({
                "parentHash": "0x0",
                "stateRoot": "0x0",
                "timestamp": 0,
                "number": 0,
                "sequencerAddress": "0x6bd82a20984e638c8e1d45770e2924e274e315b9609eb15c26384eac0094cf1",
                "gasPrices": {
                    "ETH": 1000,
                    "STRK": 1000
                },
                "accounts": {
                },
                "classes": [],
                "feeToken": {
                    "name": "Ether",
                    "symbol": "ETH",
                    "address": "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
                    "decimals": 18
                },
                "universalDeployer": {}
            });

            serde_json::from_value::<GenesisJson>(default_genesis)?
        };

        let dir = tempdir()?;
        let path = dir.path().join("genesis.json");

        std::fs::write(&path, serde_json::to_string(&genesis)?)
            .context("failed to write genesis file to temporary file")?;

        // 3. instantiate the local katana with the custom genesis
        let mut process = Command::new("katana")
            .args(["--genesis", path.to_str().unwrap()])
            .spawn()?;

        process.wait()?;

        println!("katana is killed");

        Ok(())
    }
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

fn genesis_value_parser(path: &str) -> anyhow::Result<String> {
    let path = PathBuf::from(shellexpand::full(path)?.into_owned());
    let genesis = GenesisJson::load(path)?;
    let encoded = genesis::json::to_base64(genesis)?;
    Ok(String::from_utf8(encoded)?)
}
