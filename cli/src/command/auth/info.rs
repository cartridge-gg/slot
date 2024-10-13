use anyhow::Result;
use clap::Args;
use slot::api;
use slot::api::Client;

#[derive(Debug, Args)]
pub struct InfoArgs;

impl InfoArgs {
    pub async fn run(&self) -> Result<()> {
        let client = Client::new_with_stored_credential()?;
        let client = api::Auth::new(&client);

        let info = client.info().await?;
        print!("{info:?}");

        Ok(())
    }
}
