use anyhow::Result;
use clap::Args;
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::rpc::list_rpc_api_keys::{ResponseData, Variables};
use slot::graphql::rpc::ListRpcApiKeys;
use slot::graphql::GraphQLQuery;

#[derive(Debug, Args)]
#[command(next_help_heading = "List RPC tokens options")]
pub struct ListArgs {
    #[arg(long, help = "Team name to list tokens for.")]
    team: String,
}

impl ListArgs {
    pub async fn run(&self) -> Result<()> {
        let request_body = ListRpcApiKeys::build_query(Variables {
            team_name: self.team.clone(),
            first: Some(100),
            after: None,
            where_: None,
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let data: ResponseData = client.query(&request_body).await?;

        if let Some(connection) = data.rpc_api_keys {
            if let Some(edges) = connection.edges {
                let tokens: Vec<_> = edges
                    .iter()
                    .filter_map(|edge| edge.as_ref())
                    .filter_map(|edge| edge.node.as_ref())
                    .collect();

                if tokens.is_empty() {
                    println!("\nNo RPC API keys found for team '{}'", self.team);
                    return Ok(());
                }

                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec![
                        Cell::new("ID"),
                        Cell::new("Name"),
                        Cell::new("Key Prefix"),
                        Cell::new("Active"),
                        Cell::new("Created At"),
                        Cell::new("Last Used"),
                    ]);

                for token in tokens {
                    table.add_row(vec![
                        Cell::new(&token.id),
                        Cell::new(&token.name),
                        Cell::new(&token.key_prefix),
                        Cell::new(if token.active { "✓" } else { "✗" }),
                        Cell::new(&token.created_at),
                        Cell::new(token.last_used_at.as_ref().map_or("-", |s| s.as_str())),
                    ]);
                }

                println!("\nRPC API Keys for team '{}':", self.team);
                println!("{table}");
            }
        }

        Ok(())
    }
}
