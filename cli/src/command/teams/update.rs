use anyhow::Result;
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::team::update_team;
use slot::graphql::team::UpdateTeam;
use slot::graphql::GraphQLQuery;

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Team update options")]
pub struct UpdateTeamArgs {
    #[arg(long)]
    #[arg(help = "The email address for team notifications.")]
    pub email: Option<String>,
}

impl UpdateTeamArgs {
    pub async fn run(&self, team: String) -> Result<()> {
        let request_body = UpdateTeam::build_query(update_team::Variables {
            name: team.clone(),
            input: update_team::TeamInput {
                email: self.email.clone(),
            },
        });

        let credentials = Credentials::load()?;
        let client = Client::new_with_token(credentials.access_token);

        let _data: update_team::ResponseData = client.query(&request_body).await?;

        println!("Team {} updated successfully 🚀", team);

        Ok(())
    }
}
