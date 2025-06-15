use anyhow::{Context, Result};
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::paymaster::list_policies;
use slot::graphql::paymaster::ListPolicies;
use slot::graphql::GraphQLQuery;
use std::fs;
use std::path::Path;
use std::collections::HashSet;

#[derive(Debug, Args)]
pub struct DuneArgs {}

impl DuneArgs {
    pub async fn run(&self, name: String) -> Result<()> {
        // 1. Load Credentials
        let credentials = Credentials::load()?;

        // 2. Build Query Variables
        let variables = list_policies::Variables { name: name.clone() };
        let request_body = ListPolicies::build_query(variables);

        // 3. Create Client
        let client = Client::new_with_token(credentials.access_token);

        // 4. Execute Query
        let data: list_policies::ResponseData = client.query(&request_body).await?;

        // 5. Generate SQL query
        match data.paymaster {
            Some(paymaster) => {
                let policies_list: Vec<_> = paymaster
                    .policies
                    .edges
                    .into_iter()
                    .flatten()
                    .filter_map(|edge| edge.unwrap().node)
                    .collect();

                if policies_list.is_empty() {
                    println!("No policies found for paymaster '{}'.", name);
                    return Ok(());
                }

                // Format contract addresses and ensure uniqueness
                let contract_addresses: Vec<String> = policies_list
                    .iter()
                    .map(|p| format!("    0x{}", p.contract_address))
                    .collect::<HashSet<_>>()  // Convert to HashSet to remove duplicates
                    .into_iter()              // Convert back to iterator
                    .collect();               // Collect into Vec

                // Load and process the template
                let template_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("src/command/paymaster/templates/dune_query.sql");
                
                let template = fs::read_to_string(template_path)
                    .context("Failed to read SQL template file")?;

                // Replace the placeholder with actual contract addresses
                let sql_query = template.replace("{contract_addresses}", &contract_addresses.join(",\n"));

                println!("{}", sql_query);
            }
            None => {
                println!("Paymaster '{}' not found.", name);
            }
        }

        Ok(())
    }
} 