use anyhow::Result;
use clap::Args;
use slot::graphql::auth::{update_me::*, UpdateMe};
use slot::graphql::GraphQLQuery;
use slot::{api::Client, credential::Credentials};

#[derive(Debug, Args)]
#[command(next_help_heading = "Set email options")]
pub struct EmailArgs {
    #[arg(help = "The email address of the user.")]
    pub email: String,
}

impl EmailArgs {
    pub async fn run(&self) -> Result<()> {
        let credentials = Credentials::load()?;
        let client = Client::new_with_token(credentials.access_token);

        let request_body = UpdateMe::build_query(Variables {
            email: Some(self.email.clone()),
            slot_billing: None,
        });
        let res: ResponseData = client.query(&request_body).await?;
        print!("{:?}", res);

        Ok(())
    }
}
