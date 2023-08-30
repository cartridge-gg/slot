use crate::{browser::Browser, server::LocalServer};
use clap::Subcommand;
use eyre;
use tokio::runtime::Runtime;

#[derive(Subcommand, Debug)]
pub enum Auth {
    Login,
}

impl Auth {
    pub fn handle(&self) -> eyre::Result<()> {
        match self {
            Auth::Login => {
                Self::login()?;
            }
        }

        Ok(())
    }

    fn login() -> eyre::Result<()> {
        let rt = Runtime::new()?;

        let handler = std::thread::spawn(move || {
            let server = LocalServer::new().expect("Failed to start a server");
            let addr = &server.local_addr().unwrap();

            let res = rt.block_on(async { tokio::join!(server.start(), Browser::open(addr)) });

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
