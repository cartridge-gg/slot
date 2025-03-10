use self::{email::EmailArgs, info::InfoArgs, login::LoginArgs};
use crate::command::auth::billing::BillingArgs;
use crate::command::auth::fund::FundArgs;
use crate::command::auth::transfer::TransferArgs;
use anyhow::Result;
use clap::Subcommand;

mod billing;
mod email;
mod fund;
mod info;
mod login;
mod session;
mod transfer;

#[derive(Subcommand, Debug)]
pub enum Auth {
    #[command(about = "Login to your Cartridge account.")]
    Login(LoginArgs),

    #[command(about = "Display info about the authenticated user.")]
    Info(InfoArgs),

    #[command(about = "Set the email address for the authenticated user.")]
    SetEmail(EmailArgs),

    #[command(about = "Manage slot billing for the authenticated user.")]
    EnableSlotBilling(BillingArgs),

    #[command(about = "Fund the authenticated user's account.")]
    Fund(FundArgs),

    #[command(about = "Transfer funds to a slot team.")]
    Transfer(TransferArgs),

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
            Auth::SetEmail(args) => args.run().await,
            Auth::EnableSlotBilling(args) => args.run().await,
            Auth::Fund(args) => args.run().await,
            Auth::Transfer(args) => args.run().await,
        }
    }
}
