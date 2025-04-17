use anyhow::Result;
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::team::{teams_list, TeamsList};
use slot::graphql::GraphQLQuery;

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "List all teams")]
pub struct TeamListAllArgs {}

impl TeamListAllArgs {
    pub async fn run(&self) -> Result<()> {
        let request_body = TeamsList::build_query(teams_list::Variables {});

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let data: teams_list::ResponseData = client.query(&request_body).await?;

        data.me
            .unwrap()
            .teams
            .edges
            .into_iter()
            .flatten()
            .for_each(|edge| {
                if let Some(node) = edge.and_then(|edge| edge.node) {
                    println!("  {}", node.name)
                }
            });

        Ok(())
    }
}
