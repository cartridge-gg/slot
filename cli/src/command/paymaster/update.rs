use anyhow::Result;
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::paymaster::{add_paymaster_policies, get_paymaster, remove_paymaster_policies};
use slot::graphql::paymaster::{AddPaymasterPolicies, GetPaymaster, RemovePaymasterPolicies};
use slot::graphql::GraphQLQuery;

#[derive(Debug, Args)]
#[command(next_help_heading = "Update paymaster options")]
pub struct UpdateArgs {
    #[arg(long, help = "ID of the paymaster to update.")]
    paymaster_id: String,

    #[arg(
        long,
        help = "Preset name to use for configuring the paymaster (e.g., 'dopewars')."
    )]
    preset: String,

    #[arg(long, help = "Chain ID to use for the preset policies.")]
    chain_id: String,

    #[arg(
        long,
        help = "Remove existing policies before applying preset policies.",
        default_value = "false"
    )]
    replace: bool,
}

impl UpdateArgs {
    pub async fn run(&self) -> Result<()> {
        // 1. Load Credentials
        let credentials = Credentials::load()?;

        // 2. Create Client
        let client = Client::new_with_token(credentials.access_token);

        println!(
            "Updating paymaster '{}' with preset '{}'...",
            self.paymaster_id, self.preset
        );

        // 3. Load preset configuration
        let config = slot::presets::load_preset(&self.preset)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to load preset: {}", e))?;

        // 4. Extract paymaster policies
        let policies = slot::presets::extract_paymaster_policies(&config, &self.chain_id);

        if policies.is_empty() {
            println!("No paymaster policies found in preset");
            return Ok(());
        }

        // 5. Remove existing policies if replace is true
        if self.replace {
            println!("Removing existing policies...");

            // First get the paymaster to retrieve existing policy IDs
            let variables = get_paymaster::Variables {
                id: self.paymaster_id.clone(),
            };

            let request_body = GetPaymaster::build_query(variables);
            let data: get_paymaster::ResponseData = client.query(&request_body).await?;

            // Extract policy IDs
            let mut policy_ids = Vec::new();

            if let Some(paymaster) = data.paymaster {
                // Use if let instead of for loop for Option
                if let Some(edges) = paymaster.policies.edges {
                    // Use flatten to simplify the nested Option handling
                    for edge in edges.into_iter().flatten() {
                        if let Some(node) = edge.node {
                            policy_ids.push(node.id);
                        }
                    }
                }
            }

            if !policy_ids.is_empty() {
                // Remove existing policies
                let variables = remove_paymaster_policies::Variables {
                    paymaster_id: self.paymaster_id.clone(),
                    policy_ids,
                };

                let request_body = RemovePaymasterPolicies::build_query(variables);
                let _: remove_paymaster_policies::ResponseData =
                    client.query(&request_body).await?;

                println!("Existing policies removed");
            }
        }

        // 6. Add new policies from preset
        println!(
            "Applying {} paymaster policies from preset...",
            policies.len()
        );

        // Convert to GraphQL input type
        let policies_gql: Vec<add_paymaster_policies::PaymasterPolicyInput> = policies
            .into_iter()
            .map(|p| add_paymaster_policies::PaymasterPolicyInput {
                contract_address: p.contract_address,
                entry_point: p.entry_point,
                selector: p.selector,
            })
            .collect();

        // Add policies to paymaster
        let variables = add_paymaster_policies::Variables {
            paymaster_id: self.paymaster_id.clone(),
            policies: policies_gql,
        };

        let request_body = AddPaymasterPolicies::build_query(variables);
        let result: add_paymaster_policies::ResponseData = client.query(&request_body).await?;

        println!(
            "Successfully updated paymaster with {} policies from preset",
            result
                .add_paymaster_policies
                .as_ref()
                .map_or(0, |v| v.len())
        );

        Ok(())
    }
}
