use anyhow::Result;
use chrono::prelude::*;
use clap::Args;
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
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

        for edge in edges.into_iter().flatten() {
            if let Some(node) = edge.node {
                let is_current_month = node.month == current_month;

                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .set_content_arrangement(ContentArrangement::Dynamic);

                let title = if is_current_month {
                    format!("Invoices for team `{}` (Current Month)", team_name)
                } else {
                    format!("Invoices for team `{}`", team_name)
                };

                table.set_header(vec![title, "".to_string()]);

                table.add_row(vec![Cell::new("Month"), Cell::new(&node.month)]);
                table.add_row(vec![
                    Cell::new("Total Credits"),
                    Cell::new(format_credits(node.total_credits)),
                ]);
                table.add_row(vec![
                    Cell::new("Total Debits"),
                    Cell::new(format_credits(node.total_debits)),
                ]);
                table.add_row(vec![
                    Cell::new("Slot Debits"),
                    Cell::new(format_credits(node.slot_debits)),
                ]);
                table.add_row(vec![
                    Cell::new("Paymaster Debits"),
                    Cell::new(format_credits(node.paymaster_debits)),
                ]);
                table.add_row(vec![
                    Cell::new("Incubator Credits"),
                    Cell::new(format_credits(node.incubator_credits)),
                ]);
                table.add_row(vec![
                    Cell::new("Net Amount"),
                    Cell::new(format_credits(node.net_amount)),
                ]);
                table.add_row(vec![
                    Cell::new("Incubator Stage"),
                    Cell::new(node.incubator_stage.unwrap_or_else(|| "None".to_string())),
                ]);
                table.add_row(vec![
                    Cell::new("Finalized"),
                    Cell::new(if node.finalized { "Yes" } else { "No" }),
                ]);

                println!("{table}");
                println!();
            }
        }

        Ok(())
    }
}

fn format_credits(credits: i64) -> String {
    let dollars = credits as f64 / 100.0 / 1e6;
    format!("${:.2}", dollars)
}
