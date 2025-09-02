#![allow(clippy::enum_variant_names)]

use anyhow::Result;
use clap::Args;
use katana_primitives::contract::ContractAddress;
use katana_primitives::genesis::allocation::{DevAllocationsGenerator, GenesisAccountAlloc};
use katana_primitives::genesis::Genesis;
use slot_graphql::deployments::{katana_accounts::*, KatanaAccounts};
use slot_graphql::GraphQLQuery;

use slot_core::credentials::Credentials;
use slot_graphql::api::Client;

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

        let _data: ResponseData = client.query(&request_body).await?;

        // TODO use data.deployment.config and parse `accounts`
        // let mut accounts_vec = Vec::new();
        // for account in accounts {
        //     let address =
        //         ContractAddress::new(Felt::from_str(&account.address).unwrap());
        //
        //     let public_key = Felt::from_str(&account.public_key).unwrap();
        //     let private_key = Felt::from_str(&account.private_key).unwrap();
        //
        //     let genesis_account = GenesisAccount {
        //         public_key,
        //         ..GenesisAccount::default()
        //     };
        //
        //     accounts_vec.push((
        //         address,
        //         GenesisAccountAlloc::DevAccount(DevGenesisAccount {
        //             private_key,
        //             inner: genesis_account,
        //         }),
        //     ));
        // }
        // print_genesis_accounts(accounts_vec.iter().map(|(a, b)| (a, b)), None);

        // NOTICE: This is implementation assume that the Katana instance is configured with the default seed and total number of accounts. If not, the
        // generated addresses will be different from the ones in the Katana instance. This is rather a hack until `slot` can return the addresses directly (or
        // at least the exact configurations of the instance).

        let seed = "0";
        let total_accounts = 10;

        let accounts = DevAllocationsGenerator::new(total_accounts)
            .with_seed(parse_seed(seed))
            .generate();

        let mut genesis = Genesis::default();
        genesis.extend_allocations(accounts.into_iter().map(|(k, v)| (k, v.into())));
        print_genesis_accounts(genesis.accounts().peekable(), Some(seed));

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

// Mimic how Katana parse the seed to generate the predeployed accounts
// https://github.com/dojoengine/dojo/blob/85c0b025f108bd1ed64a5b35cfb574f61545a0ff/crates/katana/cli/src/utils.rs#L24-L34
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
