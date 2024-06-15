use anyhow::Result;
use axum::{
    extract::Query,
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
    api::Client,
    browser, constant,
    credential::Credentials,
    graphql::auth::{
        me::{ResponseData, Variables},
        Me,
    },
    server::LocalServer,
};

#[derive(Debug, Args)]
pub struct LoginArgs;

impl LoginArgs {
    pub async fn run(&self) -> Result<()> {
        let server = Self::callback_server().expect("Failed to create a server");
        let port = server.local_addr()?.port();
        let callback_uri = format!("http://localhost:{port}/callback");

        let url = format!("https://x.cartridge.gg/slot/auth?callback_uri={callback_uri}");

        browser::open(&url)?;
        server.start().await?;

        Ok(())
    }

    fn callback_server() -> Result<LocalServer> {
        let router = Router::new().route("/callback", get(handler));
        LocalServer::new(router)
    }
}

#[derive(Debug, Deserialize)]
struct CallbackPayload {
    code: Option<String>,
}

#[derive(Debug, thiserror::Error)]
enum CallbackError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Api error: {0}")]
    Api(#[from] slot::api::Error),
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

async fn handler(Query(payload): Query<CallbackPayload>) -> Result<Redirect, CallbackError> {
    // 1. Get access token using the authorization code
    match payload.code {
        Some(code) => {
            let mut api = Client::new();

            let token = api.oauth2(&code).await?;
            api.set_token(token.clone());

            // fetch the account information
            let request_body = Me::build_query(Variables {});
            let res: graphql_client::Response<ResponseData> = api.query(&request_body).await?;

            // display the errors if any, but still process bcs we have the token
            if let Some(errors) = res.errors {
                for err in errors {
                    eprintln!("Error: {}", err.message);
                }
            }

            let account_info = res.data.map(|data| data.me.expect("should exist"));

            // 2. Store the access token locally
            Credentials::new(account_info, token).write()?;

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
