use anyhow::{Ok, Result};
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::rpc::create_rpc_cors_domain;
use slot::graphql::rpc::CreateRpcCorsDomain;
use slot::graphql::GraphQLQuery;

#[derive(Debug, Args)]
#[command(next_help_heading = "Add whitelist origin options")]
pub struct AddArgs {
    #[arg(help = "Origin URL to add to whitelist.")]
    origin: String,

    #[arg(long, help = "Team name to add the origin for.")]
    team: String,
}

impl AddArgs {
    pub async fn run(&self) -> Result<()> {
        let credentials = Credentials::load()?;

        let variables = create_rpc_cors_domain::Variables {
            team_name: self.team.clone(),
            domain: self.origin.clone(),
            rate_limit_per_minute: Some(60), // Default rate limit
        };
        let request_body = CreateRpcCorsDomain::build_query(variables);

        let client = Client::new_with_token(credentials.access_token);

        let data: create_rpc_cors_domain::ResponseData = client.query(&request_body).await?;

        println!("\n✅ Origin Added to CORS Whitelist Successfully");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        println!("🌐 Details:");
        println!("  • ID: {}", data.create_rpc_cors_domain.id);
        println!("  • Domain: {}", data.create_rpc_cors_domain.domain);
        println!("  • Team: {}", self.team);
        println!("  • Created: {}", data.create_rpc_cors_domain.created_at);

        Ok(())
    }
}
