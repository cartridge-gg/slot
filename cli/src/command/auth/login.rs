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
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);

        let server = Self::callback_server(auth_tx.clone(), shutdown_tx.clone())?;
        let port = server.local_addr()?.port();
        let callback_uri = format!("http://localhost:{port}/callback");

        let keychain_url = vars::get_cartridge_keychain_url();
        let url = format!("{keychain_url}/slot?callback_uri={callback_uri}");

        println!("To authenticate, please visit the following URL in your browser:");
        println!("\n\t{}\n", url);

        // Try to open the browser automatically
        let _ = browser::open(&url);

        println!("Complete flow in the browser or enter authorization code: ");

        // Run both authentication methods concurrently
        self.run_concurrent_auth(server, auth_tx, auth_rx, shutdown_tx, shutdown_rx)
            .await
    }

    async fn run_concurrent_auth(
        &self,
        server: LocalServer,
        auth_tx: mpsc::Sender<AuthResult>,
        mut auth_rx: mpsc::Receiver<AuthResult>,
        shutdown_tx: mpsc::Sender<()>,
        shutdown_rx: mpsc::Receiver<()>,
    ) -> Result<()> {
        // Start the callback server
        let server_task =
            tokio::spawn(async move { server.with_shutdown_signal(shutdown_rx).start().await });

        // Create a channel to signal manual input to stop
        let (cancel_tx, cancel_rx) = mpsc::channel::<()>(1);

        // Start manual input task
        let manual_input_task = tokio::spawn(Self::manual_input_loop(auth_tx, cancel_rx));

        // Wait for either authentication method to complete
        let result = auth_rx.recv().await;

        // Send cancellation signal to manual input task
        let _ = cancel_tx.send(()).await;

        // Send shutdown signal to server
        let _ = shutdown_tx.send(()).await;

        // Wait a bit for tasks to clean up gracefully
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Clean up tasks
        server_task.abort();
        manual_input_task.abort();

        match result {
            Some(AuthResult::Success(code)) => self.complete_authentication(&code).await,
            Some(AuthResult::Failure(err)) => {
                Err(anyhow::anyhow!("Authentication failed: {}", err))
            }
            None => Err(anyhow::anyhow!(
                "Authentication channel closed unexpectedly"
            )),
        }
    }

    async fn manual_input_loop(
        auth_tx: mpsc::Sender<AuthResult>,
        mut cancel_rx: mpsc::Receiver<()>,
    ) {
        let stdin = tokio::io::stdin();
        let mut reader = tokio::io::BufReader::new(stdin);

        loop {
            let mut input = String::new();
            use tokio::io::AsyncBufReadExt;

            tokio::select! {
                // Listen for cancellation signal
                _ = cancel_rx.recv() => {
                    break;
                }
                // Read from stdin
                result = reader.read_line(&mut input) => {
                    match result {
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
        }
    }

    async fn complete_authentication(&self, code: &str) -> Result<()> {
        // Exchange code for token
        let mut api = Client::new();
        let token = api.oauth2(code).await?;
        api.set_token(token.clone());

        // Fetch account information
        let request_body = Me::build_query(Variables {});
        let data: ResponseData = api.query(&request_body).await?;

        let account = data.me.expect("missing payload");
        let account = AccountInfo::from(account);

        // Store credentials
        Credentials::new(account, token).store()?;

        println!("\nYou are now logged in!");

        // Force exit after successful authentication
        // This is necessary because stdin keeps the process alive
        std::process::exit(0);
    }

    fn callback_server(
        auth_tx: mpsc::Sender<AuthResult>,
        shutdown_tx: mpsc::Sender<()>,
    ) -> Result<LocalServer> {
        let shared_state = Arc::new(AppState::new(auth_tx, shutdown_tx));

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
    shutdown_tx: Sender<()>,
}

impl AppState {
    fn new(auth_tx: Sender<AuthResult>, shutdown_tx: Sender<()>) -> Self {
        Self {
            auth_tx,
            shutdown_tx,
        }
    }

    async fn send_auth_result(&self, result: AuthResult) -> Result<()> {
        self.auth_tx.send(result).await?;
        Ok(())
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
    let redirect_url = match payload.code {
        Some(code) => {
            // Send the authentication code through the channel
            state.send_auth_result(AuthResult::Success(code)).await?;

            // Shutdown the server
            state.shutdown().await?;

            format!("{}/success", vars::get_cartridge_keychain_url())
        }
        None => {
            error!("User denied consent. Try again.");

            state
                .send_auth_result(AuthResult::Failure("User denied consent".to_string()))
                .await?;
            state.shutdown().await?;

            format!("{}/failure", vars::get_cartridge_keychain_url())
        }
    };

    Ok(Redirect::permanent(&redirect_url))
}
