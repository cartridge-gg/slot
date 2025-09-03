use anyhow::Result;
use clap::Args;
use colored::*;
use slot_core::credentials::Credentials;
use slot_graphql::api::Client;
use slot_graphql::auth::{me::*, Me};
use slot_graphql::GraphQLQuery;

#[derive(Debug, Args)]
pub struct InfoArgs;

fn format_usd(credits: i64) -> String {
    // format two digits currency
    let amount = credits as f64 / 100f64;
    // format two digits e.g. $1.02
    format!("${:.2}", amount)
}

impl InfoArgs {
    // TODO: find the account info from `credentials.json` first before making a request
    pub async fn run(&self) -> Result<()> {
        let credentials = Credentials::load()?;
        let client = Client::new_with_token(credentials.access_token);

        let request_body = Me::build_query(Variables {});
        let res: ResponseData = client.query(&request_body).await?;
        let info = res.me.clone().unwrap();
        println!("Username: {}", info.username);
        println!(
            "Balance: {}",
            // round usd to 2 digits
            format_usd(info.credits_plain)
        );

        println!();
        println!("Teams:");
        let teams = res.me.unwrap().teams.edges.unwrap();

        if teams.is_empty() {
            println!("  No teams yet");
        }

        for edge in teams {
            let team = edge.unwrap().node.unwrap();
            if team.deleted {
                continue;
            }
            println!();
            println!("  Name: {}", team.name);
            println!(
                "  Balance: {}",
                // round usd to 2 digits
                format_usd((team.credits as f64 / 1e6) as i64)
            );

            let total_balance = if let Some(incubator_stage) = &team.incubator_stage {
                println!("  Incubator Stage: {:?}", incubator_stage);

                // Determine total balance based on incubator stage
                // Match against the Debug representation of the enum
                match format!("{:?}", incubator_stage).as_str() {
                    "senpai" => 500000,  // $5k in cents
                    "sensei" => 2500000, // $25k in cents
                    _ => 0,
                }
            } else {
                0
            };

            println!("  Total Balance: {}", format_usd(total_balance));

            println!(
                "  Total Spent: {}",
                format_usd((team.total_debits as f64 / 1e6) as i64)
            );

            // Calculate remaining balance
            let remaining_balance = total_balance - ((team.total_debits as f64 / 1e6) as i64);
            println!(
                "  Remaining Incubator Credits: {}",
                format_usd(remaining_balance)
            );

            println!("  Deployments:");
            let deployments = team.deployments.edges.unwrap();
            let active_deployments: Vec<_> = deployments
                .iter()
                .filter_map(|edge| edge.as_ref())
                .filter_map(|edge| edge.node.as_ref())
                .filter(|deployment| format!("{:?}", deployment.status) != "deleted")
                .collect();

            if active_deployments.is_empty() {
                println!("    No deployments yet");
            }

            for deployment in active_deployments {
                let deprecated_indicator = if deployment.deprecated.unwrap_or(false) {
                    format!(" {}", "(deprecated!)".bold())
                } else {
                    String::new()
                };

                println!(
                    "    Deployment: {}/{}{}",
                    deployment.project, deployment.service_id, deprecated_indicator
                );
            }

            println!("  Members:");
            let members = team.membership.edges.unwrap();
            for edge in members {
                let member = edge.unwrap().node.unwrap();
                println!(
                    "    Member: {} ({:?})",
                    member.account.username, member.role
                );
            }
        }

        Ok(())
    }
}
