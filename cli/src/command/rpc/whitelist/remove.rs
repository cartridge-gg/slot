use anyhow::{Ok, Result};
use clap::Args;
use slot::api::Client;
use slot::credential::Credentials;
use slot::graphql::rpc::delete_rpc_cors_domain;
use slot::graphql::rpc::DeleteRpcCorsDomain;
use slot::graphql::GraphQLQuery;

#[derive(Debug, Args)]
#[command(next_help_heading = "Remove whitelist origin options")]
pub struct RemoveArgs {
    #[arg(help = "ID of the whitelist origin to remove.")]
    id: String,
}

impl RemoveArgs {
    pub async fn run(&self) -> Result<()> {
        let credentials = Credentials::load()?;

        let variables = delete_rpc_cors_domain::Variables {
            id: self.id.clone(),
        };
        let request_body = DeleteRpcCorsDomain::build_query(variables);

        let client = Client::new_with_token(credentials.access_token);

        let data: delete_rpc_cors_domain::ResponseData = client.query(&request_body).await?;

        if data.delete_rpc_cors_domain {
            println!("\n✅ Origin Removed from CORS Whitelist Successfully");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("🗑️  CORS domain ID {} has been removed", self.id);
        } else {
            println!("❌ Failed to remove origin from CORS whitelist");
        }

        Ok(())
    }
}
