use anyhow::Result;
use clap::Args;
use slot::api::Client;
use slot::graphql::team::delete_team;

#[derive(Debug, Args)]
pub struct DeleteTeamArgs {}

impl DeleteTeamArgs {
    pub async fn run(&self, name: String) -> Result<()> {
        let client = Client::new()?;

        let variables = delete_team::Variables { name };
        let response = client.graphql::<delete_team::DeleteTeam>(variables).await?;

        if response.delete_team {
            println!("Team '{}' deleted successfully", name);
        } else {
            println!("Failed to delete team '{}'", name);
        }

        Ok(())
    }
}
