use std::io;
use std::net::{SocketAddr, TcpListener};

use axum::Router;
use tokio::sync::mpsc::Receiver;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// A simple local server.
#[derive(Debug)]
pub struct LocalServer {
    router: Router,
    listener: TcpListener,
    shutdown_rx: Option<Receiver<()>>,
}

impl LocalServer {
    pub fn new(router: Router) -> anyhow::Result<Self> {
        // Port number of 0 requests OS to find an available port.
        let listener = TcpListener::bind("localhost:0")?;
        // To view the logs emitted by the server, set `RUST_LOG=tower_http=trace`
        let router = router.layer(TraceLayer::new_for_http());

        Ok(Self {
            router,
            listener,
            shutdown_rx: None,
        })
    }

    /// Add a CORS layer to the server.
    pub fn cors(mut self, cors: CorsLayer) -> Self {
        self.router = self.router.layer(cors);
        self
    }

    /// Disable immediately shutdown the server upon handling the first request.
    pub fn with_shutdown_signal(mut self, receiver: Receiver<()>) -> Self {
        self.shutdown_rx = Some(receiver);
        self
    }

    pub fn local_addr(&self) -> Result<SocketAddr, io::Error> {
        self.listener.local_addr()
    }

    pub async fn start(mut self) -> anyhow::Result<()> {
        let addr = self.listener.local_addr()?;
        tracing::info!(?addr, "Callback server started");

        let server = axum::Server::from_tcp(self.listener)?.serve(self.router.into_make_service());
        if let Some(mut rx) = self.shutdown_rx.take() {
            server
                .with_graceful_shutdown(async { rx.recv().await.expect("channel closed") })
                .await?;
        } else {
            server.await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::server::LocalServer;
    use axum::{routing::get, Router};

    #[tokio::test]
    async fn test_server_graceful_shutdown() {
        let (tx, rx) = tokio::sync::mpsc::channel(1);

        let router = Router::new().route("/callback", get(|| async { "Hello, World!" }));
        let server = LocalServer::new(router).unwrap().with_shutdown_signal(rx);
        let port = server.local_addr().unwrap().port();

        let client = reqwest::Client::new();
        let url = format!("http://localhost:{port}/callback");

        // start the local server
        tokio::spawn(server.start());

        // first request should succeed
        assert!(client.get(&url).send().await.is_ok());

        // send shutdown signal
        tx.send(()).await.unwrap();

        // sending request after sending the shutdown signal should fail as server
        // should've been shutdown
        assert!(client.get(url).send().await.is_err())
    }
}
