#![allow(clippy::enum_variant_names)]

use anyhow::Result;
use clap::Args;
use graphql_client::{GraphQLQuery, Response};

use crate::api::ApiClient;

use self::list_deployments::{ResponseData, Variables};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/command/deployments/list.graphql",
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

                    let deployments: Vec<_> = teams
                        .iter()
                        .filter_map(|team| team.deployments.edges.as_ref())
                        .flatten()
                        .map(|deployment| deployment.as_ref().unwrap())
                        .collect();

                    for deployment in deployments {
                        println!("Project: {}", deployment.node.as_ref().unwrap().project);
                        println!("Service: {}", deployment.node.as_ref().unwrap().service.id);
                        println!("---");
                    }
                }
            }
        }

        Ok(())
    }
}
