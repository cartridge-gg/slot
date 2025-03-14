use anyhow::Result;
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::GraphQLQuery;
use slot::graphql::team::CreateTeam;
use slot::graphql::team::team_member_add::Variables;
use slot::graphql::team::create_team;

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Team create options")]
pub struct CreateTeamArgs {
    #[arg(help = "Name of the team to create.")]
    pub name: String,
}

impl CreateTeamArgs {
    pub async fn run(&self, team: String) -> Result<()> {
        let request_body = CreateTeam::build_query(create_team::Variables {
            name: self.name.clone(),
        });

        let credentials = Credentials::load()?;
        let client = Client::new_with_token(credentials.access_token);

        let data: create_team::ResponseData = client.query(&request_body).await?;

        println!("Team created successfully 🚀");

        Ok(())
    }
}
