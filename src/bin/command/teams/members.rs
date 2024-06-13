use anyhow::Result;
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::team::{
    team_member_add, team_member_remove, team_members_list, TeamMemberAdd, TeamMemberRemove,
    TeamMembersList,
};
use slot::graphql::{GraphQLQuery, Response};

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Team list options")]
pub struct TeamListArgs;

impl TeamListArgs {
    pub async fn run(&self, team: String) -> Result<()> {
        let request_body =
            TeamMembersList::build_query(team_members_list::Variables { team: team.clone() });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let res: Response<team_members_list::ResponseData> = client.query(&request_body).await?;
        if let Some(errors) = res.errors.clone() {
            for err in errors {
                println!("Error: {}", err.message);
            }

            return Ok(());
        }

        if let Some(data) = res.data {
            println!("{} members:", team);
            data.team
                .and_then(|team_list| team_list.members.edges)
                .into_iter()
                .flatten()
                .for_each(|edge| {
                    if let Some(node) = edge.and_then(|edge| edge.node) {
                        println!("  {}", node.id)
                    }
                });
        }

        Ok(())
    }
}

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Team add options")]
pub struct TeamAddArgs {
    #[arg(help = "Name of the team member to add.")]
    pub account: Vec<String>,
}

impl TeamAddArgs {
    pub async fn run(&self, team: String) -> Result<()> {
        let request_body = TeamMemberAdd::build_query(team_member_add::Variables {
            team,
            accounts: self.account.clone(),
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let res: Response<team_member_add::ResponseData> = client.query(&request_body).await?;
        if let Some(errors) = res.errors {
            for err in errors {
                println!("Error: {}", err.message);
            }

            return Ok(());
        }

        Ok(())
    }
}

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Team remove options")]
pub struct TeamRemoveArgs {
    #[arg(help = "Name of the team member to add.")]
    pub account: Vec<String>,
}

impl TeamRemoveArgs {
    pub async fn run(&self, team: String) -> Result<()> {
        let request_body = TeamMemberRemove::build_query(team_member_remove::Variables {
            team,
            accounts: self.account.clone(),
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let res: Response<team_member_remove::ResponseData> = client.query(&request_body).await?;
        if let Some(errors) = res.errors {
            for err in errors {
                println!("Error: {}", err.message);
            }

            return Ok(());
        }

        Ok(())
    }
}
