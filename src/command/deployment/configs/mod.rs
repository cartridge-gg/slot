use clap::Subcommand;

use self::{katana::Katana, torii::Torii};

mod katana;
mod madara;
mod torii;

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum CreateCommands {
    #[command(about = "Katana deployment.")]
    Katana(Katana),
    // #[command(about = "Madara deployment.")]
    // Madara(Madara),
    #[command(about = "Torii deployment.")]
    Torii(Torii),
}
