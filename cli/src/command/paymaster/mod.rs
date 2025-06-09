use anyhow::Result;
use clap::{Args, Subcommand};
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use serde::{Deserialize, Serialize};

// Import the structs defined in the subcommand files
use self::budget::BudgetCmd;
use self::create::CreateArgs;
use self::info::InfoArgs;
use self::policy::PolicyCmd;
use self::stats::StatsArgs;
use self::transactions::TransactionArgs;
use self::update::UpdateArgs;

mod budget;
mod create;
mod info;
mod policy;
mod stats;
mod transactions;
mod update;
mod utils;

#[derive(Debug, Args, Serialize, Deserialize)]
pub struct PolicyArgs {
    #[arg(long, help = "Contract address of the policy")]
    #[serde(rename = "contractAddress")]
    contract: String,

    #[arg(long, help = "Entrypoint name")]
    #[serde(rename = "entrypoint")]
    entrypoint: String,
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
    #[command(about = "Create a new paymaster.")]
    Create(CreateArgs),

    #[command(about = "Update paymaster.")]
    Update(UpdateArgs),

    #[command(about = "Manage paymaster policies.")]
    Policy(PolicyCmd),

    #[command(about = "Manage paymaster budget.")]
    Budget(BudgetCmd),

    #[command(about = "Manage paymaster stats.")]
    Stats(StatsArgs),

    #[command(about = "Get paymaster info.")]
    Info(InfoArgs),

    #[command(about = "Get paymaster transactions.")]
    Transactions(TransactionArgs),
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
        }
    }
}

pub fn print_policies_table(policies: &[PolicyArgs]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Contract Address", "Entry Point"]);

    for policy in policies {
        table.add_row(vec![
            Cell::new(&policy.contract),
            Cell::new(&policy.entrypoint),
        ]);
    }

    println!("{}", table);
}
