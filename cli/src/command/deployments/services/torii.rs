use std::path::PathBuf;

use clap::Args;

#[derive(Clone, Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Torii create options")]
pub struct ToriiCreateArgs {
    #[arg(long, short = 'c')]
    #[arg(help = "Path to the Torii configuration file (TOML format). This is required.")]
    pub config: PathBuf,

    #[arg(long, default_value = "1")]
    #[arg(help = "The number of replicas to deploy.")]
    pub replicas: Option<i64>,

    #[arg(long)]
    #[arg(help = "The version of Torii to deploy.")]
    pub version: Option<String>,

    #[arg(long)]
    #[arg(help = "Enable database replication using litestream.")]
    pub replication: bool,
}

/// Update a Torii deployment.
///
/// The main purpose of update is usually to change slot configuration (replicate, regions, tier, etc...),
/// but it can also be used to change Torii parameters.
/// For the latter, it is only possible using the configuration file (and not each individual parameter in the CLI),
/// since the deployment has already been created with a configuration.
#[derive(Clone, Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Torii update options")]
pub struct ToriiUpdateArgs {
    #[arg(long)]
    #[arg(help = "The number of replicas to deploy.")]
    pub replicas: Option<i64>,

    #[arg(long)]
    #[arg(help = "The version of Torii to deploy.")]
    pub version: Option<String>,

    #[arg(long)]
    #[arg(
        help = "The path to the configuration file to use for the update. This will replace the existing configuration."
    )]
    pub config: Option<PathBuf>,
}
