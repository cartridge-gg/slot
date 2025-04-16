#![allow(clippy::enum_variant_names)]

use anyhow::Result;
use clap::Args;
use slot::graphql::deployments::{katana_accounts::*, KatanaAccounts};
use slot::graphql::GraphQLQuery;

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

        unimplemented!("fetch katana accounts")
    }
}
