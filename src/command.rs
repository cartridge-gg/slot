pub mod auth;

use auth::Auth;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(subcommand)]
    Auth(Auth),
}

impl Command {
    pub fn handle(&self) -> eyre::Result<()> {
        match &self {
            Command::Auth(cmd) => {
                cmd.handle()?;

                Ok(())
            }
        }
    }
}
