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
    graphql::auth::{
        me::{ResponseData, Variables},
        Me,
    },
    server::LocalServer,
    vars,
};
use tokio::sync::mpsc::{self, Sender};

#[derive(Debug, Args)]
pub struct LoginArgs;

impl LoginArgs {
    pub async fn run(&self) -> Result<()> {
        let (auth_tx, auth_rx) = mpsc::channel::<AuthResult>(1);

        let server = Self::callback_server(auth_tx.clone())?;
        let port = server.local_addr()?.port();
        let callback_uri = format!("http://localhost:{port}/callback");

        let keychain_url = vars::get_cartridge_keychain_url();
        let url = format!("{keychain_url}/slot?callback_uri={callback_uri}");

        // Try to open the browser automatically
        let _ = browser::open(&url);

        println!("Complete flow in the browser or enter authorization code:\n");

        // Run both authentication methods concurrently
        self.run_concurrent_auth(server, auth_tx, auth_rx).await
    }

    async fn run_concurrent_auth(
        &self,
        server: LocalServer,
        auth_tx: mpsc::Sender<AuthResult>,
        mut auth_rx: mpsc::Receiver<AuthResult>,
    ) -> Result<()> {
        // Start the callback server (no shutdown signal needed since we'll exit)
        let _server_task = tokio::spawn(async move { server.start().await });

        // Start manual input task (no cancellation needed since we'll exit)
        let _manual_input_task = tokio::spawn(Self::manual_input_loop(auth_tx));

        // Wait for either authentication method to complete
        let result = auth_rx.recv().await;

        match result {
            Some(AuthResult::Success(code)) => self.complete_authentication(&code).await,
            Some(AuthResult::Failure(err)) => {
                eprintln!("Authentication failed: {}", err);
                std::process::exit(1);
            }
            None => {
                eprintln!("Authentication channel closed unexpectedly");
                std::process::exit(1);
            }
        }
    }

    async fn manual_input_loop(auth_tx: mpsc::Sender<AuthResult>) {
        let stdin = tokio::io::stdin();
        let mut reader = tokio::io::BufReader::new(stdin);

        loop {
            let mut input = String::new();
            use tokio::io::AsyncBufReadExt;

            match reader.read_line(&mut input).await {
                Ok(_) => {
                    let code = input.trim();
                    if !code.is_empty() {
                        let _ = auth_tx.send(AuthResult::Success(code.to_string())).await;
                        break;
                    }
                }
                Err(e) => {
                    let _ = auth_tx.send(AuthResult::Failure(e.to_string())).await;
                    break;
                }
            }
        }
    }

    async fn complete_authentication(&self, code: &str) -> Result<()> {
        // Exchange code for token
        let mut api = Client::new();
        let token = match api.oauth2(code).await {
            Ok(token) => token,
            Err(e) => {
                eprintln!("\nAuthentication failed: {}", e);
                eprintln!("\nPlease ensure you've copied the complete code from the browser");
                std::process::exit(1);
            }
        };
        api.set_token(token.clone());

        // Fetch account information
        let request_body = Me::build_query(Variables {});
        let data: ResponseData = match api.query(&request_body).await {
            Ok(data) => data,
            Err(e) => {
                eprintln!("\nFailed to fetch account information: {}", e);
                std::process::exit(1);
            }
        };

        let account = data.me.expect("missing payload");
        let account = AccountInfo::from(account);

        // Store credentials
        if let Err(e) = Credentials::new(account, token).store() {
            eprintln!("\nFailed to store credentials: {}", e);
            std::process::exit(1);
        }

        println!("\nYou are now logged in!");

        // Force exit after successful authentication
        // This is necessary because stdin keeps the process alive
        std::process::exit(0);
    }

    fn callback_server(auth_tx: mpsc::Sender<AuthResult>) -> Result<LocalServer> {
        let shared_state = Arc::new(AppState::new(auth_tx));

        let router = Router::new()
            .route("/callback", get(handler))
            .with_state(shared_state);

        Ok(LocalServer::new(router)?)
    }
}

enum AuthResult {
    Success(String),
    Failure(String),
}

#[derive(Debug, Deserialize)]
struct CallbackPayload {
    code: Option<String>,
}

#[derive(Clone)]
struct AppState {
    auth_tx: Sender<AuthResult>,
}

impl AppState {
    fn new(auth_tx: Sender<AuthResult>) -> Self {
        Self { auth_tx }
    }

    async fn send_auth_result(&self, result: AuthResult) -> Result<()> {
        self.auth_tx.send(result).await?;
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
    let redirect_url = match payload.code {
        Some(code) => {
            // Send the authentication code through the channel
            state.send_auth_result(AuthResult::Success(code)).await?;

            format!("{}/success", vars::get_cartridge_keychain_url())
        }
        None => {
            error!("User denied consent. Try again.");

            state
                .send_auth_result(AuthResult::Failure("User denied consent".to_string()))
                .await?;

            format!("{}/failure", vars::get_cartridge_keychain_url())
        }
    };

    Ok(Redirect::permanent(&redirect_url))
}
