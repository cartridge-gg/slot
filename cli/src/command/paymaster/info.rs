use anyhow::Result;
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::paymaster::paymaster_info;
use slot::graphql::paymaster::paymaster_info::PaymasterBudgetFeeUnit;
use slot::graphql::paymaster::PaymasterInfo;
use slot::graphql::GraphQLQuery;

#[derive(Debug, Args)]
#[command(next_help_heading = "Paymaster info options")]
pub struct InfoArgs {}

impl InfoArgs {
    pub async fn run(&self, name: String) -> Result<()> {
        let credentials = Credentials::load()?;

        let variables = paymaster_info::Variables { name: name.clone() };
        let request_body = PaymasterInfo::build_query(variables);

        let client = Client::new_with_token(credentials.access_token);

        let data: paymaster_info::ResponseData = client.query(&request_body).await?;

        match data.paymaster {
            Some(paymaster) => {
                // Format budget with 2 decimal places by dividing by 1e6
                let budget_formatted = paymaster.budget as f64 / 1e6;
                let strk_fees_formatted = paymaster.strk_fees as f64 / 1e6;
                let credit_fees_formatted = paymaster.credit_fees as f64 / 1e6;

                // Convert budget fee unit to string
                let budget_unit = match paymaster.budget_fee_unit {
                    PaymasterBudgetFeeUnit::CREDIT => "CREDIT",
                    PaymasterBudgetFeeUnit::STRK => "STRK",
                    _ => "UNKNOWN",
                };
                println!("\n🔍 Paymaster Info for '{}'", name);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                println!("🏢 Details:");
                println!(
                    "  • Team: {}",
                    paymaster
                        .team
                        .as_ref()
                        .map(|t| t.name.as_str())
                        .unwrap_or("Unknown")
                );
                println!(
                    "  • Active: {}",
                    if paymaster.active {
                        "✅ Yes"
                    } else {
                        "❌ No"
                    }
                );

                println!("\n💰 Budget:");
                let usd_equivalent = match paymaster.budget_fee_unit {
                    PaymasterBudgetFeeUnit::CREDIT => budget_formatted * 0.01, // 100 credit = 1 USD
                    _ => 0.0,
                };

                if usd_equivalent > 0.0 {
                    println!(
                        "  • Amount: {} {} (${:.2} USD)",
                        budget_formatted as i64, budget_unit, usd_equivalent
                    );
                } else {
                    println!("  • Amount: {} {}", budget_formatted as i64, budget_unit);
                }

                // Only display the relevant fee type based on budget unit
                match paymaster.budget_fee_unit {
                    PaymasterBudgetFeeUnit::STRK => {
                        println!("  • Total Spent: {:.2} STRK", strk_fees_formatted);
                    }
                    PaymasterBudgetFeeUnit::CREDIT => {
                        println!("  • Total Spent: {:.2} CREDIT", credit_fees_formatted);
                    }
                    _ => {}
                }

                println!("\n📋 Policies:");
                println!("  • Count: {}", paymaster.policies.total_count);
            }
            None => {
                println!("Paymaster '{}' not found", name);
            }
        }

        Ok(())
    }
}
