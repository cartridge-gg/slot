#![cfg_attr(not(test), warn(unused_crate_dependencies))]

mod command;

use crate::command::Command;
use clap::Parser;
use colored::*;
use update_informer::{registry, Check};

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

    let name = "cartridge-gg/slot";
    let current = env!("CARGO_PKG_VERSION");
    let informer = update_informer::new(registry::GitHub, name, current);

    match &cli.command.run().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }

    if let Some(version) = informer.check_version().ok().flatten() {
        notify_new_version(current, version.to_string().as_str());
    }
}

fn notify_new_version(current_version: &str, latest_version: &str) {
    println!(
        "\n{} {}{} → {}",
        "Slot CLI update available:".bold(),
        "v".red().bold(),
        current_version.red().bold(),
        latest_version.green().bold()
    );
    println!("To upgrade, run: {}", "`slotup`".cyan().bold());
    println!("\n")
}
