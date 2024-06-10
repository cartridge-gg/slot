mod api;
mod browser;
mod cli;
mod command;
mod constant;
mod credential;
mod server;

mod utils;

use clap::Parser;
use cli::Cli;
use log::error;

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command.run().await {
        Ok(_) => {}
        Err(e) => {
            error!("{e}")
        }
    }
}
