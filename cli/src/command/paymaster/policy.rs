use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::paymaster::add_policies::PolicyInput;
use slot::graphql::paymaster::{add_policies, list_policies, remove_all_policies, remove_policies};
use slot::graphql::paymaster::{AddPolicies, ListPolicies, RemoveAllPolicies, RemovePolicies};
use slot::graphql::GraphQLQuery;
use slot::preset::{extract_paymaster_policies, load_preset, PaymasterPolicyInput};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use super::PolicyArgs;

#[derive(Debug, Args)]
#[command(next_help_heading = "Paymaster policy options")]
pub struct PolicyCmd {
    #[command(subcommand)]
    command: PolicySubcommand,
}

#[derive(Subcommand, Debug)]
enum PolicySubcommand {
    #[command(about = "Add policy to a paymaster")]
    Add(PolicyArgs),

    #[command(about = "Add policies to a paymaster from preset")]
    AddFromPreset(AddPresetPolicyArgs),

    #[command(about = "Add policies to a paymaster from a JSON file.")]
    AddFromJson(AddJsonPolicyArgs),

    #[command(about = "Remove policies from a paymaster by ID.")]
    Remove(RemovePolicyArgs),

    #[command(about = "Remove all policies from a paymaster.")]
    RemoveAll(RemoveAllArgs),

    #[command(about = "List policies from a paymaster.")]
    List(ListPolicyArgs),
}

#[derive(Debug, Args)]
struct AddJsonPolicyArgs {
    #[arg(
        long,
        help = "Path to a JSON file containing an array of policies to add. Each policy should have 'contractAddress', 'entryPoint', and 'selector'."
    )]
    file: PathBuf,
}

#[derive(Debug, Args)]
struct AddPresetPolicyArgs {
    #[arg(
        long,
        help = "The name of the preset to add. https://github.com/cartridge-gg/presets/tree/main/configs"
    )]
    name: String,
}

#[derive(Debug, Args)]
struct RemovePolicyArgs {
    #[arg(
        long,
        required = true,
        value_delimiter = ',',
        help = "Comma-separated list of policy IDs to remove."
    )]
    policy_ids: Vec<String>,
}

#[derive(Debug, Args)]
struct RemoveAllArgs {}

#[derive(Debug, Args)]
struct ListPolicyArgs {}

impl PolicyCmd {
    pub async fn run(&self, name: String) -> Result<()> {
        match &self.command {
            PolicySubcommand::Add(args) => Self::run_add(args, name.clone()).await,
            PolicySubcommand::AddFromJson(args) => {
                Self::run_add_from_json(args, name.clone()).await
            }
            PolicySubcommand::AddFromPreset(args) => {
                Self::run_add_from_preset(args, name.clone()).await
            }
            PolicySubcommand::Remove(args) => Self::run_remove(args, name.clone()).await,
            PolicySubcommand::RemoveAll(_) => Self::run_remove_all(name.clone()).await,
            PolicySubcommand::List(_) => Self::run_list(name.clone()).await,
        }
    }

    async fn run_add(args: &PolicyArgs, name: String) -> Result<()> {
        println!("Adding policy to paymaster: {} ", name);

        let credentials = Credentials::load()?;
        let variables = add_policies::Variables {
            paymaster_name: name.clone(),
            policies: vec![PolicyInput {
                contract_address: args.contract.clone(),
                entry_point: args.entrypoint.clone(),
            }],
        };
        let request_body = AddPolicies::build_query(variables);
        let client = Client::new_with_token(credentials.access_token);
        let data: add_policies::ResponseData = client.query(&request_body).await?;
        let added_policies = data.add_policies.unwrap_or_default();
        let policy_args: Vec<PolicyArgs> = added_policies
            .iter()
            .map(|p| PolicyArgs {
                contract: p.contract_address.clone(),
                entrypoint: p.entry_point.clone(),
            })
            .collect();

        super::print_policies_table(&policy_args);

        Ok(())
    }

