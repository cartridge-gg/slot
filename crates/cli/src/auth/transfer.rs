use anyhow::Result;
use clap::Args;
use slot_core::credentials::Credentials;
use slot_graphql::api::Client;
use slot_graphql::auth::{transfer::*, Transfer};
use slot_graphql::GraphQLQuery;

#[derive(Debug, Args)]
#[command(next_help_heading = "Transfer")]
pub struct TransferArgs {
    #[arg(help = "The team name to transfer funds to.", value_name = "team")]
    pub team: String,

    #[arg(long, help = "The USD amount to transfer", value_name = "USD")]
    pub usd: Option<i64>,

    #[arg(long, help = "The credits amount to transfer", value_name = "CREDITS")]
    pub credits: Option<i64>,
}

pub fn get_amount(usd: Option<i64>, credits: Option<i64>) -> Result<i64> {
    match (usd, credits) {
        (Some(usd_amount), None) => Ok(usd_amount * 100),
        (None, Some(credits_amount)) => Ok(credits_amount),
        (None, None) => Err(anyhow::anyhow!(
            "Either --usd or --credits must be specified"
        )),
        (Some(_), Some(_)) => Err(anyhow::anyhow!("Cannot specify both --usd and --credits")),
    }
}

impl TransferArgs {
    pub async fn run(&self) -> Result<()> {
        let credentials = Credentials::load()?;
        let client = Client::new_with_token(credentials.access_token);

        let amount = get_amount(self.usd, self.credits)?;

        let request_body = Transfer::build_query(Variables {
            transfer: TransferInput {
                amount,
                team: self.team.clone(),
            },
        });
        let res: ResponseData = client.query(&request_body).await?;

        // Display the appropriate amount in the message
        let amount_display = if let Some(usd) = self.usd {
            format!("${} USD", usd)
        } else if let Some(credits) = self.credits {
            format!("{} credits", credits)
        } else {
            // This shouldn't happen due to the error check in get_amount
            "amount".to_string()
        };

        println!("Transferred {} to {}", amount_display, self.team);
        println!(
            "User balance: {} credits (~${} USD) -> {} credits (~${} USD)",
            res.transfer.account_before,
            res.transfer.account_before / 100,
            res.transfer.account_after,
            res.transfer.account_after / 100
        );
        println!(
            "Team balance: {} credits (~${} USD) -> {} credits (~${} USD)",
            res.transfer.team_before,
            res.transfer.team_before / 100,
            res.transfer.team_after,
            res.transfer.team_after / 100
        );

        Ok(())
    }
}
