use anyhow::Result;
use clap::Args;
use graphql_client::{GraphQLQuery, Response};
use me::{ResponseData, Variables};

use crate::api::Client;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/command/auth/info.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct Me;

#[derive(Debug, Args)]
pub struct InfoArgs {}

impl InfoArgs {
    // TODO: find the account info from `credentials.json` first before making a request
    pub async fn run(&self) -> Result<()> {
        let request_body = Me::build_query(Variables {});

        let client = Client::new();
        let res: Response<ResponseData> = client.query(&request_body).await?;
        if let Some(errors) = res.errors {
            for err in errors {
                println!("Error: {}", err.message);
            }
            return Ok(());
        }

        print!("{:?}", res.data.unwrap());

        Ok(())
    }
}
