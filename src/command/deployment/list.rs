#![allow(clippy::enum_variant_names)]

use anyhow::Result;
use clap::Args;
use graphql_client::{GraphQLQuery, Response};

use crate::api::ApiClient;

use self::list_deployments::{ResponseData, Variables};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/command/deployment/list.graphql",
    response_derives = "Debug"
)]
pub struct ListDeployments;

#[derive(Debug, Args)]
#[command(next_help_heading = "List options")]
pub struct ListArgs {}

impl ListArgs {
    pub async fn run(&self) -> Result<()> {
        let request_body = ListDeployments::build_query(Variables {});

        let client = ApiClient::new();
        let res: Response<ResponseData> = client.post(&request_body).await?;
        if let Some(errors) = res.errors.clone() {
            for err in errors {
                println!("Error: {}", err.message);
            }
        }

        if let Some(data) = res.data {
            if let Some(me) = data.me {
                if let Some(teams) = me.teams.edges {
                    let teams: Vec<_> = teams
                        .iter()
                        .filter_map(|team| team.as_ref())
                        .filter_map(|team| team.node.as_ref())
                        .collect::<_>();

                    let mut all_deployments = Vec::new();

                    for team in teams {
                        if let Some(deployments) = &team.deployments.edges {
                            all_deployments.extend(deployments.clone());
                        }
                    }

                    println!("{:?}", all_deployments);
                }
            }
        }

        Ok(())
    }
}
