use anyhow::Result;
use clap::Args;
use slot_core::credentials::Credentials;
use slot_graphql::api::Client;
use slot_graphql::team::delete_team;
use slot_graphql::team::DeleteTeam;
use slot_graphql::GraphQLQuery;

#[derive(Debug, Args)]
pub struct DeleteTeamArgs {}

impl DeleteTeamArgs {
    pub async fn run(&self, name: String) -> Result<()> {
        let request_body = DeleteTeam::build_query(delete_team::Variables { name: name.clone() });

        let credentials = Credentials::load()?;
        let client = Client::new_with_token(credentials.access_token);

        let data: delete_team::ResponseData = client.query(&request_body).await?;

        if data.delete_team {
            println!("Team '{}' deleted successfully", name);
        } else {
            println!("Failed to delete team '{}'", name);
        }

        Ok(())
    }
}
