use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::paymaster::paymaster_transactions;
use slot::graphql::paymaster::paymaster_transactions::{PaymasterTransactionFilter, PaymasterTransactionOrder};
use slot::graphql::paymaster::PaymasterTransactions;
use slot::graphql::GraphQLQuery;
use std::time::{SystemTime, UNIX_EPOCH};

use super::utils;

#[derive(Debug, Args)]
#[command(next_help_heading = "Transaction query options")]
pub struct TransactionArgs {
    #[arg(long, help = "Filter transactions by status (SUCCESSFUL, REVERTED, ALL).")]
    filter: Option<String>,
    
    #[arg(long, help = "Order transactions by (FEES_ASC, FEES_DESC, EXECUTED_AT_ASC, EXECUTED_AT_DESC).")]
    order_by: Option<String>,
    
    #[arg(
        long,
        help = "Time period to look back (e.g., 1hr, 2min, 24hr, 1day, 1week). Default is 24hr.",
        default_value = "24hr"
    )]
    last: String,
    
    #[arg(long, help = "Maximum number of transactions to return.", default_value = "10")]
    limit: Option<i64>,
}

#[derive(Debug)]
pub struct TransactionDisplay {
    pub transaction_hash: String,
    pub executed_at: String,
    pub status: String,
    pub usd_fee: String,
}

impl TransactionArgs {
    pub async fn run(&self, paymaster_name: String) -> Result<()> {
        let credentials = Credentials::load()?;
        
        // Parse the filter enum
        let filter = match self.filter.as_deref() {
            Some(f) => match f.to_uppercase().as_str() {
                "SUCCESSFUL" => Some(PaymasterTransactionFilter::SUCCESSFUL),
                "REVERTED" => Some(PaymasterTransactionFilter::REVERTED),
                "ALL" => Some(PaymasterTransactionFilter::ALL),
                _ => return Err(anyhow!("Invalid filter: {}. Use SUCCESSFUL, REVERTED, or ALL", f)),
            },
            None => None,
        };
        
        // Parse the order_by enum, default to EXECUTED_AT_DESC if None
        let order_by = match self.order_by.as_deref() {
            Some(o) => match o.to_uppercase().as_str() {
                "FEES_ASC" => Some(PaymasterTransactionOrder::FEES_ASC),
                "FEES_DESC" => Some(PaymasterTransactionOrder::FEES_DESC),
                "EXECUTED_AT_ASC" => Some(PaymasterTransactionOrder::EXECUTED_AT_ASC),
                "EXECUTED_AT_DESC" => Some(PaymasterTransactionOrder::EXECUTED_AT_DESC),
                _ => return Err(anyhow!("Invalid order_by: {}. Use FEES_ASC, FEES_DESC, EXECUTED_AT_ASC, or EXECUTED_AT_DESC", o)),
            },
            None => Some(PaymasterTransactionOrder::EXECUTED_AT_DESC), // Default to EXECUTED_AT_DESC
        };
        
        // Parse the time duration using shared utility
        let duration = utils::parse_duration(&self.last)?;

        // Calculate the "since" timestamp
        let now = SystemTime::now();
        let since_time = now - duration;
        let since_timestamp = since_time
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("Invalid time calculation"))?
            .as_secs();

        // Convert to RFC3339 format
        let since_rfc3339 = DateTime::<Utc>::from_timestamp(since_timestamp as i64, 0)
            .ok_or_else(|| anyhow!("Invalid timestamp"))?
            .to_rfc3339();
        
        let variables = paymaster_transactions::Variables {
            paymaster_name: paymaster_name.clone(),
            filter,
            order_by,
            since: since_rfc3339,
            limit: self.limit,
        };
        
        let request_body = PaymasterTransactions::build_query(variables);
        let client = Client::new_with_token(credentials.access_token);
        let data: paymaster_transactions::ResponseData = client.query(&request_body).await?;
        
        let transactions = data.paymaster_transactions;
        
        if transactions.is_empty() {
            println!("\n📊 Paymaster Transactions for '{}' (Last {})", paymaster_name, self.last);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("No transactions found.");
            return Ok(());
        }
        
        // Convert to display format
        let display_transactions: Vec<TransactionDisplay> = transactions
            .into_iter()
            .map(|t| TransactionDisplay {
                transaction_hash: t.transaction_hash.clone(),
                executed_at: format_relative_time(&t.executed_at),
                status: format!("{:?}", t.status),
                usd_fee: format!("${:.4}", t.usd_fee),
            })
            .collect();
        
        // Print header with time reference like stats.rs
        println!("\n📊 Paymaster Transactions for '{}' (Last {})", paymaster_name, self.last);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        print_transactions_table(&display_transactions);
        
        Ok(())
    }
}

fn format_relative_time(executed_at: &str) -> String {
    // Parse the timestamp string to a DateTime
    let executed_time = match executed_at.parse::<DateTime<Utc>>() {
        Ok(dt) => dt,
        Err(_) => return executed_at.to_string(), // Fallback to original string if parsing fails
    };
    
    let now = Utc::now();
    let diff = now.signed_duration_since(executed_time);
    
    let total_seconds = diff.num_seconds();
    
    if total_seconds < 0 {
        return "in the future".to_string();
    }
    
    if total_seconds < 60 {
        return format!("{}s ago", total_seconds);
    }
    
    let minutes = total_seconds / 60;
    if minutes < 60 {
        return format!("{}m ago", minutes);
    }
    
    let hours = minutes / 60;
    if hours < 24 {
        return format!("{}h ago", hours);
    }
    
    let days = hours / 24;
    if days < 7 {
        return format!("{}d ago", days);
    }
    
    let weeks = days / 7;
    format!("{}w ago", weeks)
}

pub fn print_transactions_table(transactions: &[TransactionDisplay]) {
    if transactions.is_empty() {
        return;
    }

    // Print header - adjusted widths for full hash and relative time
    println!("{:<66} {:<12} {:<12} {:<12}", 
             "Transaction Hash", 
             "Executed", 
             "Status", 
             "USD Fee");
    println!("{}", "─".repeat(110));

    // Print transactions
    for transaction in transactions {
        println!("{:<66} {:<12} {:<12} {:<12}", 
                 transaction.transaction_hash,
                 transaction.executed_at,
                 transaction.status,
                 transaction.usd_fee);
    }
} 