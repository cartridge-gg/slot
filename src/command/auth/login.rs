use crate::{browser::Browser, server::LocalServer};
use anyhow::Result;
use clap::Args;
use tokio::runtime::Runtime;

#[derive(Debug, Args)]
pub struct LoginArgs {}

impl LoginArgs {
    pub fn run(&self) -> Result<()> {
        let rt = Runtime::new()?;

        let handler = std::thread::spawn(move || {
            let server = LocalServer::new().expect("Failed to start a server");
            let addr = server.local_addr().unwrap();

            let res = rt.block_on(async { tokio::join!(server.start(), Browser::open(&addr)) });

            match res {
                (Err(e), _) => {
                    eprintln!("Server error: {e}");
                }
                (_, Err(e)) => {
                    eprintln!("Browser error: {e}");
                }
                _ => {
                    // println!("Success");
                }
            }
        });

        handler.join().unwrap();

        Ok(())
    }
}
