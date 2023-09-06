use clap::Args;

use super::MachineSpecs;

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Madara options")]
pub struct Madara {
    #[command(flatten)]
    requests: MachineSpecs,
    #[arg(long, default_value = "32")]
    #[arg(value_name = "storage")]
    #[arg(help = "Amount of storage to request.")]
    storage: i32,
}
