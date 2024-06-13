use anyhow::Result;
use clap::Subcommand;

use self::{info::InfoArgs, login::LoginArgs};

mod info;
mod login;

#[derive(Subcommand, Debug)]
pub enum Auth {
    #[command(about = "Login to your Cartridge account.")]
    Login(LoginArgs),
    #[command(about = "Display info about the authenticated user.")]
    Info(InfoArgs),
}

impl Auth {
    pub async fn run(&self) -> Result<()> {
        match &self {
            Auth::Login(args) => args.run(),
            Auth::Info(args) => args.run().await,
        }
    }
}
