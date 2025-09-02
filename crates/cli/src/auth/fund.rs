use anyhow::Result;
use clap::Args;
use slot_utils::{browser, vars};

#[derive(Debug, Args)]
pub struct FundArgs;

impl FundArgs {
    pub async fn run(&self) -> Result<()> {
        let url = vars::get_cartridge_keychain_url();

        let url = format!("{url}/slot/fund");

        browser::open(&url)?;

        Ok(())
    }
}
