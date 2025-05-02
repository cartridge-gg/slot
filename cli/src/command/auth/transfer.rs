use anyhow::Result;
use clap::Args;
use slot::graphql::auth::{transfer::*, Transfer};
use slot::graphql::GraphQLQuery;
use slot::{api::Client, credential::Credentials};

#[derive(Debug, Args)]
#[command(next_help_heading = "Transfer")]
pub struct TransferArgs {
    #[arg(help = "The team name to transfer funds to.", value_name = "team")]
    pub team: String,

    #[arg(help = "The amount to transfer.", value_name = "amount")]
    pub amount: i64,
}

impl TransferArgs {
    pub async fn run(&self) -> Result<()> {
        let credentials = Credentials::load()?;
        let client = Client::new_with_token(credentials.access_token);

        let request_body = Transfer::build_query(Variables {
            transfer: TransferInput {
                amount: self.amount,
                team: self.team.clone(),
            },
        });
        let res: ResponseData = client.query(&request_body).await?;

        println!("Transferred ${} to {}", self.amount, self.team);
        println!(
            "User balance: ${} -> ${}",
            res.transfer.account_before, res.transfer.account_after
        );
        println!(
            "Team balance: ${} -> ${}",
            res.transfer.team_before, res.transfer.team_after
        );

        Ok(())
    }
}
