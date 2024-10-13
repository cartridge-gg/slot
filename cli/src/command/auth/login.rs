use anyhow::Result;
use clap::Args;
<<<<<<< Updated upstream
use graphql_client::GraphQLQuery;
use hyper::StatusCode;
use log::error;
use serde::Deserialize;
use slot::{
    account::AccountInfo,
    api::Client,
    browser,
    credential::Credentials,
    graphql::auth::{
        me::{ResponseData, Variables},
        Me,
    },
    server::LocalServer,
    vars,
};
use tokio::sync::mpsc::Sender;
=======
use slot::api::{self, Client};
>>>>>>> Stashed changes

#[derive(Debug, Args)]
pub struct LoginArgs;

impl LoginArgs {
    pub async fn run(&self) -> Result<()> {
        let client = Client::new();
        let _ = api::Auth::new(&client).login().await?;
        Ok(())
    }
}
<<<<<<< Updated upstream

#[derive(Debug, thiserror::Error)]
enum CallbackError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),

    #[error(transparent)]
    Slot(#[from] slot::Error),
}

impl IntoResponse for CallbackError {
    fn into_response(self) -> Response {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        let message = format!("Something went wrong: {self}");
        (status, message).into_response()
    }
}

async fn handler(
    State(state): State<Arc<AppState>>,
    Query(payload): Query<CallbackPayload>,
) -> Result<Redirect, CallbackError> {
    // 1. Shutdown the server
    state.shutdown().await?;

    // 2. Get access token using the authorization code
    match payload.code {
        Some(code) => {
            let mut api = Client::new();

            let token = api.oauth2(&code).await?;
            api.set_token(token.clone());

            // fetch the account information
            let request_body = Me::build_query(Variables {});
            let data: ResponseData = api.query(&request_body).await?;

            let account = data.me.expect("missing payload");
            let account = AccountInfo::from(account);

            // 3. Store the access token locally
            Credentials::new(account, token).store()?;

            println!("You are now logged in!\n");

            Ok(Redirect::permanent(&format!(
                "{}/success",
                vars::get_cartridge_keychain_url()
            )))
        }
        None => {
            error!("User denied consent. Try again.");

            Ok(Redirect::permanent(&format!(
                "{}/failure",
                vars::get_cartridge_keychain_url()
            )))
        }
    }
}
=======
>>>>>>> Stashed changes
