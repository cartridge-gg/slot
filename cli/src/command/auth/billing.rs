use anyhow::Result;
use clap::Args;
use slot::graphql::auth::{update_me::*, UpdateMe};
use slot::graphql::GraphQLQuery;
use slot::{api::Client, credential::Credentials};

#[derive(Debug, Args)]
#[command(next_help_heading = "Set billing options")]
pub struct BillingArgs {}

impl BillingArgs {
    pub async fn run(&self) -> Result<()> {
        let credentials = Credentials::load()?;
        let client = Client::new_with_token(credentials.access_token);

        let request_body = UpdateMe::build_query(Variables {
            email: None,
            slot_billing: Some(true),
        });
        let res: ResponseData = client.query(&request_body).await?;
        print!("{:?}", res);

        Ok(())
    }
}
