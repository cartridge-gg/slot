use std::str::FromStr;

use anyhow::{anyhow, ensure, Result};
use clap::Parser;
use slot::session::{self, PolicyMethod};
use starknet::core::types::Felt;
use url::Url;

#[derive(Debug, Parser)]
pub struct CreateSession {
    #[arg(long)]
    #[arg(value_name = "URL")]
    // #[arg(default_value = "http://localhost:5050")]
    #[arg(help = "The RPC URL of the network you want to create a session for.")]
    rpc_url: String,

    #[arg(help = "The session's policies.")]
    #[arg(value_parser = parse_policy_method)]
    #[arg(required = true)]
    policies: Vec<PolicyMethod>,
}

impl CreateSession {
    pub async fn run(&self) -> Result<()> {
        let url = Url::parse(&self.rpc_url)?;
        let session = session::create(url, &self.policies).await?;
        session::store(session.chain_id, &session)?;
        Ok(())
    }
}

fn parse_policy_method(value: &str) -> Result<PolicyMethod> {
    let mut parts = value.split(',');

    let target = parts.next().ok_or(anyhow!("missing target"))?.to_owned();
    let target = Felt::from_str(&target)?;
    let method = parts.next().ok_or(anyhow!("missing method"))?.to_owned();

    ensure!(parts.next().is_none());

    Ok(PolicyMethod { target, method })
}
