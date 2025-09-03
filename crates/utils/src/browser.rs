use anyhow::{Context, Result};
use tracing::trace;

pub fn open(url: &str) -> Result<()> {
    trace!(%url, "Opening browser.");
    webbrowser::open(url).context("Failed to open web browser")?;
    println!("Your browser has been opened to visit: \n\n    {url}\n");
    Ok(())
}
