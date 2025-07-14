use anyhow::{Ok, Result};
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::paymaster::create_paymaster;
use slot::graphql::paymaster::create_paymaster::FeeUnit;
use slot::graphql::paymaster::CreatePaymaster;
use slot::graphql::GraphQLQuery;

#[derive(Debug, Args)]
#[command(next_help_heading = "Create paymaster options")]
pub struct CreateArgs {
    #[arg(long, help = "Team name to associate the paymaster with.")]
    team: String,
    #[arg(long, help = "Initial budget for the paymaster.")]
    budget: u64,
    #[arg(long, help = "Unit for the budget (USD or STRK).")]
    unit: String,
}

impl CreateArgs {
    pub async fn run(&self, name: String) -> Result<()> {
        let credentials = Credentials::load()?;

        let (unit, budget_for_api) = match self.unit.to_uppercase().as_str() {
            "USD" => (FeeUnit::CREDIT, (self.budget as f64 * 100.0) as i64), // Convert USD to credits
            "STRK" => (FeeUnit::STRK, self.budget as i64),
            _ => return Err(anyhow::anyhow!("Invalid unit: {}", self.unit)),
        };

        let variables = create_paymaster::Variables {
            name: name.clone(),
            team_name: self.team.clone(),
            budget: budget_for_api,
            unit,
        };
        let request_body = CreatePaymaster::build_query(variables);

        let client = Client::new_with_token(credentials.access_token);

        let data: create_paymaster::ResponseData = client.query(&request_body).await?;

        let budget_formatted = data.create_paymaster.budget as f64 / 1e6;

        // Calculate display values based on original unit
        let display_budget = match self.unit.to_uppercase().as_str() {
            "USD" => format!("${:.2} USD", budget_formatted * 0.01), // Convert credits back to USD for display
            "STRK" => format!("{} STRK", budget_formatted as i64),
            _ => format!("{} {}", budget_formatted as i64, self.unit.to_uppercase()),
        };

        println!("\n✅ Paymaster Created Successfully");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        println!("🏢 Details:");
        println!("  • Name: {}", data.create_paymaster.name);
        println!("  • Team: {}", self.team);

        println!("\n💰 Initial Budget:");
        println!("  • Amount: {}", display_budget);

        Ok(())
    }
}
