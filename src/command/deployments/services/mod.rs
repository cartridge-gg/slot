use clap::{Subcommand, ValueEnum};

use self::{
    katana::{KatanaCreateArgs, KatanaUpdateArgs},
    torii::{ToriiCreateArgs, ToriiUpdateArgs},
};

mod katana;
mod madara;
mod torii;

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum CreateServiceCommands {
    #[command(about = "Katana deployment.")]
    Katana(KatanaCreateArgs),
    #[command(about = "Torii deployment.")]
    Torii(ToriiCreateArgs),
}

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum UpdateServiceCommands {
    #[command(about = "Katana deployment.")]
    Katana(KatanaUpdateArgs),
    #[command(about = "Torii deployment.")]
    Torii(ToriiUpdateArgs),
}

#[derive(Clone, Debug, ValueEnum, serde::Serialize)]
pub enum Service {
    Katana,
    Torii,
}
