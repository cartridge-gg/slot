use anyhow::Result;
use clap::{Args, Subcommand};

// Import the structs defined in the subcommand files
use self::budget::BudgetCmd;
use self::create::CreateArgs;
use self::get::GetArgs;
use self::list::ListArgs;
use self::policy::PolicyCmd;
use self::update::UpdateArgs;

mod budget;
mod create;
mod get;
mod list;
mod policy;
mod update;

/// Command group for managing Paymasters
#[derive(Debug, Args)]
#[command(next_help_heading = "Paymaster options")]
pub struct PaymasterCmd {
    #[command(subcommand)]
    command: PaymasterSubcommand,
}

// Enum defining the specific paymaster actions
#[derive(Subcommand, Debug)]
enum PaymasterSubcommand {
    #[command(about = "List paymasters for the current user.", aliases = ["ls"])]
    List(ListArgs),

    #[command(about = "Create a new paymaster.")]
    Create(CreateArgs),

    #[command(about = "Get paymaster details by ID.")]
    Get(GetArgs),

    #[command(about = "Manage paymaster policies.")]
    Policy(PolicyCmd),

    #[command(about = "Manage paymaster budget.")]
    Budget(BudgetCmd),

    #[command(about = "Update paymaster configuration from a preset.")]
    Update(UpdateArgs),
}

impl PaymasterCmd {
    // Main entry point for the paymaster command group
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            PaymasterSubcommand::List(args) => args.run().await,
            PaymasterSubcommand::Create(args) => args.run().await,
            PaymasterSubcommand::Get(args) => args.run().await,
            PaymasterSubcommand::Policy(cmd) => cmd.run().await,
            PaymasterSubcommand::Budget(cmd) => cmd.run().await,
            PaymasterSubcommand::Update(args) => args.run().await,
        }
    }
}
