use anyhow::Result;
use clap::Subcommand;

use self::create::CreateArgs;

mod configs;
mod create;

#[derive(Subcommand, Debug)]
pub enum Deployment {
    #[command(about = "Create a new deployment.")]
    Create(CreateArgs),
}

impl Deployment {
    pub async fn run(&self) -> Result<()> {
        match &self {
            Deployment::Create(args) => {
                args.run().await?;
                Ok(())
            }
        }
    }
}
