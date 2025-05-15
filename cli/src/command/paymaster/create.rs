use anyhow::{Ok, Result};
use clap::Args;
use num_bigint::BigInt;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::paymaster::{add_paymaster_policies, create_paymaster};
use slot::graphql::paymaster::{AddPaymasterPolicies, CreatePaymaster};
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
    #[arg(
        long,
        help = "Preset name to use for configuring the paymaster (e.g., 'dopewars')."
    )]
    preset: Option<String>,
    #[arg(long, help = "Chain ID to use for the preset policies.")]
    chain_id: Option<String>,
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

        if let (Some(preset), Some(chain_id)) = (&self.preset, &self.chain_id) {
            println!("Loading preset '{}' for chain '{}'...", preset, chain_id);

            let config = slot::presets::load_preset(preset)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load preset: {}", e))?;

            let policies = slot::presets::extract_paymaster_policies(&config, chain_id);

            if policies.is_empty() {
                println!("No paymaster policies found in preset");
                return Ok(());
            }

            println!(
                "Applying {} paymaster policies from preset...",
                policies.len()
            );

            let policies_gql: Vec<add_paymaster_policies::PaymasterPolicyInput> = policies
                .into_iter()
                .map(|p| add_paymaster_policies::PaymasterPolicyInput {
                    contract_address: p.contract_address,
                    entry_point: p.entry_point,
                    selector: p.selector,
                })
                .collect();

            let variables = add_paymaster_policies::Variables {
                paymaster_id: data.create_paymaster.id.clone(),
                policies: policies_gql,
            };

            let request_body = AddPaymasterPolicies::build_query(variables);
            let result: add_paymaster_policies::ResponseData = client.query(&request_body).await?;

            println!(
                "Successfully added {} policies from preset",
                result
                    .add_paymaster_policies
                    .as_ref()
                    .map_or(0, |v| v.len())
            );
        }

        Ok(())
    }
}
