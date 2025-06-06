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
    account::AccountInfo,
    api::Client,
    browser,
    credential::Credentials,
    eula,
    graphql::auth::{
        me::{ResponseData, Variables},
        Me,
    },
    server::LocalServer,
    vars,
};
use tokio::sync::mpsc::Sender;

#[derive(Debug, Args)]
pub struct LoginArgs;

impl LoginArgs {
    pub async fn run(&self) -> Result<()> {
        let server = Self::callback_server().expect("Failed to create a server");
        let port = server.local_addr()?.port();
        let callback_uri = format!("http://localhost:{port}/callback");

        let url = vars::get_cartridge_keychain_url();

        let url = format!("{url}/slot?callback_uri={callback_uri}");

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

            // 4. Check and prompt for EULA acceptance
            if !eula::has_accepted_current_eula()? {
                println!("Before using the Slot CLI, you must accept the End User License Agreement.\n");
                
                if !eula::display_and_accept_eula()? {
                    error!("EULA not accepted. You must accept the EULA to use the Slot CLI.");
                    return Ok(Redirect::permanent(&format!(
                        "{}/failure",
                        vars::get_cartridge_keychain_url()
                    )));
                }
            }

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
