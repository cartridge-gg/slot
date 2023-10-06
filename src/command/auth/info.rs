use anyhow::Result;
use clap::Args;
use graphql_client::{GraphQLQuery, Response};

use crate::api::ApiClient;

use self::me::{ResponseData, Variables};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/command/auth/info.graphql",
    response_derives = "Debug"
)]
pub struct Me;

#[derive(Debug, Args)]
pub struct InfoArgs {}

impl InfoArgs {
    pub async fn run(&self) -> Result<()> {
        let request_body = Me::build_query(Variables {});

        let client = ApiClient::new();
        let res: Response<ResponseData> = client.post(&request_body).await?;
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
