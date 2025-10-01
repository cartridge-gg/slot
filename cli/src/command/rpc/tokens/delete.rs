use anyhow::{Ok, Result};
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::rpc::delete_rpc_api_key;
use slot::graphql::rpc::DeleteRpcApiKey;
use slot::graphql::GraphQLQuery;

#[derive(Debug, Args)]
#[command(next_help_heading = "Delete RPC token options")]
pub struct DeleteArgs {
    #[arg(help = "ID of the RPC token to delete.")]
    id: String,
}

impl DeleteArgs {
    pub async fn run(&self) -> Result<()> {
        let credentials = Credentials::load()?;

        let variables = delete_rpc_api_key::Variables {
            id: self.id.clone(),
        };
        let request_body = DeleteRpcApiKey::build_query(variables);

        let client = Client::new_with_token(credentials.access_token);

        let data: delete_rpc_api_key::ResponseData = client.query(&request_body).await?;

        if data.delete_rpc_api_key {
            println!("\n✅ RPC API Key Deleted Successfully");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("🗑️  API Key ID {} has been removed", self.id);
        } else {
            println!("❌ Failed to delete RPC API key");
        }

        Ok(())
    }
}
