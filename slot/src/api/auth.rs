use graphql_client::GraphQLQuery;
use log::error;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use hyper::StatusCode;
use serde::Deserialize;
use tokio::sync::mpsc::Sender;

use crate::{
    account::AccountInfo,
    api::Client,
    browser,
    credential::Credentials,
    graphql::auth::{
        me::{ResponseData, Variables},
        Me,
    },
    graphql::{self},
    server::LocalServer,
    vars, Error,
};

/// Client for the auth API.
pub struct Auth<'c> {
    client: &'c Client,
}

impl<'c> Auth<'c> {
    /// Creates a new [`AuthApi`] with the given [`Client`].
    pub fn new(client: &'c Client) -> Self {
        Self { client }
    }

    /// Logs in the user.
    ///
    /// This will prompt user the Controller login page in the system's default browser.
    /// The access token generated from the login will be automatically stored in the system's
    /// storage.
    pub async fn login(&self) -> Result<(), Error> {
        let server = callback_server().context("Failed to create a server")?;

        let port = server.local_addr()?.port();
        let callback_uri = format!("http://localhost:{port}/callback");

        let url = vars::get_cartridge_keychain_url();

        let url = format!("{url}/slot?callback_uri={callback_uri}");

        browser::open(&url)?;
        server.start().await?;

        Ok(())
    }

    /// Returns the account info of the currently authenticated user.
    pub async fn info(&self) -> Result<AccountInfo, Error> {
        let req = graphql::auth::Me::build_query(Variables {});
        let res: ResponseData = self.client.query(&req).await?;
        Ok(AccountInfo::from(res.me.expect("non-null response data")))
    }
}

fn callback_server() -> Result<LocalServer> {
    let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);
    let shared_state = Arc::new(AppState::new(tx));

    let router = Router::new()
        .route("/callback", get(handler))
        .with_state(shared_state);

    Ok(LocalServer::new(router)?.with_shutdown_signal(rx))
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
    Slot(#[from] Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
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
