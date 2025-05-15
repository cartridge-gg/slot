use anyhow::Result;
use clap::{Args, Subcommand};
use num_bigint::BigInt;
use slot::api::Client;
use slot::credential::Credentials;
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
    #[arg(long, help = "ID of the paymaster.")]
    paymaster_id: String,
    #[arg(long, help = "Amount to increase the budget by (in wei).")]
    amount: BigInt,
}

#[derive(Debug, Args)]
struct DecreaseBudgetArgs {
    #[arg(long, help = "ID of the paymaster.")]
    paymaster_id: String,
    #[arg(long, help = "Amount to decrease the budget by (in wei).")]
    amount: BigInt,
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

        // 2. Build Query Variables
        let variables = increase_budget::Variables {
            paymaster_id: args.paymaster_id.clone(),
            amount: args.amount.clone(), // Pass BigInt directly
        };
        let request_body = IncreaseBudget::build_query(variables);

        // 3. Create Client
        let client = Client::new_with_token(credentials.access_token);

        // 4. Execute Query
        println!(
            "Increasing budget for paymaster ID: {} by {}...",
            args.paymaster_id, args.amount
        );
        let data: increase_budget::ResponseData = client.query(&request_body).await?;

        // 5. Print Result (assuming mutation returns name and id)
        // Check the .graphql file - budget might not be returned
        println!(
            "Budget increased successfully for Paymaster '{}' (ID: {}).",
            data.increase_budget.name.unwrap_or_default(),
            data.increase_budget.id
        );

        Ok(())
    }

    async fn run_decrease(args: &DecreaseBudgetArgs) -> Result<()> {
        // 1. Load Credentials
        let credentials = Credentials::load()?;

        // 2. Build Query Variables
        let variables = decrease_budget::Variables {
            paymaster_id: args.paymaster_id.clone(),
            amount: args.amount.clone(), // Pass BigInt directly
        };
        let request_body = DecreaseBudget::build_query(variables);

        // 3. Create Client
        let client = Client::new_with_token(credentials.access_token);

        // 4. Execute Query
        println!(
            "Decreasing budget for paymaster ID: {} by {}...",
            args.paymaster_id, args.amount
        );
        let data: decrease_budget::ResponseData = client.query(&request_body).await?;

        // 5. Print Result (assuming mutation returns name and id)
        // Check the .graphql file - budget might not be returned
        println!(
            "Budget decreased successfully for Paymaster '{}' (ID: {}).",
            data.decrease_budget.name.unwrap_or_default(),
            data.decrease_budget.id
        );

        Ok(())
    }
}
