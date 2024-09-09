use clap::{Subcommand, ValueEnum};

use self::{
    katana::{KatanaAccountArgs, KatanaCreateArgs, KatanaForkArgs, KatanaUpdateArgs},
    madara::MadaraCreateArgs,
    saya::{SayaCreateArgs, SayaUpdateArgs},
    torii::{ToriiCreateArgs, ToriiUpdateArgs},
};

mod katana;
mod madara;
mod saya;
mod torii;

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum CreateServiceCommands {
    #[command(about = "Katana deployment.")]
    Katana(KatanaCreateArgs),
    #[command(about = "Torii deployment.")]
    Torii(ToriiCreateArgs),
    #[command(about = "Madara deployment.")]
    Madara(MadaraCreateArgs),
    #[command(about = "Saya deployment.")]
    Saya(SayaCreateArgs),
}

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum UpdateServiceCommands {
    #[command(about = "Katana deployment.")]
    Katana(KatanaUpdateArgs),
    #[command(about = "Torii deployment.")]
    Torii(ToriiUpdateArgs),
    #[command(about = "Saya deployment.")]
    Saya(SayaUpdateArgs),
}

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum ForkServiceCommands {
    #[command(about = "Katana deployment.")]
    Katana(KatanaForkArgs),
    // #[command(about = "Torii deployment.")]
    // Torii(ToriiUpdateArgs),
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
    Madara,
    Saya,
}
