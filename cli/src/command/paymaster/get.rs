use anyhow::{Ok, Result};
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::paymaster::get_paymaster;
use slot::graphql::paymaster::GetPaymaster;
use slot::graphql::GraphQLQuery;

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
            Some(paymaster_data) => {
                println!("Paymaster details:\n{:?}", paymaster_data);
            }
            None => {
                println!("Paymaster '{}' not found.", name);
            }
        }

        Ok(())
    }
}
