use clap::{Subcommand, ValueEnum};

use self::{katana::KatanaArgs, torii::ToriiArgs};

mod katana;
mod madara;
mod torii;

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum CreateCommands {
    #[command(about = "Katana deployment.")]
    Katana(KatanaArgs),
    // #[command(about = "Madara deployment.")]
    // Madara(Madara),
    #[command(about = "Torii deployment.")]
    Torii(ToriiArgs),
}

#[derive(Clone, Debug, ValueEnum, serde::Serialize)]
pub enum Service {
    Katana,
    Torii,
}
