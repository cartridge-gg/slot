use clap::Args;

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Madara create options")]
pub struct MadaraCreateArgs {
  #[arg(long, short, value_name = "version")]
  #[arg(help = "Service version to use.")]
  pub version: Option<String>,

  #[arg(long, short, value_name = "name")]
  #[arg(help = "Name.")]
  pub name: Option<String>,
}
