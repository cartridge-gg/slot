#![allow(clippy::enum_variant_names)]

use anyhow::Result;
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::deployments::{katana_accounts::*, KatanaAccounts};
use slot::graphql::GraphQLQuery;

use serde_json::{json, Value};

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

        // NOTICE: Use the dev_predeployedAccounts since Katana is usually in dev mode.
        // This is a temporary solution until we have a way to get the accounts from the deployment.
        // This is also better than using the seed 0, since here the accounts addresses are already computed.

        // Use raw reqwest client to make the json rpc call:
        let client = reqwest::Client::new();
        let response = client
            .post(format!(
                "https://api.cartridge.gg/x/{}/katana",
                self.project
            ))
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "dev_predeployedAccounts",
                "params": [],
                "id": 1
            }))
            .send()
            .await?;

        let json_response = response.json::<Value>().await?;

        // If some accounts are found, print them. Otherwise, empty output.
        if let Some(result) = json_response.get("result").and_then(|r| r.as_array()) {
            for account in result {
                let address = account
                    .get("address")
                    .and_then(|a| a.as_str())
                    .unwrap_or("");
                let public_key = account
                    .get("publicKey")
                    .and_then(|pk| pk.as_str())
                    .unwrap_or("");

                println!("| Account Address |  {}", address);

                // Private key is optional.
                if let Some(private_key) = account.get("privateKey").and_then(|pk| pk.as_str()) {
                    println!("| Private key     |  {}", private_key);
                }

                println!("| Public key      |  {}", public_key);
                println!();
            }
        }

        Ok(())
    }
}
