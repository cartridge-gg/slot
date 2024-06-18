use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use clap::Args;
use graphql_client::GraphQLQuery;
use hyper::StatusCode;
use log::error;
use serde::Deserialize;
use slot::{
    account::Account,
    api::Client,
    browser, constant,
    credential::Credentials,
    graphql::auth::{
        me::{ResponseData, Variables},
        AccountTryFromGraphQLError, Me,
    },
    server::LocalServer,
};
use tokio::sync::mpsc::Sender;

#[derive(Debug, Args)]
pub struct LoginArgs;

impl LoginArgs {
    pub async fn run(&self) -> Result<()> {
        let server = Self::callback_server().expect("Failed to create a server");
        let port = server.local_addr()?.port();
        let callback_uri = format!("http://localhost:{port}/callback");

        let url = format!("https://x.cartridge.gg/slot/auth?callback_uri={callback_uri}");

        browser::open(&url)?;
        server.start().await?;

        Ok(())
    }

    fn callback_server() -> Result<LocalServer> {
        let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);
        let shared_state = Arc::new(AppState::new(tx));

        let router = Router::new()
            .route("/callback", get(handler))
            .with_state(shared_state);

        Ok(LocalServer::new(router)?.with_shutdown_signal(rx))
    }
}

#[derive(Debug, Deserialize)]
struct CallbackPayload {
    code: Option<String>,
}

#[derive(Clone)]
struct AppState {
    shutdown_tx: Sender<()>,
}

impl AppState {
    fn new(shutdown_tx: Sender<()>) -> Self {
        Self { shutdown_tx }
    }

    async fn shutdown(&self) -> Result<()> {
        self.shutdown_tx.send(()).await?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
enum CallbackError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),

    #[error(transparent)]
    Slot(#[from] slot::Error),

    #[error(transparent)]
    Parse(#[from] AccountTryFromGraphQLError),
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
            let res: graphql_client::Response<ResponseData> = api.query(&request_body).await?;

            // display the errors if any, but still process bcs we have the token
            if let Some(errors) = res.errors {
                for err in errors {
                    eprintln!("Error: {}", err.message);
                }
            }

            let account = res.data.and_then(|data| data.me).expect("missing payload");
            let account = Account::try_from(account)?;

            // 3. Store the access token locally
            Credentials::new(account, token).store()?;

            println!("You are now logged in!\n");

            Ok(Redirect::permanent(&format!(
                "{}/slot/auth/success",
                constant::CARTRIDGE_KEYCHAIN_URL
            )))
        }
        None => {
            error!("User denied consent. Try again.");

            Ok(Redirect::permanent(&format!(
                "{}/slot/auth/failure",
                constant::CARTRIDGE_KEYCHAIN_URL
            )))
        }
    }
}
