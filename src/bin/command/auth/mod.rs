use anyhow::Result;
use clap::Subcommand;

use self::{info::InfoArgs, login::LoginArgs};

mod info;
mod login;
mod session;

#[derive(Subcommand, Debug)]
pub enum Auth {
    #[command(about = "Login to your Cartridge account.")]
    Login(LoginArgs),
    #[command(about = "Display info about the authenticated user.")]
    Info(InfoArgs),
    // Mostly for testing purposes, will eventually turn it into a library call from `sozo`.
    #[command(hide = true)]
    CreateSession(session::CreateSession),
}

impl Auth {
    pub async fn run(&self) -> Result<()> {
        match &self {
            Auth::Login(args) => args.run().await,
            Auth::Info(args) => args.run().await,
            Auth::CreateSession(args) => args.run().await,
        }
    }
}
