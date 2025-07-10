use anyhow::Result;
use chrono::prelude::*;
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::team::team_invoices::{InvoiceOrder, Variables};
use slot::graphql::team::team_invoices::{InvoiceOrderField, OrderDirection};
use slot::graphql::team::TeamInvoices;
use slot::graphql::GraphQLQuery;

#[derive(Debug, Args)]
pub struct InvoicesArgs {}

impl InvoicesArgs {
    pub async fn run(&self, team_name: String) -> Result<()> {
        let request_body = TeamInvoices::build_query(Variables {
            team: team_name.clone(),
            first: Some(100), // Get up to 100 invoices
            after: None,
            order_by: Some(InvoiceOrder {
                field: InvoiceOrderField::CREATED_AT,
                direction: OrderDirection::DESC, // Most recent first
            }),
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let data: slot::graphql::team::team_invoices::ResponseData =
            client.query(&request_body).await?;
        let team = data
            .team
            .ok_or_else(|| anyhow::anyhow!("Team '{}' not found", team_name))?;

        let edges = team.invoices.edges.unwrap_or_default();

        if edges.is_empty() {
            println!("No invoices found for team '{}'", team_name);
            return Ok(());
        }

        let current_month = Utc::now().format("%Y-%m").to_string();

        println!("Invoices for team '{}':", team_name);
        println!("=====================================");

        for edge in edges.into_iter().flatten() {
            if let Some(node) = edge.node {
                let is_current_month = node.month == current_month;
                let prefix = if is_current_month { ">>> " } else { "    " };

                println!("{}Month: {}", prefix, node.month);
                println!(
                    "{}Total Credits: {}",
                    prefix,
                    format_credits(node.total_credits)
                );
                println!(
                    "{}Total Debits: {}",
                    prefix,
                    format_credits(node.total_debits)
                );
                println!(
                    "{}Slot Debits: {}",
                    prefix,
                    format_credits(node.slot_debits)
                );
                println!(
                    "{}Paymaster Debits: {}",
                    prefix,
                    format_credits(node.paymaster_debits)
                );
                println!(
                    "{}Incubator Credits: {}",
                    prefix,
                    format_credits(node.incubator_credits)
                );
                println!("{}Net Amount: {}", prefix, format_credits(node.net_amount));
                println!(
                    "{}Incubator Stage: {}",
                    prefix,
                    node.incubator_stage.unwrap_or_else(|| "None".to_string())
                );
                println!(
                    "{}Finalized: {}",
                    prefix,
                    if node.finalized { "Yes" } else { "No" }
                );
                println!("{}---\n\n", prefix);
            }
        }

        Ok(())
    }
}

fn format_credits(credits: i64) -> String {
    let dollars = credits as f64 / 100.0;
    format!("${:.2}", dollars)
}
