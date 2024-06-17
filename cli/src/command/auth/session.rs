use std::str::FromStr;

use anyhow::{anyhow, ensure, Result};
use clap::Parser;
use slot::session::{self, Policy};
use starknet::core::types::FieldElement;
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider};
use url::Url;

#[derive(Debug, Parser)]
pub struct CreateSession {
    #[arg(long)]
    #[arg(value_name = "URL")]
    // #[arg(default_value = "http://localhost:5050")]
    #[arg(help = "The RPC URL of the network you want to create a session for.")]
    rpc_url: String,

    #[arg(help = "The session's policies.")]
    #[arg(value_parser = parse_policy)]
    #[arg(required = true)]
    policies: Vec<Policy>,
}

impl CreateSession {
    pub async fn run(&self) -> Result<()> {
        let url = Url::parse(&self.rpc_url)?;
        let chain_id = get_network_chain_id(url.clone()).await?;
        let session = session::create(url, &self.policies).await?;
        session::store(chain_id, &session)?;
        Ok(())
    }
}

fn parse_policy(value: &str) -> Result<Policy> {
    let mut parts = value.split(',');

    let target = parts.next().ok_or(anyhow!("missing target"))?.to_owned();
    let target = FieldElement::from_str(&target)?;
    let method = parts.next().ok_or(anyhow!("missing method"))?.to_owned();

    ensure!(parts.next().is_none(), " bruh");

    Ok(Policy { target, method })
}

async fn get_network_chain_id(url: Url) -> Result<FieldElement> {
    let provider = JsonRpcClient::new(HttpTransport::new(url));
    Ok(provider.chain_id().await?)
}
