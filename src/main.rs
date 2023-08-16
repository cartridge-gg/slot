use clap::Parser;

/// Slot CLI for Cartridge
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

#[tokio::main]
async fn main() {
    let _args = Args::parse();
}
