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
        let info = res.me.clone().unwrap();
        println!("Username: {}", info.username);
        println!(
            "Credits: {} (${})",
            info.credits_plain,
            info.credits_plain / 100
        );

        println!();
        println!("Teams:");
        let teams = res.me.unwrap().teams.edges.unwrap();

        if teams.is_empty() {
            println!("  No teams yet");
        }

        for edge in teams {
            let team = edge.unwrap().node.unwrap();
            println!();
            println!("  Name: {}", team.name);

            println!("  Deployments:");
            let deployments = team.deployments.edges.unwrap();

            if deployments.is_empty() {
                println!("    No deployments yet");
            }

            for edge in deployments {
                let deployment = edge.unwrap().node.unwrap();
                println!(
                    "    Deployment: {}/{}",
                    deployment.project, deployment.service_id
                );
            }

            println!("  Members:");
            let members = team.membership.edges.unwrap();
            for edge in members {
                let member = edge.unwrap().node.unwrap();
                println!(
                    "    Member: {} ({:?})",
                    member.account.username, member.role
                );
            }
        }

        Ok(())
    }
}
