use clap::{Subcommand, ValueEnum};

use self::{
    katana::{KatanaAccountArgs, KatanaCreateArgs, KatanaUpdateArgs},
    saya::{SayaCreateArgs, SayaUpdateArgs},
    torii::{ToriiCreateArgs, ToriiUpdateArgs},
};

mod katana;
mod saya;
mod torii;

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum CreateServiceCommands {
    #[command(about = "Katana deployment.")]
    Katana(KatanaCreateArgs),
    #[command(about = "Torii deployment.")]
    Torii(ToriiCreateArgs),
    #[command(about = "Saya deployment.")]
    Saya(SayaCreateArgs),
}

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum UpdateServiceCommands {
    #[command(about = "Katana deployment.")]
    Katana(KatanaUpdateArgs),
    #[command(about = "Torii deployment.")]
    Torii(Box<ToriiUpdateArgs>),
    #[command(about = "Saya deployment.")]
    Saya(SayaUpdateArgs),
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
    Saya,
}

impl std::fmt::Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Service::Katana => write!(f, "katana"),
            Service::Torii => write!(f, "torii"),
            Service::Saya => write!(f, "saya"),
        }
    }
}
