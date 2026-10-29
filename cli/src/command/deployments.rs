use anyhow::{bail, Result};
use clap::Args;

/// Slot Katana and Torii deployments are discontinued. The command is kept
/// hidden so existing scripts get a clear error instead of a parse failure.
#[derive(Debug, Args)]
pub struct Deployments {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    args: Vec<String>,
}

impl Deployments {
    pub async fn run(&self) -> Result<()> {
        bail!(
            "Slot deployments have been discontinued. Katana and Torii deployments can no \
             longer be created, updated, or managed with `slot deployments`."
        )
    }
}
