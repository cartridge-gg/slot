use anyhow::anyhow;
use clap::Args;
use katana_cli::NodeArgs;

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Katana create options")]
pub struct KatanaCreateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,

    #[command(flatten)]
    pub node_args: NodeArgs,

    #[arg(long, short, value_name = "persistent mode")]
    #[arg(help = "Whether to run the service in persistent mode (saya).")]
    pub persistent: bool,

    #[arg(long, short, value_name = "network")]
    #[arg(help = "Network to use for the service. Only in persistent (saya) mode.")]
    pub network: Option<String>,
}

impl KatanaCreateArgs {
    /// Validate the provided arguments
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.network.is_some() && !self.persistent {
            return Err(anyhow!(
                "The `network` option can only be supplied when `--persistent` is enabled.",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Katana update options")]
pub struct KatanaUpdateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,

    #[command(flatten)]
    pub node_args: NodeArgs,
}

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Katana account options")]
pub struct KatanaAccountArgs {}
