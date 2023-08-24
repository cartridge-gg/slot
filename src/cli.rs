use crate::command::Command;
use clap::Parser;

/// Slot CLI for Cartridge
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}
