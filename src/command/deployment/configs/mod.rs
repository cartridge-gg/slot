use clap::{Args, Subcommand};

use self::{madara::Madara, torii::Torii};

mod madara;
mod torii;

#[derive(Debug, Args, Clone, serde::Serialize)]
#[command(next_help_heading = "Machine Specs")]
pub struct MachineSpecs {
    #[arg(long, default_value = "32")]
    #[arg(value_name = "memory")]
    #[arg(help = "The amount of memory.")]
    memory: u8,
    #[arg(long, default_value = "2")]
    #[arg(value_name = "cpu")]
    #[arg(help = "The number of cpu cores.")]
    cpu: u8,
}

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum CreateCommands {
    #[command(about = "Madara deployment.")]
    Madara(Madara),
    #[command(about = "Torii deployment.")]
    Torii(Torii),
}
