use anyhow::Result;
use clap::Args;
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::team::team_balance::Variables;
use slot::graphql::team::TeamBalance;
use slot::graphql::GraphQLQuery;

#[derive(Debug, Args)]
pub struct BalanceArgs {}

impl BalanceArgs {
    pub async fn run(&self, team_name: String) -> Result<()> {
        let request_body = TeamBalance::build_query(Variables {
            team: team_name.clone(),
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let data: slot::graphql::team::team_balance::ResponseData =
            client.query(&request_body).await?;
        let team = data
            .team
            .ok_or_else(|| anyhow::anyhow!("Team '{}' not found", team_name))?;

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(vec![
            format!("Balance for team `{}`", team_name),
            "".to_string(),
        ]);

        table.add_row(vec![Cell::new("USD"), Cell::new(format_usd(team.credits))]);
        table.add_row(vec![Cell::new("STRK"), Cell::new(format_strk(team.strk))]);

        println!("{table}");

        Ok(())
    }
}

fn format_usd(credits: i64) -> String {
    let dollars = credits as f64 / 100.0 / 1e6;
    format!("${:.2}", dollars)
}

fn format_strk(strk: i64) -> String {
    let amount = strk as f64 / 1e6;
    format!("{:.6}", amount)
}
