use clap::Args;

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Madara options")]
pub struct Madara {}
