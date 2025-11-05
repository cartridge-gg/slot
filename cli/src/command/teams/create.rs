use anyhow::{bail, Result};
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::team::create_team;
use slot::graphql::team::CreateTeam;
use slot::graphql::GraphQLQuery;
use slot::utils::is_valid_email;

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Team create options")]
pub struct CreateTeamArgs {
    #[arg(long)]
    #[arg(help = "The email address for team notifications.")]
    pub email: String,

    #[arg(long)]
    #[arg(help = "The physical address for the team.")]
    pub address: Option<String>,

    #[arg(long)]
    #[arg(help = "The tax ID for the team.")]
    pub tax_id: Option<String>,
}

impl CreateTeamArgs {
    pub async fn run(&self, team: String) -> Result<()> {
        // Validate email format
        if !is_valid_email(&self.email) {
            bail!("Invalid email format: {}", self.email);
        }

        let request_body = CreateTeam::build_query(create_team::Variables {
            name: team.clone(),
            input: Some(create_team::TeamInput {
                email: Some(self.email.clone()),
                address: self.address.clone(),
                tax_id: self.tax_id.clone(),
            }),
        });

        let credentials = Credentials::load()?;
        let client = Client::new_with_token(credentials.access_token);

        let data: create_team::ResponseData = client.query(&request_body).await?;

        println!("Team {} created successfully 🚀", data.create_team.name);

        Ok(())
    }
}
