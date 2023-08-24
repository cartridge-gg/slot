use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct LocalServer {
    router: Router,
    shutdown_rx: Receiver<()>,
}

impl<'a> LocalServer {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);

        let shared_state = Arc::new(AppState::new(tx));
        let router = Router::new()
            .route("/callback", post(Self::callback))
            .with_state(shared_state);

        Self {
            router,
            shutdown_rx: rx,
        }
    }

    pub async fn start(mut self, host: &'a str) -> eyre::Result<()> {
        axum::Server::bind(&host.parse()?)
            .serve(self.router.into_make_service())
            .with_graceful_shutdown(async {
                let _ = &self.shutdown_rx.recv().await;
            })
            .await?;

        Ok(())
    }

    async fn callback(
        State(state): State<Arc<AppState>>,
        Query(params): Query<HashMap<String, String>>,
    ) -> Result<Json<Value>, AppError> {
        let auth_code = &params["code"];
        println!("{auth_code}");

        // TODO: get access token using the authorization code

        state.shutdown().await?;

        Ok(Json(json!({ "success": true })))
    }
}

#[derive(Clone)]
struct AppState {
    shutdown_tx: Sender<()>,
}

impl AppState {
    fn new(shutdown_tx: Sender<()>) -> Self {
        Self { shutdown_tx }
    }

    async fn shutdown(&self) -> eyre::Result<()> {
        self.shutdown_tx.send(()).await?;

        Ok(())
    }
}

struct AppError(eyre::Error);

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
    E: Into<eyre::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
