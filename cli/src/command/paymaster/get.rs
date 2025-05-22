use anyhow::{Ok, Result};
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::paymaster::get_paymaster;
use slot::graphql::paymaster::GetPaymaster;
use slot::graphql::GraphQLQuery;

use crate::command::paymaster::PolicyArgs;

#[derive(Debug, Args)]
#[command(next_help_heading = "Get paymaster options")]
pub struct GetArgs {}

impl GetArgs {
    pub async fn run(&self, name: String) -> Result<()> {
        // 1. Load Credentials
        let credentials = Credentials::load()?;

        // 2. Build Query Variables
        let variables = get_paymaster::Variables { name: name.clone() };
        let request_body = GetPaymaster::build_query(variables);

        // 3. Create Client
        let client = Client::new_with_token(credentials.access_token);

        // 4. Execute Query
        println!("Fetching paymaster: {}", name);
        let data: get_paymaster::ResponseData = client.query(&request_body).await?;

        // 5. Print Result (using Debug format as workaround for Serialize issue)
        match data.paymaster {
            Some(paymaster) => {
                let policies_list: Vec<_> = paymaster
                    .policies
                    .edges
                    .into_iter()
                    .flatten()
                    .filter_map(|edge| edge.unwrap().node)
                    .collect();

                if policies_list.is_empty() {
                    println!("No policies found for paymaster '{}'.", name);
                    return Ok(());
                }

                let policy_args: Vec<PolicyArgs> = policies_list
                    .iter()
                    .map(|p| PolicyArgs {
                        contract: p.contract_address.clone(),
                        entrypoint: p.entry_point.clone(),
                    })
                    .collect();

                super::print_policies_table(&policy_args);
            }
            None => {
                println!("Paymaster '{}' not found.", name);
            }
        }

        Ok(())
    }
}
