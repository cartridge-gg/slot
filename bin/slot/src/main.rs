#![cfg_attr(not(test), warn(unused_crate_dependencies))]
use num_bigint as _;
use update_informer as _;

use clap::Parser;
use slot_cli::Command;

/// Slot CLI for Cartridge
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command.run().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
