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
    #[arg(long, help = "Unit for the budget (CREDIT or STRK).")]
    unit: String,
}

impl CreateArgs {
    pub async fn run(&self, name: String) -> Result<()> {
        let credentials = Credentials::load()?;

        let unit = match self.unit.to_uppercase().as_str() {
            "CREDIT" => FeeUnit::CREDIT,
            "STRK" => FeeUnit::STRK,
            _ => return Err(anyhow::anyhow!("Invalid unit: {}", self.unit)),
        };

        let variables = create_paymaster::Variables {
            name: name.clone(),
            team_name: self.team.clone(),
            budget: self.budget as i64,
            unit,
        };
        let request_body = CreatePaymaster::build_query(variables);

        let client = Client::new_with_token(credentials.access_token);

        let data: create_paymaster::ResponseData = client.query(&request_body).await?;

        println!(
            "Paymaster '{}' created with budget of {} {}",
            data.create_paymaster.name,
            self.budget,
            self.unit.to_uppercase()
        );

        Ok(())
    }
}
