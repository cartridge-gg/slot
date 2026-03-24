use anyhow::Result;
use clap::{Args, Subcommand};
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use serde::{Deserialize, Serialize};

// Import the structs defined in the subcommand files
use self::budget::BudgetCmd;
use self::create::CreateArgs;
use self::dune::DuneArgs;
use self::info::InfoArgs;
use self::policy::PolicyCmd;
use self::stats::StatsArgs;
use self::transactions::TransactionArgs;
use self::update::UpdateArgs;
mod budget;
mod create;
mod dune;
mod info;
mod policy;
mod stats;
mod transactions;
mod update;
pub(crate) mod utils;

#[derive(Debug, Args, Serialize, Deserialize)]
pub struct PolicyArgs {
    #[arg(long, help = "Contract address of the policy")]
    #[serde(rename = "contractAddress")]
    contract: String,

    #[arg(long, help = "Entrypoint name")]
    #[serde(rename = "entrypoint")]
    entrypoint: String,

    #[arg(
        long,
        help = "Trigger contract address (policy only applies if this call exists in the multicall)"
    )]
    #[serde(rename = "triggerContract", skip_serializing_if = "Option::is_none")]
    trigger_contract: Option<String>,

    #[arg(long, help = "Trigger entrypoint name")]
    #[serde(rename = "triggerEntrypoint", skip_serializing_if = "Option::is_none")]
    trigger_entrypoint: Option<String>,
}

/// Command group for managing Paymasters
#[derive(Debug, Args)]
#[command(next_help_heading = "Paymaster options")]
pub struct PaymasterCmd {
    #[arg(help = "the name of the paymaster to manage.")]
    name: String,

    #[command(subcommand)]
    command: PaymasterSubcommand,
}

// Enum defining the specific paymaster actions
#[derive(Subcommand, Debug)]
enum PaymasterSubcommand {
    #[command(about = "Create a new paymaster.", alias = "c")]
    Create(CreateArgs),

    #[command(about = "Update paymaster.", alias = "u")]
    Update(UpdateArgs),

    #[command(about = "Manage paymaster policies.", alias = "p")]
    Policy(PolicyCmd),

    #[command(about = "Manage paymaster budget.", alias = "b")]
    Budget(BudgetCmd),

    #[command(about = "Manage paymaster stats.", alias = "s")]
    Stats(StatsArgs),

    #[command(about = "Get paymaster info.", alias = "i")]
    Info(InfoArgs),

    #[command(about = "Get paymaster transactions.", alias = "t")]
    Transactions(TransactionArgs),

    #[command(about = "Generate Dune SQL query for paymaster policies")]
    Dune(DuneArgs),
}

impl PaymasterCmd {
    // Main entry point for the paymaster command group
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            PaymasterSubcommand::Create(args) => args.run(self.name.clone()).await,
            //PaymasterSubcommand::Get(args) => args.run(self.name.clone()).await,
            PaymasterSubcommand::Policy(cmd) => cmd.run(self.name.clone()).await,
            PaymasterSubcommand::Budget(cmd) => cmd.run(self.name.clone()).await,
            PaymasterSubcommand::Stats(cmd) => cmd.run(self.name.clone()).await,
            PaymasterSubcommand::Info(cmd) => cmd.run(self.name.clone()).await,
            PaymasterSubcommand::Update(cmd) => cmd.run(self.name.clone()).await,
            PaymasterSubcommand::Transactions(cmd) => cmd.run(self.name.clone()).await,
            PaymasterSubcommand::Dune(cmd) => cmd.run(self.name.clone()).await,
        }
    }
}

pub fn print_policies_table(policies: &[PolicyArgs]) {
    let has_triggers = policies
        .iter()
        .any(|p| p.trigger_contract.is_some() || p.trigger_entrypoint.is_some());

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    if has_triggers {
        table.set_header(vec![
            "Contract Address",
            "Entry Point",
            "Trigger Contract",
            "Trigger Entry Point",
        ]);
        for policy in policies {
            table.add_row(vec![
                Cell::new(&policy.contract),
                Cell::new(&policy.entrypoint),
                Cell::new(policy.trigger_contract.as_deref().unwrap_or("-")),
                Cell::new(policy.trigger_entrypoint.as_deref().unwrap_or("-")),
            ]);
        }
    } else {
        table.set_header(vec!["Contract Address", "Entry Point"]);
        for policy in policies {
            table.add_row(vec![
                Cell::new(&policy.contract),
                Cell::new(&policy.entrypoint),
            ]);
        }
    }

    println!("{}", table);
}
