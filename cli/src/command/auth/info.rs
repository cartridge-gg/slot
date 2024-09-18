use anyhow::Result;
use clap::Args;
use slot::graphql::auth::{me::*, Me};
use slot::graphql::GraphQLQuery;
use slot::{api::Client, credential::Credentials};

#[derive(Debug, Args)]
pub struct InfoArgs;

impl InfoArgs {
    // TODO: find the account info from `credentials.json` first before making a request
    pub async fn run(&self) -> Result<()> {
        let credentials = Credentials::load()?;
        let client = Client::new_with_token(credentials.access_token);

        let request_body = Me::build_query(Variables {});
        let res: ResponseData = client.query(&request_body).await?;
        print!("{:?}", res);

        Ok(())
    }
}
