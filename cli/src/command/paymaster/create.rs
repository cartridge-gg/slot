use anyhow::{Ok, Result};
use clap::Args;
use num_bigint::BigInt;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::paymaster::create_paymaster;
use slot::graphql::paymaster::CreatePaymaster;
use slot::graphql::GraphQLQuery;

#[derive(Debug, Args)]
#[command(next_help_heading = "Create paymaster options")]
pub struct CreateArgs {
    #[arg(long, help = "Name for the new paymaster.")]
    name: String,
    #[arg(long, help = "Team name to associate the paymaster with.")]
    team: String,
    #[arg(long, help = "Initial budget for the paymaster (in wei).")]
    budget: BigInt,
}

impl CreateArgs {
    pub async fn run(&self) -> Result<()> {
        // 1. Load Credentials
        let credentials = Credentials::load()?;

        // 2. Build Query Variables
        let variables = create_paymaster::Variables {
            name: self.name.clone(),
            team_name: self.team.clone(),
            budget: self.budget.clone(),
        };
        let request_body = CreatePaymaster::build_query(variables);

        // 3. Create Client
        let client = Client::new_with_token(credentials.access_token);

        // 4. Execute Query
        println!(
            "Creating paymaster '{}' for team '{}'...",
            self.name, self.team
        );
        let data: create_paymaster::ResponseData = client.query(&request_body).await?;

        // 5. Print Result
        // Note: name is Option<String>, budget field removed based on .graphql file
        println!(
            "Paymaster '{}' created successfully with ID: {}",
            data.create_paymaster.name.unwrap_or_default(), // Handle Option
            data.create_paymaster.id,
        );

        Ok(())
    }
}
