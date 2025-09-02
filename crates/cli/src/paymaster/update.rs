use anyhow::Result;
use clap::Args;
use slot_core::credentials::Credentials;
use slot_graphql::api::Client;
use slot_graphql::paymaster::update_paymaster;
use slot_graphql::paymaster::UpdatePaymaster;
use slot_graphql::GraphQLQuery;

#[derive(Debug, Args)]
#[command(next_help_heading = "Update paymaster options")]
pub struct UpdateArgs {
    #[arg(long, help = "New name for the paymaster.")]
    name: Option<String>,
    #[arg(long, help = "New team name to associate the paymaster with.")]
    team: Option<String>,
    #[arg(long, help = "Set the active state of the paymaster.")]
    active: Option<bool>,
}

impl UpdateArgs {
    pub async fn run(&self, current_name: String) -> Result<()> {
        // Check if any update parameters are provided
        if self.name.is_none() && self.team.is_none() && self.active.is_none() {
            return Err(anyhow::anyhow!(
                "No update parameters provided. Use --name, --team, or --active to specify what to update."
            ));
        }

        // Proceed with the update
        let credentials = Credentials::load()?;

        let variables = update_paymaster::Variables {
            paymaster_name: current_name.clone(),
            new_name: self.name.clone(),
            team_name: self.team.clone(),
            active: self.active,
        };
        let request_body = UpdatePaymaster::build_query(variables);

        let client = Client::new_with_token(credentials.access_token);

        let _data: update_paymaster::ResponseData = client.query(&request_body).await?;

        // Display success message
        println!("\n✅ Paymaster Updated Successfully");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        println!(
            "🏢 Updated paymaster: {}",
            self.name.as_ref().unwrap_or(&current_name)
        );

        println!("\n🔧 Applied changes:");
        if let Some(ref new_name) = self.name {
            println!("  • Name updated to: {}", new_name);
        }

        if let Some(ref new_team) = self.team {
            println!("  • Team updated to: {}", new_team);
        }

        if let Some(active_state) = self.active {
            println!(
                "  • Active state updated to: {}",
                if active_state {
                    "✅ Active"
                } else {
                    "❌ Inactive"
                }
            );
        }

        Ok(())
    }
}
