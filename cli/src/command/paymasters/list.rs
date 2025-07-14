use anyhow::Result;
use clap::Args;
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::paymaster::list_paymasters::PaymasterBudgetFeeUnit;
use slot::graphql::paymaster::list_paymasters::{ResponseData, Variables};
use slot::graphql::paymaster::ListPaymasters;
use slot::graphql::GraphQLQuery;

const BUDGET_DECIMALS: i64 = 1_000_000;

#[derive(Debug, Args)]
#[command(next_help_heading = "List paymasters options")]
pub struct ListArgs {}

impl ListArgs {
    pub async fn run(&self) -> Result<()> {
        let request_body = ListPaymasters::build_query(Variables {});

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        // 5. Process and Print Results - adapt to the new nested structure
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Paymaster", "Team", "Budget", "Active"]);

        let data: ResponseData = client.query(&request_body).await?;
        let mut paymasters_found = false; // Ensure this is declared before the selection or adjust scope
        if let Some(me) = data.me {
            if let Some(teams_edges) = me.teams.edges {
                // Iterate through teams, filtering out None edges and nodes
                let paymasters_data = teams_edges
                    .iter()
                    .filter_map(|team_edge_opt| team_edge_opt.as_ref()) // Get &TeamEdge, filtering None
                    .filter_map(|team_edge| team_edge.node.as_ref()) // Get &TeamNode, filtering None
                    .filter_map(|team_node| {
                        // Get paymaster edges for the team, keeping team_node info
                        // Handle Option<PaymastersConnection> and Option<Vec<Option<PaymasterEdge>>>
                        team_node
                            .paymasters // Access the connection struct directly
                            .edges // Access the 'edges' field (likely Option<Vec<Option<Edge>>>)
                            .as_ref() // Call as_ref() on the Option<Vec<...>>
                            .map(|pm_edges| (team_node, pm_edges)) // pm_edges is now &Vec<Option<Edge>>
                    })
                    .flat_map(|(team_node, pm_edges)| {
                        // Flatten the list of paymasters across all teams
                        pm_edges
                            .iter()
                            .filter_map(|pm_edge_opt| pm_edge_opt.as_ref()) // Get &PaymasterEdge, filtering None
                            .filter_map(move |pm_edge| {
                                // Get &PaymasterNode from Option<PaymasterNode>, keeping team_node info
                                pm_edge.node.as_ref().map(|pm_node| (team_node, pm_node))
                            })
                    });

                // Populate the table
                for (team_node, pm_node) in paymasters_data {
                    paymasters_found = true;
                    let budget = match pm_node.budget_fee_unit {
                        PaymasterBudgetFeeUnit::CREDIT => {
                            let budget_usd = (pm_node.budget / BUDGET_DECIMALS) as f64 * 0.01;
                            format!("${:.2} USD", budget_usd)
                        }
                        PaymasterBudgetFeeUnit::STRK => {
                            format!("{} STRK", pm_node.budget / BUDGET_DECIMALS)
                        }
                        _ => format!("{} UNKNOWN", pm_node.budget / BUDGET_DECIMALS),
                    };
                    table.add_row(vec![
                        Cell::new(pm_node.name.as_str()),
                        Cell::new(&team_node.name),
                        Cell::new(&budget),
                        Cell::new(pm_node.active.to_string()),
                    ]);
                }
            }
        }

        if !paymasters_found {
            println!("No paymasters found for your teams.");
        } else {
            println!("{table}");
        }

        Ok(())
    }
}
