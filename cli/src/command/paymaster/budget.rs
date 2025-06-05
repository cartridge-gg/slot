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
    #[arg(long, help = "Amount to decrease the budget.")]
    amount: u64,
    #[arg(long, help = "Unit for the budget (CREDIT or STRK).")]
    unit: String,
}

#[derive(Debug, Args)]
struct DecreaseBudgetArgs {
    #[arg(long, help = "Amount to decrease the budget.")]
    amount: u64,
    #[arg(long, help = "Unit for the budget (CREDIT or STRK).")]
    unit: String,
}

impl BudgetCmd {
    pub async fn run(&self, name: String) -> Result<()> {
        match &self.command {
            BudgetSubcommand::Increase(args) => Self::run_increase(args, name.clone()).await,
            BudgetSubcommand::Decrease(args) => Self::run_decrease(args, name.clone()).await,
        }
    }

    async fn run_increase(args: &IncreaseBudgetArgs, name: String) -> Result<()> {
        let credentials = Credentials::load()?;

        let unit = match args.unit.to_uppercase().as_str() {
            "CREDIT" => IncreaseBudgetFeeUnit::CREDIT,
            "STRK" => IncreaseBudgetFeeUnit::STRK,
            _ => return Err(anyhow::anyhow!("Invalid unit: {}", args.unit)),
        };

        let variables = increase_budget::Variables {
            paymaster_name: name.clone(),
            amount: args.amount as i64,
            unit,
        };
        let request_body = IncreaseBudget::build_query(variables);

        let client = Client::new_with_token(credentials.access_token);

        let data: increase_budget::ResponseData = client.query(&request_body).await?;

        let new_budget_formatted = data.increase_budget.budget as f64 / 1e6;

        // Calculate USD equivalent for CREDIT only
        let usd_equivalent = match args.unit.to_uppercase().as_str() {
            "CREDIT" => new_budget_formatted * 0.01, // 100 credit = 1 USD
            _ => 0.0,
        };

        println!("\n✅ Budget Increased Successfully");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        println!("🏢 Paymaster: {}", data.increase_budget.name);

        println!("\n📈 Operation:");
        println!("  • Action: Increased");
        println!("  • Amount: {} {}", args.amount, args.unit.to_uppercase());

        println!("\n💰 New Budget:");
        if usd_equivalent > 0.0 {
            println!(
                "  • Amount: {} {} (${:.2} USD)",
                new_budget_formatted as i64,
                args.unit.to_uppercase(),
                usd_equivalent
            );
        } else {
            println!(
                "  • Amount: {} {}",
                new_budget_formatted as i64,
                args.unit.to_uppercase()
            );
        }

        Ok(())
    }

    async fn run_decrease(args: &DecreaseBudgetArgs, name: String) -> Result<()> {
        // 1. Load Credentials
        let credentials = Credentials::load()?;

        let unit = match args.unit.to_uppercase().as_str() {
            "CREDIT" => DecreaseBudgetFeeUnit::CREDIT,
            "STRK" => DecreaseBudgetFeeUnit::STRK,
            _ => return Err(anyhow::anyhow!("Invalid unit: {}", args.unit)),
        };

        // 2. Build Query Variables
        let variables = decrease_budget::Variables {
            paymaster_name: name.clone(),
            amount: args.amount as i64,
            unit,
        };
        let request_body = DecreaseBudget::build_query(variables);

        // 3. Create Client
        let client = Client::new_with_token(credentials.access_token);

        let data: decrease_budget::ResponseData = client.query(&request_body).await?;

        let new_budget_formatted = data.decrease_budget.budget as f64 / 1e6;

        // Calculate USD equivalent for CREDIT only
        let usd_equivalent = match args.unit.to_uppercase().as_str() {
            "CREDIT" => new_budget_formatted * 0.01, // 100 credit = 1 USD
            _ => 0.0,
        };

        println!("\n✅ Budget Decreased Successfully");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        println!("🏢 Paymaster: {}", data.decrease_budget.name);

        println!("\n📉 Operation:");
        println!("  • Action: Decreased");
        println!("  • Amount: {} {}", args.amount, args.unit.to_uppercase());

        println!("\n💰 New Budget:");
        if usd_equivalent > 0.0 {
            println!(
                "  • Amount: {} {} (${:.2} USD)",
                new_budget_formatted as i64,
                args.unit.to_uppercase(),
                usd_equivalent
            );
        } else {
            println!(
                "  • Amount: {} {}",
                new_budget_formatted as i64,
                args.unit.to_uppercase()
            );
        }

        Ok(())
    }
}
