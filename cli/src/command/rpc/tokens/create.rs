use anyhow::{Ok, Result};
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::rpc::create_rpc_api_key;
use slot::graphql::rpc::CreateRpcApiKey;
use slot::graphql::GraphQLQuery;

#[derive(Debug, Args)]
#[command(next_help_heading = "Create RPC token options")]
pub struct CreateArgs {
    #[arg(help = "Name for the RPC token.")]
    name: String,

    #[arg(long, help = "Team name to associate the token with.")]
    team: String,
}

impl CreateArgs {
    pub async fn run(&self) -> Result<()> {
        let credentials = Credentials::load()?;

        let variables = create_rpc_api_key::Variables {
            team_name: self.team.clone(),
            name: self.name.clone(),
        };
        let request_body = CreateRpcApiKey::build_query(variables);

        let client = Client::new_with_token(credentials.access_token);

        let data: create_rpc_api_key::ResponseData = client.query(&request_body).await?;

        println!("\n✅ RPC API Key Created Successfully");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        println!("🔑 Details:");
        println!("  • ID: {}", data.create_rpc_api_key.api_key.id);
        println!("  • Name: {}", data.create_rpc_api_key.api_key.name);
        println!("  • Team: {}", self.team);
        println!(
            "  • Created: {}",
            data.create_rpc_api_key.api_key.created_at
        );

        println!("\n🔐 Secret Key:");
        println!("  • {}", data.create_rpc_api_key.secret_key);

        println!("\n⚠️  Important: Save this secret key securely - it won't be shown again!");
        println!(
            "🔍 Key Prefix (for identification): {}",
            data.create_rpc_api_key.api_key.key_prefix
        );

        Ok(())
    }
}
