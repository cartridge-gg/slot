#![allow(clippy::enum_variant_names)]

use anyhow::Result;
use clap::Args;

use slot_core::credentials::Credentials;
use slot_graphql::api::Client;
use slot_graphql::deployments::list_deployments::{ResponseData, Variables};
use slot_graphql::deployments::ListDeployments;
use slot_graphql::GraphQLQuery;

#[derive(Debug, Args)]
#[command(next_help_heading = "List options")]
pub struct ListArgs {}

impl ListArgs {
    pub async fn run(&self) -> Result<()> {
        let request_body = ListDeployments::build_query(Variables {});

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let data: ResponseData = client.query(&request_body).await?;
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
                    .filter_map(|deployment| deployment.as_ref())
                    .filter(|deployment| {
                        deployment
                            .node
                            .as_ref()
                            .is_some_and(|node| format!("{:?}", node.status) != "deleted")
                    })
                    .collect();

                for deployment in deployments {
                    println!("Project: {}", deployment.node.as_ref().unwrap().project);
                    println!("Service: {}", deployment.node.as_ref().unwrap().service.id);
                    println!("---");
                }
            }
        }

        Ok(())
    }
}
