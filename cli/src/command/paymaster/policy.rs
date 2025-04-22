use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use serde::Deserialize;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::paymaster::add_paymaster_policies::PaymasterPolicyInput;
use slot::graphql::paymaster::{add_paymaster_policies, remove_paymaster_policies};
use slot::graphql::paymaster::{AddPaymasterPolicies, RemovePaymasterPolicies};
use slot::graphql::GraphQLQuery;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Args)]
#[command(next_help_heading = "Paymaster policy options")]
pub struct PolicyCmd {
    #[command(subcommand)]
    command: PolicySubcommand,
}

#[derive(Subcommand, Debug)]
enum PolicySubcommand {
    #[command(about = "Add policies to a paymaster from a JSON file.")]
    Add(AddPolicyArgs),
    #[command(about = "Remove policies from a paymaster by ID.")]
    Remove(RemovePolicyArgs),
}

#[derive(Debug, Args)]
struct AddPolicyArgs {
    #[arg(long, help = "ID of the paymaster to add policies to.")]
    paymaster_id: String,
    #[arg(
        long,
        help = "Path to a JSON file containing an array of policies to add. Each policy should have 'contractAddress', 'entryPoint', and 'selector'."
    )]
    policies_file: PathBuf,
}

#[derive(Debug, Args)]
struct RemovePolicyArgs {
    #[arg(long, help = "ID of the paymaster to remove policies from.")]
    paymaster_id: String,
    #[arg(
        long,
        required = true,
        value_delimiter = ',',
        help = "Comma-separated list of policy IDs to remove."
    )]
    policy_ids: Vec<String>,
}

// Temporary struct for deserializing policy JSON from file
#[derive(Deserialize, Debug)]
struct PolicyInputJson {
    #[serde(rename = "contractAddress")]
    contract_address: String,
    #[serde(rename = "entryPoint")]
    entry_point: String,
    selector: String,
}

impl PolicyCmd {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            PolicySubcommand::Add(args) => Self::run_add(args).await,
            PolicySubcommand::Remove(args) => Self::run_remove(args).await,
        }
    }

    async fn run_add(args: &AddPolicyArgs) -> Result<()> {
        println!(
            "Adding policies to paymaster ID: {} from file: {:?}...",
            args.paymaster_id, args.policies_file
        );

        // 1. Read and Parse File
        let file_content = fs::read_to_string(&args.policies_file).context(format!(
            "Failed to read policies file: {:?}",
            args.policies_file
        ))?;
        let policies_json: Vec<PolicyInputJson> =
            serde_json::from_str(&file_content).context(format!(
                "Failed to parse policies JSON from file: {:?}",
                args.policies_file
            ))?;

        // Map JSON input to GraphQL input type
        let policies_gql: Vec<PaymasterPolicyInput> = policies_json
            .into_iter()
            .map(|p| PaymasterPolicyInput {
                contract_address: p.contract_address,
                entry_point: p.entry_point,
                selector: p.selector,
            })
            .collect();

        if policies_gql.is_empty() {
            println!("Warning: No policies found in the provided file.");
            return Ok(());
        }

        // 2. Load Credentials
        let credentials = Credentials::load()?;

        // 3. Build Query Variables
        let variables = add_paymaster_policies::Variables {
            paymaster_id: args.paymaster_id.clone(),
            policies: policies_gql,
        };
        let request_body = AddPaymasterPolicies::build_query(variables);

        // 4. Create Client
        let client = Client::new_with_token(credentials.access_token);

        // 5. Execute Query
        let data: add_paymaster_policies::ResponseData = client.query(&request_body).await?;

        // 6. Print Result
        let added_policies = data.add_paymaster_policies.unwrap_or_default();

        println!("Successfully added {} policies:", added_policies.len());

        for policy_item in added_policies {
            println!(
                "  - ID: {}, Contract: {}, EntryPoint: {}, Selector: {}",
                policy_item.id,
                policy_item.contract_address,
                policy_item.entry_point,
                policy_item.selector
            );
        }

        Ok(())
    }

    async fn run_remove(args: &RemovePolicyArgs) -> Result<()> {
        println!(
            "Removing policies {:?} from paymaster ID: {}...",
            args.policy_ids, args.paymaster_id
        );

        if args.policy_ids.is_empty() {
            println!("Warning: No policy IDs provided for removal.");
            return Ok(());
        }

        // 1. Load Credentials
        let credentials = Credentials::load()?;

        // 2. Build Query Variables
        let variables = remove_paymaster_policies::Variables {
            paymaster_id: args.paymaster_id.clone(),
            policy_ids: args.policy_ids.clone(),
        };
        let request_body = RemovePaymasterPolicies::build_query(variables);

        // 3. Create Client
        let client = Client::new_with_token(credentials.access_token);

        // 4. Execute Query
        let data: remove_paymaster_policies::ResponseData = client.query(&request_body).await?;

        // 5. Print Result
        if data.remove_paymaster_policies {
            println!("Successfully removed policies: {:?}", args.policy_ids);
        } else {
            // The boolean response doesn't give much detail, maybe log a warning or error
            println!(
                "Failed to remove policies or some/all IDs were not found for paymaster {}.",
                args.paymaster_id
            );
            // Consider returning an error or specific exit code?
        }

        Ok(())
    }
}