    async fn run_add_from_json(args: &AddJsonPolicyArgs, name: String) -> Result<()> {
        println!(
            "Adding policies to paymaster: {} from file: {:?}...",
            name, args.file
        );

        let file_content = fs::read_to_string(&args.file)
            .context(format!("Failed to read policies file: {:?}", args.file))?;
        let policies_json: Vec<PaymasterPolicyInput> = serde_json::from_str(&file_content)
            .context(format!(
                "Failed to parse policies JSON from file: {:?}",
                args.file
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
            paymaster_name: name.clone(),
            policies: policies_gql,
        };
        let request_body = AddPolicies::build_query(variables);
        let client = Client::new_with_token(credentials.access_token);
        let data: add_policies::ResponseData = client.query(&request_body).await?;
        let added_policies = data.add_policies.unwrap_or_default();
        let policy_args: Vec<PolicyArgs> = added_policies
            .iter()
            .map(|p| PolicyArgs {
                contract: p.contract_address.clone(),
                entrypoint: p.entry_point.clone(),
            })
            .collect();

        super::print_policies_table(&policy_args);

        Ok(())
    }

    async fn run_add_from_preset(args: &AddPresetPolicyArgs, name: String) -> Result<()> {
        println!(
            "Adding policies to paymaster: {} from preset name: {}",
            name, args.name
        );

        let config = load_preset(&args.name).await?;
        let policies = extract_paymaster_policies(&config, "SN_MAIN");

        let policies_gql: Vec<PolicyInput> = policies
            .into_iter()
            .map(|p| PolicyInput {
                contract_address: p.contract_address,
                entry_point: p.entry_point,
            })
            .collect();

        if policies_gql.is_empty() {
            println!("Warning: No policies found in preset.");
            return Ok(());
        }

        let credentials = Credentials::load()?;

        let variables = add_policies::Variables {
            paymaster_name: name.clone(),
            policies: policies_gql,
        };
        let request_body = AddPolicies::build_query(variables);
        let client = Client::new_with_token(credentials.access_token);
        let data: add_policies::ResponseData = client.query(&request_body).await?;
        let added_policies = data.add_policies.unwrap_or_default();
        let policy_args: Vec<PolicyArgs> = added_policies
            .iter()
            .map(|p| PolicyArgs {
                contract: p.contract_address.clone(),
                entrypoint: p.entry_point.clone(),
            })
            .collect();

        super::print_policies_table(&policy_args);

        Ok(())
    }

    async fn run_remove(args: &RemovePolicyArgs, name: String) -> Result<()> {
        println!(
            "Removing policies {:?} from paymaster: {}...",
            args.policy_ids, name
        );

        if args.policy_ids.is_empty() {
            println!("Warning: No policy IDs provided for removal.");
            return Ok(());
        }

        // 1. Load Credentials
        let credentials = Credentials::load()?;

        // 2. Build Query Variables
        let variables = remove_policies::Variables {
            paymaster_name: name.clone(),
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
                name
            );
            // Consider returning an error or specific exit code?
        }

        Ok(())
    }

    async fn run_remove_all(name: String) -> Result<()> {
        // Ask for confirmation
        print!("Remove all policies from paymaster {}? [y/N]: ", name);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Operation cancelled.");
            return Ok(());
        }

        println!("Removing all policies from paymaster {}...", name);

        // 1. Load Credentials
        let credentials = Credentials::load()?;

        // 2. Build Query Variables
        let variables = remove_all_policies::Variables {
            paymaster_name: name.clone(),
        };
        let request_body = RemoveAllPolicies::build_query(variables);

        // 3. Create Client
        let client = Client::new_with_token(credentials.access_token);

        // 4. Execute Query
        let data: remove_all_policies::ResponseData = client.query(&request_body).await?;

        // 5. Print Result
        if data.remove_all_policies {
            println!("Successfully removed all policies from paymaster {}", name);
        } else {
            println!("Failed to remove all policies from paymaster {}", name);
        }

        Ok(())
    }

    async fn run_list(name: String) -> Result<()> {
        // 1. Load Credentials
        let credentials = Credentials::load()?;

        // 2. Build Query Variables
        let variables = list_policies::Variables { name: name.clone() };
        let request_body = ListPolicies::build_query(variables);

        // 3. Create Client
        let client = Client::new_with_token(credentials.access_token);

        // 4. Execute Query
        println!("Fetching paymaster: {}", name);
        let data: list_policies::ResponseData = client.query(&request_body).await?;

        // 5. Print Result (using Debug format as workaround for Serialize issue)
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

                let policy_args: Vec<PolicyArgs> = policies_list
                    .iter()
                    .map(|p| PolicyArgs {
                        contract: p.contract_address.clone(),
                        entrypoint: p.entry_point.clone(),
                    })
                    .collect();

                super::print_policies_table(&policy_args);
            }
            None => {
                println!("Paymaster '{}' not found.", name);
            }
        }

        Ok(())
    }
}
