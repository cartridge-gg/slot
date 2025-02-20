use clap::Args;

use torii_cli::ToriiArgs;

#[derive(Clone, Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Torii create options")]
pub struct ToriiCreateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,

    #[arg(long, default_value = "1")]
    #[arg(help = "The number of replicas to deploy.")]
    pub replicas: Option<i64>,

    #[command(flatten)]
    pub torii_args: ToriiArgs,
}

#[derive(Clone, Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Torii update options")]
pub struct ToriiUpdateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,

    #[arg(long)]
    #[arg(help = "The number of replicas to deploy.")]
    pub replicas: Option<i64>,

    #[command(flatten)]
    pub torii_args: ToriiArgs,
}
