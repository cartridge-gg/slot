use anyhow::Result;
use clap::Args;
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::rpc::list_rpc_cors_domains::{ResponseData, Variables};
use slot::graphql::rpc::ListRpcCorsDomains;
use slot::graphql::GraphQLQuery;

#[derive(Debug, Args)]
#[command(next_help_heading = "List whitelist origins options")]
pub struct ListArgs {
    #[arg(long, help = "Team name to list whitelist origins for.")]
    team: String,
}

impl ListArgs {
    pub async fn run(&self) -> Result<()> {
        let request_body = ListRpcCorsDomains::build_query(Variables {
            team_name: self.team.clone(),
            first: Some(100),
            after: None,
            where_: None,
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let data: ResponseData = client.query(&request_body).await?;

        if let Some(connection) = data.rpc_cors_domains {
            if let Some(edges) = connection.edges {
                let domains: Vec<_> = edges
                    .iter()
                    .filter_map(|edge| edge.as_ref())
                    .filter_map(|edge| edge.node.as_ref())
                    .collect();

                if domains.is_empty() {
                    println!("\nNo CORS domains found for team '{}'", self.team);
                    return Ok(());
                }

                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec![
                        Cell::new("ID"),
                        Cell::new("Domain"),
                        Cell::new("Created At"),
                    ]);

                for domain in domains {
                    table.add_row(vec![
                        Cell::new(&domain.id),
                        Cell::new(&domain.domain),
                        Cell::new(&domain.created_at),
                    ]);
                }

                println!("\nCORS Domains for team '{}':", self.team);
                println!("{table}");
            }
        }

        Ok(())
    }
}
