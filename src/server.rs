use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use log::error;
use serde::Deserialize;
use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{constant, credential::Credentials};

pub struct LocalServer {
    router: Router,
    shutdown_rx: Receiver<()>,
    listener: TcpListener,
}

impl<'a> LocalServer {
    pub fn new() -> Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);
        // Port number of 0 requests OS to find an available port.
        let listener = TcpListener::bind("localhost:0")?;

        let shared_state = Arc::new(AppState::new(tx));
        let router = Router::new()
            .route("/callback", get(Self::callback))
            .with_state(shared_state);

        Ok(Self {
            router,
            shutdown_rx: rx,
            listener,
        })
    }

    pub fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.listener.local_addr()
    }

    pub async fn start(mut self) -> Result<()> {
        axum::Server::from_tcp(self.listener)?
            .serve(self.router.into_make_service())
            .with_graceful_shutdown(async {
                let _ = &self.shutdown_rx.recv().await;
            })
            .await?;

        Ok(())
    }

    async fn callback(
        State(state): State<Arc<AppState>>,
        Query(payload): Query<CallbackPayload>,
    ) -> Result<Redirect, AppError> {
        // 1. Shutdown the server
        state.shutdown().await?;

        // 2. Get access token using the authorization code
        match payload.code {
            Some(code) => {
                let client = reqwest::Client::new();
                let response = client
                    .post(format!("{}oauth2/token", constant::CARTRIDGE_API_URL))
                    .form(&[("code", &code)])
                    .send()
                    .await?;

                let cred: Credentials = response.json().await?;

                // 3. Store the access token locally
                cred.write()?;

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
}

#[derive(Deserialize)]
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

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
