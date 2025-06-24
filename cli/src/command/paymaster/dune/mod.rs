use super::utils;
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::paymaster::list_policies;
use slot::graphql::paymaster::paymaster_info;
use slot::graphql::paymaster::ListPolicies;
use slot::graphql::paymaster::PaymasterInfo;
use slot::graphql::GraphQLQuery;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Args)]
pub struct DuneArgs {
    #[arg(long, help = "Use fast query (default)", default_value = "true")]
    fast: bool,

    #[arg(long, help = "Use exact query (slower but more precise)")]
    exact: bool,

    #[arg(
        long,
        help = "Time period to look back (e.g., 1hr, 2min, 24hr, 1day, 1week). If not specified, uses paymaster creation time."
    )]
    last: Option<String>,

    #[arg(
        long,
        help = "Use Dune template parameters ({{start_time}}/{{end_time}}) instead of actual timestamps",
        default_value = "false"
    )]
    dune_params: bool,
}

impl DuneArgs {
    pub async fn run(&self, name: String) -> Result<()> {
        // 1. Load Credentials
        let credentials = Credentials::load()?;

        // Create client once
        let client = Client::new_with_token(credentials.access_token);

        // 2. Get timestamp - skip if using dune params
        let start_time = if self.dune_params {
            "{{start_time}}".to_string()
        } else {
            let created_at = if let Some(last) = &self.last {
                // Calculate time from --last parameter
                let duration = utils::parse_duration(last)?;
                let now = SystemTime::now();
                let since_time = now - duration;
                let since_timestamp = since_time
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| anyhow!("Invalid time calculation"))?
                    .as_secs();

                DateTime::<Utc>::from_timestamp(since_timestamp as i64, 0)
                    .ok_or_else(|| anyhow!("Invalid timestamp"))?
                    .to_rfc3339()
            } else {
                // Get creation time from paymaster
                let info_variables = paymaster_info::Variables { name: name.clone() };
                let info_request = PaymasterInfo::build_query(info_variables);
                let info_data: paymaster_info::ResponseData = client.query(&info_request).await?;

                match info_data.paymaster {
                    Some(pm) => pm.created_at,
                    None => {
                        println!("Paymaster '{}' not found.", name);
                        return Ok(());
                    }
                }
            };

            // Format the timestamp for Dune
            created_at.replace("T", " ").replace("Z", "")
        };

        // 3. Get policies
        let variables = list_policies::Variables { name: name.clone() };
        let request_body = ListPolicies::build_query(variables);
        let data: list_policies::ResponseData = client.query(&request_body).await?;

        // 4. Generate SQL query
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
                    .map(|p| {
                        let addr = p.contract_address.trim_start_matches("0x");
                        format!("    0x{:0>64}", addr) // Pad with zeros to 64 chars
                    })
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect();

                // Load the appropriate template
                let template_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("src/command/paymaster/dune")
                    .join(if self.exact { "exact.sql" } else { "fast.sql" });

                let template = fs::read_to_string(template_path)
                    .context("Failed to read SQL template file")?;

                // Replace placeholders in template
                let mut sql_query = template
                    .replace("{contract_addresses}", &contract_addresses.join(",\n"))
                    .replace("{start_time}", &start_time);

                // Only add end_time constraint if using dune params
                if self.dune_params {
                    sql_query = sql_query.replace(
                        "{end_time_constraint}",
                        "AND t.block_time <= TIMESTAMP '{{end_time}}'",
                    );
                } else {
                    sql_query = sql_query.replace("{end_time_constraint}", "");
                }

                println!("{}", sql_query);
            }
            None => {
                println!("Paymaster '{}' not found.", name);
            }
        }

        Ok(())
    }
}
