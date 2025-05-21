use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use serde::Deserialize;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::paymaster::add_policies::PolicyInput;
use slot::graphql::paymaster::{add_policies, remove_all_policies, remove_policies};
use slot::graphql::paymaster::{AddPolicies, RemoveAllPolicies, RemovePolicies};
use slot::graphql::GraphQLQuery;
use std::fs;
use std::io::{self, Write};
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
    #[command(about = "Remove all policies from a paymaster.")]
    RemoveAll(RemoveAllArgs),
}

#[derive(Debug, Args)]
struct AddPolicyArgs {
    #[arg(long, help = "Name of the paymaster to add policies to.")]
    name: String,
    #[arg(
        long,
        help = "Path to a JSON file containing an array of policies to add. Each policy should have 'contractAddress', 'entryPoint', and 'selector'."
    )]
    policies_file: PathBuf,
}

#[derive(Debug, Args)]
struct RemovePolicyArgs {
    #[arg(long, help = "Name the paymaster to remove policies from.")]
    name: String,
    #[arg(
        long,
        required = true,
        value_delimiter = ',',
        help = "Comma-separated list of policy IDs to remove."
    )]
    policy_ids: Vec<String>,
}

#[derive(Debug, Args)]
struct RemoveAllArgs {
    #[arg(long, help = "Name of the paymaster to remove all policies from.")]
    name: String,
}

// Temporary struct for deserializing policy JSON from file
#[derive(Deserialize, Debug)]
struct PolicyInputJson {
    #[serde(rename = "contractAddress")]
    contract_address: String,
    #[serde(rename = "entryPoint")]
    entry_point: String,
}

impl PolicyCmd {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            PolicySubcommand::Add(args) => Self::run_add(args).await,
            PolicySubcommand::Remove(args) => Self::run_remove(args).await,
            PolicySubcommand::RemoveAll(args) => Self::run_remove_all(args).await,
        }
    }

    async fn run_add(args: &AddPolicyArgs) -> Result<()> {
        println!(
            "Adding policies to paymaster: {} from file: {:?}...",
            args.name, args.policies_file
        );

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
        let policies_gql: Vec<PolicyInput> = policies_json
            .into_iter()
            .map(|p| PolicyInput {
                contract_address: p.contract_address,
                entry_point: p.entry_point,
            })
            .collect();

        if policies_gql.is_empty() {
            println!("Warning: No policies found in the provided file.");
            return Ok(());
        }

        let credentials = Credentials::load()?;

        let variables = add_policies::Variables {
            paymaster_name: args.name.clone(),
            policies: policies_gql,
        };
        let request_body = AddPolicies::build_query(variables);
        let client = Client::new_with_token(credentials.access_token);
        let data: add_policies::ResponseData = client.query(&request_body).await?;
        let added_policies = data.add_policies.unwrap_or_default();
        println!("Successfully added {} policies:", added_policies.len());

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["ID", "Contract Address", "Entry Point"]);

        for policy_item in added_policies {
            table.add_row(vec![
                Cell::new(&policy_item.id),
                Cell::new(&policy_item.contract_address),
                Cell::new(&policy_item.entry_point),
            ]);
        }

        println!("{}", table);

        Ok(())
    }

    async fn run_remove(args: &RemovePolicyArgs) -> Result<()> {
        println!(
            "Removing policies {:?} from paymaster: {}...",
            args.policy_ids, args.name
        );

        if args.policy_ids.is_empty() {
            println!("Warning: No policy IDs provided for removal.");
            return Ok(());
        }

        // 1. Load Credentials
        let credentials = Credentials::load()?;

        // 2. Build Query Variables
        let variables = remove_policies::Variables {
            paymaster_name: args.name.clone(),
            policy_ids: args.policy_ids.clone(),
        };
        let request_body = RemovePolicies::build_query(variables);

        // 3. Create Client
        let client = Client::new_with_token(credentials.access_token);

        // 4. Execute Query
        let data: remove_policies::ResponseData = client.query(&request_body).await?;

        // 5. Print Result
        if data.remove_policies {
            println!("Successfully removed policies: {:?}", args.policy_ids);
        } else {
            // The boolean response doesn't give much detail, maybe log a warning or error
            println!(
                "Failed to remove policies or some/all IDs were not found for paymaster {}.",
                args.name
            );
            // Consider returning an error or specific exit code?
        }

        Ok(())
    }

    async fn run_remove_all(args: &RemoveAllArgs) -> Result<()> {
        // Ask for confirmation
        print!("Remove all policies from paymaster {}? [y/N]: ", args.name);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Operation cancelled.");
            return Ok(());
        }

        println!("Removing all policies from paymaster {}...", args.name);

        // 1. Load Credentials
        let credentials = Credentials::load()?;

        // 2. Build Query Variables
        let variables = remove_all_policies::Variables {
            paymaster_name: args.name.clone(),
        };
        let request_body = RemoveAllPolicies::build_query(variables);

        // 3. Create Client
        let client = Client::new_with_token(credentials.access_token);

        // 4. Execute Query
        let data: remove_all_policies::ResponseData = client.query(&request_body).await?;

        // 5. Print Result
        if data.remove_all_policies {
            println!(
                "Successfully removed all policies from paymaster {}",
                args.name
            );
        } else {
            println!("Failed to remove all policies from paymaster {}", args.name);
        }

        Ok(())
    }
}
