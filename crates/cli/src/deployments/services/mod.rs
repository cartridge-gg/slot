use clap::{Subcommand, ValueEnum};

use self::{
    katana::{KatanaAccountArgs, KatanaCreateArgs, KatanaUpdateArgs},
    torii::{ToriiCreateArgs, ToriiUpdateArgs},
};

mod katana;
mod torii;

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum CreateServiceCommands {
    #[command(about = "Katana deployment.")]
    Katana(KatanaCreateArgs),
    #[command(about = "Torii deployment.")]
    Torii(Box<ToriiCreateArgs>),
}

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum UpdateServiceCommands {
    #[command(about = "Katana deployment.")]
    Katana(KatanaUpdateArgs),
    #[command(about = "Torii deployment.")]
    Torii(Box<ToriiUpdateArgs>),
}

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum KatanaAccountCommands {
    #[command(about = "Katana deployment.")]
    Katana(KatanaAccountArgs),
}

#[derive(Clone, Debug, ValueEnum, serde::Serialize)]
pub enum Service {
    Katana,
    Torii,
}

impl std::fmt::Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Service::Katana => write!(f, "katana"),
            Service::Torii => write!(f, "torii"),
        }
    }
}
