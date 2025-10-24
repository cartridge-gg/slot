use anyhow::Result;
use clap::Args;
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use serde_json::json;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::rpc::list_rpc_logs::ResponseData;
use std::time::SystemTime;

use crate::command::paymaster::utils::parse_duration;

#[derive(Debug, Args)]
#[command(next_help_heading = "List RPC logs options")]
pub struct LogsArgs {
    #[arg(long, help = "Team name to list logs for.")]
    team: String,

    #[arg(
        long,
        short = 'n',
        default_value = "10",
        help = "Number of logs to fetch (max 50)."
    )]
    limit: i64,

    #[arg(long, help = "Show logs after this cursor for pagination.")]
    after: Option<String>,

    #[arg(
        long,
        short = 's',
        help = "Filter logs from the last duration (e.g., '30s', '5m', '2h', '1d'). Max 1 week."
    )]
    since: Option<String>,
}

impl LogsArgs {
    pub async fn run(&self) -> Result<()> {
        // Validate limit is within bounds
        let limit = if self.limit > 50 {
            println!("Warning: Limit exceeds maximum of 50. Using 50 instead.");
            50
        } else if self.limit < 1 {
            println!("Warning: Limit must be at least 1. Using 1 instead.");
            1
        } else {
            self.limit
        };

        // Build where filter if time filter is provided
        let where_filter = if let Some(ref since_str) = self.since {
            let duration = parse_duration(since_str)?;
            let since_time = SystemTime::now()
                .checked_sub(duration)
                .ok_or_else(|| anyhow::anyhow!("Time calculation overflow"))?;

            // Convert to RFC3339 timestamp for GraphQL
            let timestamp = chrono::DateTime::<chrono::Utc>::from(since_time)
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

            json!({ "timestampGTE": timestamp })
        } else {
            json!(null)
        };

        // Build the GraphQL query manually with JSON to support the where clause
        let request_body = json!({
            "query": r#"
                query ListRpcLogs($teamName: String!, $first: Int, $after: Cursor, $where: RPCLogWhereInput) {
                    rpcLogs(teamName: $teamName, first: $first, after: $after, where: $where) {
                        edges {
                            node {
                                id
                                teamID
                                apiKeyID
                                corsDomainID
                                clientIP
                                userAgent
                                referer
                                network
                                method
                                responseStatus
                                responseSizeBytes
                                durationMs
                                isInternal
                                costCredits
                                timestamp
                                processedAt
                            }
                            cursor
                        }
                        pageInfo {
                            hasNextPage
                            hasPreviousPage
                            startCursor
                            endCursor
                        }
                        totalCount
                    }
                }
            "#,
            "variables": {
                "teamName": self.team,
                "first": limit,
                "after": self.after,
                "where": where_filter,
            }
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let data: ResponseData = client.query(&request_body).await?;

        if let Some(connection) = data.rpc_logs {
            if let Some(edges) = connection.edges {
                let logs: Vec<_> = edges
                    .iter()
                    .filter_map(|edge| edge.as_ref())
                    .filter_map(|edge| edge.node.as_ref())
                    .collect();

                if logs.is_empty() {
                    println!("\nNo RPC logs found for team '{}'", self.team);
                    return Ok(());
                }

                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec![
                        Cell::new("Timestamp"),
                        Cell::new("Network"),
                        Cell::new("Method"),
                        Cell::new("Status"),
                        Cell::new("Duration (ms)"),
                        Cell::new("Size (bytes)"),
                        Cell::new("Cost (credits)"),
                        Cell::new("API Key ID"),
                        Cell::new("CORS Domain ID"),
                        Cell::new("Client IP"),
                    ]);

                for log in logs {
                    table.add_row(vec![
                        Cell::new(&log.timestamp),
                        Cell::new(format!("{:?}", log.network)),
                        Cell::new(log.method.as_ref().map_or("-", |s| s.as_str())),
                        Cell::new(log.response_status.to_string()),
                        Cell::new(log.duration_ms.to_string()),
                        Cell::new(log.response_size_bytes.to_string()),
                        Cell::new(log.cost_credits.to_string()),
                        Cell::new(log.api_key_id.as_deref().unwrap_or("-")),
                        Cell::new(log.cors_domain_id.as_deref().unwrap_or("-")),
                        Cell::new(&log.client_ip),
                    ]);
                }

                println!("\nRPC Logs for team '{}':", self.team);
                println!("{table}");

                // Show pagination info if available
                let page_info = connection.page_info;
                if page_info.has_next_page {
                    if let Some(end_cursor) = page_info.end_cursor {
                        println!(
                            "\nMore logs available. Use --after {} to see next page",
                            end_cursor
                        );
                    }
                }

                println!("\nTotal logs: {}", connection.total_count);
            }
        }

        Ok(())
    }
}
