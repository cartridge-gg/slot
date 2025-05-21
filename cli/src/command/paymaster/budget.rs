use anyhow::Result;
use clap::{Args, Subcommand};
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::paymaster::decrease_budget::FeeUnit as DecreaseBudgetFeeUnit;
use slot::graphql::paymaster::increase_budget::FeeUnit as IncreaseBudgetFeeUnit;
use slot::graphql::paymaster::{decrease_budget, increase_budget};
use slot::graphql::paymaster::{DecreaseBudget, IncreaseBudget};
use slot::graphql::GraphQLQuery;

#[derive(Debug, Args)]
#[command(next_help_heading = "Paymaster budget options")]
pub struct BudgetCmd {
    #[command(subcommand)]
    command: BudgetSubcommand,
}

#[derive(Subcommand, Debug)]
enum BudgetSubcommand {
    #[command(about = "Increase the budget of a paymaster.")]
    Increase(IncreaseBudgetArgs),
    #[command(about = "Decrease the budget of a paymaster.")]
    Decrease(DecreaseBudgetArgs),
}

#[derive(Debug, Args)]
struct IncreaseBudgetArgs {
    #[arg(long, help = "Name of the paymaster.")]
    name: String,
    #[arg(long, help = "Amount to decrease the budget.")]
    amount: u64,
    #[arg(long, help = "Unit for the budget (CREDIT or STRK).")]
    unit: String,
}

#[derive(Debug, Args)]
struct DecreaseBudgetArgs {
    #[arg(long, help = "Name of the paymaster.")]
    name: String,
    #[arg(long, help = "Amount to decrease the budget.")]
    amount: u64,
    #[arg(long, help = "Unit for the budget (CREDIT or STRK).")]
    unit: String,
}

impl BudgetCmd {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            BudgetSubcommand::Increase(args) => Self::run_increase(args).await,
            BudgetSubcommand::Decrease(args) => Self::run_decrease(args).await,
        }
    }

    async fn run_increase(args: &IncreaseBudgetArgs) -> Result<()> {
        // 1. Load Credentials
        let credentials = Credentials::load()?;

        let unit = match args.unit.to_uppercase().as_str() {
            "CREDIT" => IncreaseBudgetFeeUnit::CREDIT,
            "STRK" => IncreaseBudgetFeeUnit::STRK,
            _ => return Err(anyhow::anyhow!("Invalid unit: {}", args.unit)),
        };

        // 2. Build Query Variables
        let variables = increase_budget::Variables {
            paymaster_name: args.name.clone(),
            amount: args.amount as i64,
            unit,
        };
        let request_body = IncreaseBudget::build_query(variables);

        // 3. Create Client
        let client = Client::new_with_token(credentials.access_token);

        // 4. Execute Query
        println!(
            "Increasing budget for paymaster ID: {} by {} {:?}...",
            args.name, args.amount, args.unit
        );
        let data: increase_budget::ResponseData = client.query(&request_body).await?;

        // 5. Print Result (assuming mutation returns name and id)
        // Check the .graphql file - budget might not be returned
        println!(
            "Budget increased successfully for Paymaster '{}'.",
            data.increase_budget.name
        );

        Ok(())
    }

    async fn run_decrease(args: &DecreaseBudgetArgs) -> Result<()> {
        // 1. Load Credentials
        let credentials = Credentials::load()?;

        let unit = match args.unit.to_uppercase().as_str() {
            "CREDIT" => DecreaseBudgetFeeUnit::CREDIT,
            "STRK" => DecreaseBudgetFeeUnit::STRK,
            _ => return Err(anyhow::anyhow!("Invalid unit: {}", args.unit)),
        };

        // 2. Build Query Variables
        let variables = decrease_budget::Variables {
            paymaster_name: args.name.clone(),
            amount: args.amount as i64,
            unit,
        };
        let request_body = DecreaseBudget::build_query(variables);

        // 3. Create Client
        let client = Client::new_with_token(credentials.access_token);

        // 4. Execute Query
        println!(
            "Decreasing budget for paymaster ID: {} by {} {:?}...",
            args.name, args.amount, args.unit
        );
        let data: decrease_budget::ResponseData = client.query(&request_body).await?;

        // 5. Print Result (assuming mutation returns name and id)
        // Check the .graphql file - budget might not be returned
        println!(
            "Budget decreased successfully for Paymaster '{}'.",
            data.decrease_budget.name
        );

        Ok(())
    }
}
