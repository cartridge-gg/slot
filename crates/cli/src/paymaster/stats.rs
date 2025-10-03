use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use clap::Args;
use slot_core::credentials::Credentials;
use slot_graphql::api::Client;
use slot_graphql::paymaster::paymaster_stats;
use slot_graphql::paymaster::PaymasterStats;
use slot_graphql::GraphQLQuery;
use std::time::{SystemTime, UNIX_EPOCH};

use super::utils;

#[derive(Debug, Args)]
#[command(next_help_heading = "Paymaster stats options")]
pub struct StatsArgs {
    #[arg(
        long,
        help = "Time period to look back (e.g., 1hr, 2min, 24hr, 1day, 1week). Default is 24hr.",
        default_value = "24hr"
    )]
    last: String,
}

impl StatsArgs {
    pub async fn run(&self, name: String) -> Result<()> {
        // 1. Load Credentials
        let credentials = Credentials::load()?;

        // 2. Parse the time duration
        let duration = utils::parse_duration(&self.last)?;

        // 3. Calculate the "since" timestamp
        let now = SystemTime::now();
        let since_time = now - duration;
        let since_timestamp = since_time
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("Invalid time calculation"))?
            .as_secs();

        // 4. Convert to RFC3339 format
        let since_rfc3339 = DateTime::<Utc>::from_timestamp(since_timestamp as i64, 0)
            .ok_or_else(|| anyhow!("Invalid timestamp"))?
            .to_rfc3339();

        // 5. Build Query Variables
        let variables = paymaster_stats::Variables {
            paymaster_name: name.clone(),
            since: since_rfc3339,
        };

        let request_body = PaymasterStats::build_query(variables);

        // 6. Create Client
        let client = Client::new_with_token(credentials.access_token);

        let data: paymaster_stats::ResponseData = client.query(&request_body).await?;

        // 8. Print Results
        let stats = &data.paymaster_stats;

        println!("\n📊 Paymaster Stats for '{}' (Last {})", name, self.last);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📈 Transactions:");
        println!("  • Total: {}", stats.total_transactions);
        println!("  • Successful: {}", stats.successful_transactions);
        println!("  • Reverted: {}", stats.reverted_transactions);

        if stats.total_transactions > 0 {
            let success_rate =
                (stats.successful_transactions as f64 / stats.total_transactions as f64) * 100.0;
            println!("  • Success Rate: {:.1}%", success_rate);

            // Calculate TPS
            let duration_seconds = duration.as_secs() as f64;
            let tps = stats.total_transactions as f64 / duration_seconds;
            println!("  • TPS: {:.4}", tps);
        }

        println!("\n💰 Fees (USD):");
        println!(
            "  • Total ({}): ${:.2}",
            self.last,
            stats.total_usd_fees.unwrap_or(0.0)
        );
        println!("  • Average: ${:.6}", stats.avg_usd_fee.unwrap_or(0.0));
        println!("  • Minimum: ${:.6}", stats.min_usd_fee.unwrap_or(0.0));
        println!("  • Maximum: ${:.6}", stats.max_usd_fee.unwrap_or(0.0));

        println!("\n👥 Users:");
        println!("  • Unique Users: {}", stats.unique_users);

        Ok(())
    }
}
