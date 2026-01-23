use anyhow::Result;
use clap::{Args, Subcommand};
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::paymaster::decrease_budget::AdminBudgetReason as DecreaseBudgetReason;
use slot::graphql::paymaster::decrease_budget::FeeUnit as DecreaseBudgetFeeUnit;
use slot::graphql::paymaster::increase_budget::AdminBudgetReason as IncreaseBudgetReason;
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
    #[arg(long, help = "Amount to increase the budget.")]
    amount: u64,
    #[arg(long, help = "Unit for the budget (USD or STRK).")]
    unit: String,
    #[arg(long, help = "Admin performing the operation.")]
    admin: bool,
    #[arg(
        long,
        help = "Reason for the budget increase (ADVANCE, SETTLEMENT, REFUND, PROMOTION, CORRECTION)."
    )]
    reason: Option<String>,
}

#[derive(Debug, Args)]
struct DecreaseBudgetArgs {
    #[arg(long, help = "Amount to decrease the budget.")]
    amount: u64,
    #[arg(long, help = "Unit for the budget (USD or STRK).")]
    unit: String,
    #[arg(long, help = "Admin performing the operation.")]
    admin: bool,
    #[arg(
        long,
        help = "Reason for the budget decrease (ADVANCE, SETTLEMENT, REFUND, PROMOTION, CORRECTION)."
    )]
    reason: Option<String>,
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

        let (unit, amount_for_api) = match args.unit.to_uppercase().as_str() {
            "USD" => (IncreaseBudgetFeeUnit::CREDIT, (args.amount * 100) as i64), // Convert USD to credits
            "STRK" => (IncreaseBudgetFeeUnit::STRK, args.amount as i64),
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid unit: {}. Supported units: USD, STRK",
                    args.unit
                ))
            }
        };

        let reason = match args.reason.as_deref() {
            Some(r) => match r.to_uppercase().as_str() {
                "ADVANCE" => Some(IncreaseBudgetReason::ADVANCE),
                "SETTLEMENT" => Some(IncreaseBudgetReason::SETTLEMENT),
                "REFUND" => Some(IncreaseBudgetReason::REFUND),
                "PROMOTION" => Some(IncreaseBudgetReason::PROMOTION),
                "CORRECTION" => Some(IncreaseBudgetReason::CORRECTION),
                _ => {
                    return Err(anyhow::anyhow!(
                        "Invalid reason: {}. Supported reasons: ADVANCE, SETTLEMENT, REFUND, PROMOTION, CORRECTION",
                        r
                    ))
                }
            },
            None => None,
        };

        let variables = increase_budget::Variables {
            paymaster_name: name.clone(),
            amount: amount_for_api,
            unit,
            admin: Some(args.admin),
            reason,
        };
        let request_body = IncreaseBudget::build_query(variables);

        let client = Client::new_with_token(credentials.access_token);

        let data: increase_budget::ResponseData = client.query(&request_body).await?;

        let new_budget_formatted = data.increase_budget.budget as f64 / 1e6;

        // Calculate display values based on original unit
        let display_budget = match args.unit.to_uppercase().as_str() {
            "USD" => format!("${:.2} USD", new_budget_formatted * 0.01), // Convert credits back to USD for display
            "STRK" => format!("{} STRK", new_budget_formatted as i64),
            _ => format!(
                "{} {}",
                new_budget_formatted as i64,
                args.unit.to_uppercase()
            ),
        };

        println!("\n✅ Budget Increased Successfully");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        println!("🏢 Paymaster: {}", data.increase_budget.name);

        println!("\n📈 Operation:");
        println!("  • Action: Increased");
        println!("  • Amount: {} {}", args.amount, args.unit.to_uppercase());

        println!("\n💰 New Budget:");
        println!("  • Amount: {}", display_budget);

        Ok(())
    }

    async fn run_decrease(args: &DecreaseBudgetArgs, name: String) -> Result<()> {
        // 1. Load Credentials
        let credentials = Credentials::load()?;

        let (unit, amount_for_api) = match args.unit.to_uppercase().as_str() {
            "USD" => (DecreaseBudgetFeeUnit::CREDIT, (args.amount * 100) as i64), // Convert USD to credits
            "STRK" => (DecreaseBudgetFeeUnit::STRK, args.amount as i64),
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid unit: {}. Supported units: USD, STRK",
                    args.unit
                ))
            }
        };

        // 2. Build Query Variables
        let reason = match args.reason.as_deref() {
            Some(r) => match r.to_uppercase().as_str() {

                "ADVANCE" => Some(DecreaseBudgetReason::ADVANCE),
                "SETTLEMENT" => Some(DecreaseBudgetReason::SETTLEMENT),
                "REFUND" => Some(DecreaseBudgetReason::REFUND),
                "PROMOTION" => Some(DecreaseBudgetReason::PROMOTION),
                "CORRECTION" => Some(DecreaseBudgetReason::CORRECTION),
                _ => {
                    return Err(anyhow::anyhow!(
                        "Invalid reason: {}. Supported reasons: ADVANCE, SETTLEMENT, REFUND, PROMOTION, CORRECTION",
                        r
                    ))
                }
            },
            None => None,
        };

        let variables = decrease_budget::Variables {
            paymaster_name: name.clone(),
            amount: amount_for_api,
            unit,
            admin: Some(args.admin),
            reason,
        };
        let request_body = DecreaseBudget::build_query(variables);

        // 3. Create Client
        let client = Client::new_with_token(credentials.access_token);

        let data: decrease_budget::ResponseData = client.query(&request_body).await?;

        let new_budget_formatted = data.decrease_budget.budget as f64 / 1e6;

        // Calculate display values based on original unit
        let display_budget = match args.unit.to_uppercase().as_str() {
            "USD" => format!("${:.2} USD", new_budget_formatted * 0.01), // Convert credits back to USD for display
            "STRK" => format!("{} STRK", new_budget_formatted as i64),
            _ => format!(
                "{} {}",
                new_budget_formatted as i64,
                args.unit.to_uppercase()
            ),
        };

        println!("\n✅ Budget Decreased Successfully");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        println!("🏢 Paymaster: {}", data.decrease_budget.name);

        println!("\n📉 Operation:");
        println!("  • Action: Decreased");
        println!("  • Amount: {} {}", args.amount, args.unit.to_uppercase());

        println!("\n💰 New Budget:");
        println!("  • Amount: {}", display_budget);

        Ok(())
    }
}
