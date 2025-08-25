use anyhow::{Context, Result};

pub fn open(url: &str) -> Result<()> {
    println!("To authenticate, please visit the following URL in your browser:");
    println!("\n\t{}\n", url);

    webbrowser::open(url).context("Failed to open web browser")
}
