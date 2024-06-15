use std::net::{SocketAddr, TcpListener};
use std::task::{Context, Poll};
use std::{fmt, io};

use axum::Router;
use hyper::service::Service;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tower_http::cors::CorsLayer;
use tower_layer::Layer;

#[derive(Debug)]
pub struct LocalServer {
    router: Router,
    listener: TcpListener,
    shutdown_rx: Option<UnboundedReceiver<()>>,
}

impl LocalServer {
    pub fn new(router: Router) -> anyhow::Result<Self> {
        // Port number of 0 requests OS to find an available port.
        let listener = TcpListener::bind("localhost:0")?;

        let (tx, rx) = unbounded_channel::<()>();
        let router = router.layer(ShutdownLayer::new(tx));

        Ok(Self {
            router,
            listener,
            shutdown_rx: Some(rx),
        })
    }

    /// Add a CORS layer to the server.
    pub fn cors(mut self, cors: CorsLayer) -> Self {
        self.router = self.router.layer(cors);
        self
    }

    /// Disable immediately shutdown the server upon handling the first request.
    #[allow(dead_code)]
    pub fn no_immediate_shutdown(mut self) -> Self {
        self.shutdown_rx = None;
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

/// Layer for handling sending a shutdown signal to the server upon
/// receiving the callback request.
#[derive(Clone)]
struct ShutdownLayer {
    tx: UnboundedSender<()>,
}

impl ShutdownLayer {
    pub fn new(tx: UnboundedSender<()>) -> Self {
        Self { tx }
    }
}

impl<S> Layer<S> for ShutdownLayer {
    type Service = ShutdownService<S>;

    fn layer(&self, service: S) -> Self::Service {
        ShutdownService {
            tx: self.tx.clone(),
            service,
        }
    }
}

#[derive(Clone)]
pub struct ShutdownService<S> {
    tx: UnboundedSender<()>,
    service: S,
}

impl<S, Request> Service<Request> for ShutdownService<S>
where
    S: Service<Request>,
    Request: fmt::Debug,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        self.tx.send(()).expect("failed to send shutdown signal");
        self.service.call(request)
    }
}

#[cfg(test)]
mod tests {
    use crate::server::LocalServer;
    use axum::{routing::get, Router};

    #[tokio::test]
    async fn test_server_immediate_shutdown() {
        let router = Router::new().route("/callback", get(|| async { "Hello, World!" }));
        let server = LocalServer::new(router).unwrap();

        let port = server.local_addr().unwrap().port();
        let client = reqwest::Client::new();

        tokio::spawn(server.start());

        let url = format!("http://localhost:{port}/callback");
        // first request should succeed
        assert!(client.get(&url).send().await.is_ok());
        // second request should fail as server should've been shutdown after first request
        assert!(client.get(url).send().await.is_err())
    }
}
