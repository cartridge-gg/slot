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

                // Convert budget fee unit to string - display CREDIT as USD
                let budget_unit = match paymaster.budget_fee_unit {
                    PaymasterBudgetFeeUnit::CREDIT => "USD",
                    PaymasterBudgetFeeUnit::STRK => "STRK",
                    _ => "UNKNOWN",
                };

                // Calculate usage percentage and create progress bar
                let spent_amount = match paymaster.budget_fee_unit {
                    PaymasterBudgetFeeUnit::STRK => strk_fees_formatted,
                    PaymasterBudgetFeeUnit::CREDIT => credit_fees_formatted,
                    _ => 0.0,
                };

                let usage_percentage = if budget_formatted > 0.0 {
                    (spent_amount / budget_formatted * 100.0).min(100.0)
                } else {
                    0.0
                };

                // Create progress bar (40 characters wide)
                let bar_width = 30;
                let filled_width = (usage_percentage / 100.0 * bar_width as f64) as usize;
                let progress_bar = format!(
                    "[{}{}]",
                    "█".repeat(filled_width),
                    "░".repeat(bar_width - filled_width)
                );

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
                let budget_display = match paymaster.budget_fee_unit {
                    PaymasterBudgetFeeUnit::CREDIT => budget_formatted * 0.01, // Convert credits to USD
                    PaymasterBudgetFeeUnit::STRK => budget_formatted,
                    _ => 0.0,
                };

                if budget_display > 0.0 {
                    println!(
                        "  • Total: ${:.2} {}",
                        budget_display, budget_unit
                    );
                } else {
                    println!("  • Total: NONE (Please Top Up)");
                }

                // Only display the relevant fee type based on budget unit
                match paymaster.budget_fee_unit {
                    PaymasterBudgetFeeUnit::STRK => {
                        println!("  • Spent: {:.2} STRK", strk_fees_formatted);
                    }
                    PaymasterBudgetFeeUnit::CREDIT => {
                        let spent_usd = credit_fees_formatted * 0.01; // Convert credits to USD
                        println!("  • Spent: ${:.2} USD", spent_usd);
                    }
                    _ => {}
                }

                // Display usage progress bar
                if budget_formatted > 0.0 {
                    println!("  • Usage: {} {:.1}%", progress_bar, usage_percentage);
                }

                if paymaster.legacy_strk_fees > 0 || paymaster.legacy_eth_fees > 0 {
                    let legacy_strk_formatted = paymaster.legacy_strk_fees as f64 / 1e6;
                    let legacy_eth_formatted = paymaster.legacy_eth_fees as f64 / 1e6;
                    println!("\n💸 Outstanding Balance:");
                    println!("  • This is the balance due prior to self service migration.");
                    if paymaster.legacy_strk_fees > 0 {
                        println!("  • Spent STRK: {:.2}", legacy_strk_formatted);
                    }

                    if paymaster.legacy_eth_fees > 0 {
                        println!("  • Spent ETH: {:.4}", legacy_eth_formatted);
                    }
                }

                println!("\n🧾 Lifetime Transactions:");
                let total_successful =
                    paymaster.successful_transactions + paymaster.legacy_successful_transactions;
                let total_reverted =
                    paymaster.reverted_transactions + paymaster.legacy_reverted_transactions;
                println!("  • Total: {}", total_successful + total_reverted);
                println!("  • Successful: {}", total_successful);
                println!("  • Reverted: {}", total_reverted);

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
