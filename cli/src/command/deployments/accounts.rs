#![allow(clippy::enum_variant_names)]

use std::str::FromStr;

use anyhow::Result;
use clap::Args;
use katana_primitives::contract::ContractAddress;
use katana_primitives::genesis::allocation::GenesisAccountAlloc;
use katana_primitives::genesis::allocation::{
    DevAllocationsGenerator, DevGenesisAccount, GenesisAccount,
};
use katana_primitives::genesis::Genesis;
use slot::graphql::deployments::katana_accounts::KatanaAccountsDeploymentConfig::KatanaConfig;
use slot::graphql::deployments::{katana_accounts::*, KatanaAccounts};
use slot::graphql::GraphQLQuery;
use starknet::core::types::Felt;

use slot::api::Client;
use slot::credential::Credentials;

use super::services::KatanaAccountCommands;

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

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let data: ResponseData = client.query(&request_body).await?;

        if let Some(deployment) = data.deployment {
            if let KatanaConfig(config) = deployment.config {
                match config.accounts {
                    Some(accounts) => {
                        let mut accounts_vec = Vec::new();
                        for account in accounts {
                            let address =
                                ContractAddress::new(Felt::from_str(&account.address).unwrap());

                            let public_key = Felt::from_str(&account.public_key).unwrap();
                            let private_key = Felt::from_str(&account.private_key).unwrap();

                            let genesis_account = GenesisAccount {
                                public_key,
                                ..GenesisAccount::default()
                            };

                            accounts_vec.push((
                                address,
                                GenesisAccountAlloc::DevAccount(DevGenesisAccount {
                                    private_key,
                                    inner: genesis_account,
                                }),
                            ));
                        }
                        print_genesis_accounts(accounts_vec.iter().map(|(a, b)| (a, b)), None);
                    }
                    None => {
                        let accounts = DevAllocationsGenerator::new(10)
                            .with_seed(parse_seed(&config.seed))
                            .generate();

                        let mut genesis = Genesis::default();
                        genesis
                            .extend_allocations(accounts.into_iter().map(|(k, v)| (k, v.into())));
                        print_genesis_accounts(genesis.accounts().peekable(), Some(&config.seed));
                    }
                };
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
