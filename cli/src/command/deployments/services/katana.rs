use std::path::PathBuf;

use anyhow::anyhow;
use clap::Args;
use katana_cli::NodeArgs;

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Katana create options")]
pub struct KatanaCreateArgs {
    #[command(flatten)]
    pub node_args: NodeArgs,

    #[arg(long, short, value_name = "provable mode")]
    #[arg(help = "Whether to run the service in provable mode.")]
    pub provable: bool,

    #[arg(long, short, value_name = "network")]
    #[arg(help = "Network to use for the service. Only in provable mode.")]
    pub network: Option<String>,

    #[arg(long, short, value_name = "saya")]
    #[arg(
        help = "Whether to start a saya instance alongside the provable Katana. Only in provable mode."
    )]
    pub saya: bool,

    #[arg(long)]
    #[arg(help = "Enable observability for monitoring and metrics.")]
    pub observability: bool,
}

impl KatanaCreateArgs {
    /// Validate the provided arguments
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.network.is_some() && !self.provable {
            return Err(anyhow!(
                "The `network` option can only be supplied when `--provable` is enabled.",
            ));
        }
        if self.saya && !self.provable {
            return Err(anyhow!(
                "The `saya` option can only be supplied when `--provable` is enabled.",
            ));
        }
        Ok(())
    }
}

/// Update a Katana deployment.
///
/// The main purpose of update is usually to change slot configuration (regions, tier, etc...),
/// but it can also be used to change Katana parameters.
/// For the latter, it is only possible using the configuration file (and not each individual parameter in the CLI),
/// since the deployment has already been created with a configuration.
#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Katana update options")]
pub struct KatanaUpdateArgs {
    #[arg(long)]
    #[arg(help = "Enable observability for monitoring and metrics.")]
    pub observability: Option<bool>,

    #[arg(long)]
    #[arg(
        help = "The path to the configuration file to use for the update. This will replace the existing configuration."
    )]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Katana account options")]
pub struct KatanaAccountArgs {}
