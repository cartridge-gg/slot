mod browser;
mod cli;
mod command;
mod server;

use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command.run().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{e}")
        }
    }
}
