use anyhow::Result;
use clap::Args;
use slot_core::credentials::Credentials;

#[derive(Debug, Args)]
pub struct TokenArgs;

impl TokenArgs {
    pub async fn run(&self) -> Result<()> {
        let credentials = Credentials::load()?;

        eprintln!();
        eprintln!(
            "WARNING: This token provides access to your account for slot actions. Keep it safe."
        );
        eprintln!();

        // Print the raw token
        println!("{}", credentials.access_token.token);

        // Print usage instructions to stderr so they don't interfere with token parsing
        eprintln!();
        eprintln!("You can use this token for programmatic authentication by setting:");
        eprintln!(
            "export SLOT_AUTH='{}'",
            serde_json::to_string(&credentials)?
        );
        eprintln!();
        eprintln!("This is useful for CI/CD pipelines and scripts where interactive authentication is not possible.");

        Ok(())
    }
}
