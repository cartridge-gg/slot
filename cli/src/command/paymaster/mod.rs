use anyhow::Result;
use clap::{Args, Subcommand};

// Import the structs defined in the subcommand files
use self::budget::BudgetCmd;
use self::create::CreateArgs;
use self::get::GetArgs;
use self::policy::PolicyCmd;

mod budget;
mod create;
mod get;
mod policy;

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

    #[command(about = "Get paymaster details by Name.")]
    Get(GetArgs),

    #[command(about = "Manage paymaster policies.")]
    Policy(PolicyCmd),

    #[command(about = "Manage paymaster budget.")]
    Budget(BudgetCmd),
}

impl PaymasterCmd {
    // Main entry point for the paymaster command group
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            PaymasterSubcommand::Create(args) => args.run(self.name.clone()).await,
            PaymasterSubcommand::Get(args) => args.run(self.name.clone()).await,
            PaymasterSubcommand::Policy(cmd) => cmd.run(self.name.clone()).await,
            PaymasterSubcommand::Budget(cmd) => cmd.run(self.name.clone()).await,
        }
    }
}
