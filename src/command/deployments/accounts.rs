#![allow(clippy::enum_variant_names)]

use anyhow::Result;
use base64::prelude::*;
use clap::Args;
use graphql_client::{GraphQLQuery, Response};
use katana_primitives::contract::ContractAddress;
use katana_primitives::genesis::allocation::DevAllocationsGenerator;
use katana_primitives::genesis::Genesis;
use katana_primitives::genesis::{allocation::GenesisAccountAlloc, json::GenesisJson};
use std::str::FromStr;

use crate::api::ApiClient;

use super::services::KatanaAccountCommands;

use self::katana_accounts::{
    KatanaAccountsDeploymentConfig::KatanaConfig, ResponseData, Variables,
};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/command/deployments/accounts.graphql",
    response_derives = "Debug"
)]
pub struct KatanaAccounts;

#[derive(Debug, Args)]
#[command(next_help_heading = "Accounts options")]
pub struct AccountsArgs {
    #[arg(help = "The name of the project.")]
    pub project: String,

    #[command(subcommand)]
    accounts_commands: KatanaAccountCommands,
}

impl AccountsArgs {
    pub async fn run(&self) -> Result<()> {
        let request_body = KatanaAccounts::build_query(Variables {
            project: self.project.clone(),
        });

        let client = ApiClient::new();
        let res: Response<ResponseData> = client.post(&request_body).await?;
        if let Some(errors) = res.errors.clone() {
            for err in errors {
                println!("Error: {}", err.message);
            }
        }

        if let Some(data) = res.data {
            if let Some(deployment) = data.deployment {
                if let KatanaConfig(config) = deployment.config {
                    // genesis overrides seed
                    if let Some(genesis) = config.genesis {
                        let decoded = BASE64_STANDARD.decode(genesis)?;
                        let json = GenesisJson::from_str(&String::from_utf8(decoded)?)?;
                        let genesis = Genesis::try_from(json)?;
                        print_genesis_accounts(genesis.accounts().peekable(), None);

                        return Ok(());
                    }

                    let total = match config.accounts {
                        Some(accounts) => accounts as u16,
                        None => 10,
                    };

                    let accounts = DevAllocationsGenerator::new(total)
                        .with_seed(parse_seed(&config.seed))
                        .generate();
                      
                    let mut genesis = Genesis::default();
                    genesis
                        .extend_allocations(accounts.into_iter().map(|(k, v)| (k, v.into())));
                    print_genesis_accounts(genesis.accounts().peekable(), Some(&config.seed));
                }
            }
        }

        Ok(())
    }
}

fn print_genesis_accounts<'a, Accounts>(accounts: Accounts, seed: Option<&str>)
where
    Accounts: Iterator<Item = (&'a ContractAddress, &'a GenesisAccountAlloc)>,
{
    println!(
        r"

PREFUNDED ACCOUNTS
=================="
    );

    for (addr, account) in accounts {
        if let Some(pk) = account.private_key() {
            println!(
                r"
| Account address |  {addr}
| Private key     |  {pk:#x}
| Public key      |  {:#x}",
                account.public_key()
            )
        } else {
            println!(
                r"
| Account address |  {addr}
| Public key      |  {:#x}",
                account.public_key()
            )
        }
    }

    if let Some(seed) = seed {
    println!(
      r"
ACCOUNTS SEED
=============
{seed}
"
  );
    }
}

fn parse_seed(seed: &str) -> [u8; 32] {
    let seed = seed.as_bytes();

    if seed.len() >= 32 {
        unsafe { *(seed[..32].as_ptr() as *const [u8; 32]) }
    } else {
        let mut actual_seed = [0u8; 32];
        seed.iter()
            .enumerate()
            .for_each(|(i, b)| actual_seed[i] = *b);
        actual_seed
    }
}
