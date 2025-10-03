use super::utils;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use clap::Args;
use slot_core::credentials::Credentials;
use slot_graphql::api::Client;
use slot_graphql::paymaster::list_policies;
use slot_graphql::paymaster::paymaster_info;
use slot_graphql::paymaster::ListPolicies;
use slot_graphql::paymaster::PaymasterInfo;
use slot_graphql::GraphQLQuery;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

// Embed SQL template at compile time
const SQL_TEMPLATE: &str = include_str!("dune.sql");

const EXCLUDED_POLICIES: &[&str] = &[
    "0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d", // STRK Token
    "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7", // ETH Token
    "0x124aeb495b947201f5fac96fd1138e326ad86195b98df6dec9009158a533b49", // LORDS Token
];

#[derive(Debug, Args)]
pub struct DuneArgs {
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

// Helper function to normalize address for comparison
fn normalize_address(addr: &str) -> String {
    let trimmed = addr.trim_start_matches("0x");
    format!("0x{:0>64}", trimmed.to_lowercase())
}

// Helper function to format policy addresses, excluding base policies
fn format_policy_addresses(
    policies: &[&list_policies::ListPoliciesPaymasterPoliciesEdgesNode],
) -> Vec<String> {
    // Create a set of normalized base policies to exclude
    let excluded_policies_set: HashSet<String> = EXCLUDED_POLICIES
        .iter()
        .map(|addr| normalize_address(addr))
        .collect();

    let mut addresses: Vec<String> = policies
        .iter()
        .filter_map(|p| {
            let normalized_addr = normalize_address(&p.contract_address);

            // Skip if this address is in the base policies to exclude
            if excluded_policies_set.contains(&normalized_addr) {
                None
            } else {
                Some(format!("    {}", normalized_addr))
            }
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    addresses.sort();
    addresses
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

                // Separate active and inactive policies
                let active_policies: Vec<_> = policies_list.iter().filter(|p| p.active).collect();
                let inactive_policies: Vec<_> =
                    policies_list.iter().filter(|p| !p.active).collect();

                // Format active contract addresses
                let active_addresses = format_policy_addresses(&active_policies);

                // Format inactive contract addresses
                let inactive_addresses = format_policy_addresses(&inactive_policies);

                if active_addresses.is_empty() && inactive_addresses.is_empty() {
                    println!("No policies found for paymaster '{}'.", name);
                    return Ok(());
                }

                // Use embedded template instead of reading from file system
                let template = SQL_TEMPLATE;

                // Create formatted address list with comments
                let mut formatted_addresses = Vec::new();

                if !active_addresses.is_empty() {
                    formatted_addresses.push("-- Active policies".to_string());
                    formatted_addresses
                        .extend(active_addresses.iter().map(|addr| format!("{},", addr)));

                    // Remove comma from last active address if there are no inactive addresses
                    if inactive_addresses.is_empty() {
                        if let Some(last) = formatted_addresses.last_mut() {
                            *last = last.trim_end_matches(',').to_string();
                        }
                    }
                }

                if !inactive_addresses.is_empty() {
                    formatted_addresses.push("    -- Inactive policies (soft deleted)".to_string());
                    formatted_addresses.extend(inactive_addresses.iter().enumerate().map(
                        |(i, addr)| {
                            if i == inactive_addresses.len() - 1 {
                                addr.clone() // No comma for last item
                            } else {
                                format!("{},", addr)
                            }
                        },
                    ));
                }

                // Replace placeholders in template
                let mut sql_query = template
                    .replace("{contract_addresses}", &formatted_addresses.join("\n"))
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
