use anyhow::Result;
use tracing::trace;

pub fn open(url: &str) -> Result<()> {
    trace!(%url, "Opening browser.");
    match webbrowser::open(url) {
        Ok(_) => {
            println!("Your browser has been opened to visit: \n\n    {url}\n");
            Ok(())
        }
        Err(_) => {
            println!("Failed to open web browser automatically.");
            println!("Please open this URL in your browser:\n\n    {url}\n");
            Ok(())
        }
    }
}
