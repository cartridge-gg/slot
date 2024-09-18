use anyhow::{anyhow, Result};
use clap::Args;
use slot::graphql::auth::{me::*, Me};
use slot::graphql::{GraphQLQuery, Response};
use slot::{api::Client, credential::Credentials};

#[derive(Debug, Args)]
pub struct InfoArgs;

impl InfoArgs {
    // TODO: find the account info from `credentials.json` first before making a request
    pub async fn run(&self) -> Result<()> {
        let credentials = Credentials::load()?;
        let client = Client::new_with_token(credentials.access_token);

        let request_body = Me::build_query(Variables {});
        let res: Response<ResponseData> = client.query(&request_body).await?;

        if let Some(errors) = res.errors {
            for err in errors {
                println!("Error: {}", err.message);
            }
            return Err(anyhow!("API Error"));
        }

        print!("{:?}", res.data.unwrap());

        Ok(())
    }
}
