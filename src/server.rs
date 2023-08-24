use axum::{extract::Query, routing::post, Json, Router};
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct Server {
    router: Router,
}

impl<'a> Server {
    pub fn new() -> Self {
        Self {
            router: Router::new().route("/callback", post(Self::callback)),
        }
    }

    pub async fn start(self, host: &'a str) -> eyre::Result<()> {
        axum::Server::bind(&host.parse()?)
            .serve(self.router.into_make_service())
            .await?;

        Ok(())
    }

    async fn callback(Query(params): Query<HashMap<String, String>>) -> Json<Value> {
        let auth_code = &params["code"];
        println!("{auth_code}");

        // TODO: get access token using the authorization code

        Json(json!({ "success": true }))
    }
}
