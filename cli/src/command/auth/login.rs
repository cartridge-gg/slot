use std::{io::Write, sync::Arc};

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
use tokio::sync::mpsc::Sender;

#[derive(Debug, Args)]
pub struct LoginArgs;

impl LoginArgs {
    pub async fn run(&self) -> Result<()> {
        let server = Self::callback_server().expect("Failed to create a server");
        let port = server.local_addr()?.port();
        let callback_uri = format!("http://localhost:{port}/callback");

        let keychain_url = vars::get_cartridge_keychain_url();
        let url = format!("{keychain_url}/slot?callback_uri={callback_uri}");

        println!("To authenticate, please visit the following URL in your browser:");
        println!("\n{}\n", url);

        // Try to open the browser automatically
        match browser::open(&url) {
            Ok(_) => {
                println!("Opening browser for authentication...");
                println!("If the browser doesn't open or you're in a headless environment,");
                println!("you can also paste the authorization code manually when prompted.");
            }
            Err(_) => {
                println!("Could not open browser automatically.");
                println!("Please visit the URL above manually.");
            }
        }

        // Start server with timeout and manual code input fallback
        self.run_with_fallback(server).await
    }

    async fn run_with_fallback(&self, server: LocalServer) -> Result<()> {
        println!("\nWaiting for authentication...");
        println!("Press Enter at any time to manually input an authorization code instead.");

        // Run server and manual input concurrently
        let server_task = async move { server.start().await };
        let manual_input_task = async {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            Ok::<(), anyhow::Error>(())
        };

        tokio::select! {
            server_result = server_task => {
                match server_result {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        println!("\nBrowser authentication failed: {}", e);
                        println!("You can still manually enter your authorization code:");
                        self.manual_code_input().await
                    }
                }
            }
            manual_result = manual_input_task => {
                // User pressed Enter, switch to manual input
                match manual_result {
                    Ok(()) => {
                        println!("\nSwitching to manual code input...");
                        self.manual_code_input().await
                    }
                    Err(e) => Err(e)
                }
            }
        }
    }

    async fn manual_code_input(&self) -> Result<()> {
        println!("\nIf you have completed authentication in your browser, you should see");
        println!("an authorization code. Please paste it below:");

        print!("\nEnter authorization code (or press Enter to cancel): ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let code = input.trim();

        if code.is_empty() {
            anyhow::bail!("Authentication cancelled");
        }

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

        println!("You are now logged in!\n");

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
