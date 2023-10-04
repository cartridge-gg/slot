use clap::Args;

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Katana options")]
pub struct Katana {}
