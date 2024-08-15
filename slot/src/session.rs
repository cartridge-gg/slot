use std::path::Path;
use std::{fs, path::PathBuf};

use account_sdk::storage::SessionMetadata;
use anyhow::Context;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, routing::post, Json, Router};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use starknet::core::types::Felt;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tower_http::cors::CorsLayer;
use tracing::info;
use url::Url;

use crate::credential::Credentials;
use crate::error::Error;
use crate::utils::{self};
use crate::{browser, server::LocalServer, vars};

const SESSION_CREATION_PATH: &str = "/slot/session";
const SESSION_FILE_BASE_NAME: &str = "session.json";

/// A policy defines what action can be performed by the session key.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Policy {
    /// The target contract address.
    pub target: Felt,
    /// The method name.
    pub method: String,
}

/// Retrieves the session for the given chain id of the currently authenticated user.
/// Returns `None` if no session can be found for the chain id.
///
/// # Errors
///
/// This function will return an error if there is no authenticated user.
///
pub fn get(chain: Felt) -> Result<Option<SessionMetadata>, Error> {
    get_at(utils::config_dir(), chain)
}

/// Stores the session on-disk. Returns the path to the file where the `session` has been written to.
///
/// # Errors
///
/// This function will return an error if there is no authenticated user.
///
pub fn store(chain: Felt, session: &SessionMetadata) -> Result<PathBuf, Error> {
    store_at(utils::config_dir(), chain, session)
}

/// Creates a new session token for the given set of parameters for the currently authenticated user.
/// Returns the newly created session token.
///
/// # Arguments
///
/// * `rpc_url` - The RPC URL of the chain network that you want to create a session for.
/// * `policies` - The policies that the session token will have.
///
/// # Errors
///
/// This function will return an error if there is no authenticated user.
///
pub async fn create<U>(rpc_url: U, policies: &[Policy]) -> Result<SessionMetadata, Error>
where
    U: Into<Url>,
{
    let credentials = Credentials::load()?;
    let username = credentials.account.id;
    create_user_session(&username, rpc_url, policies).await
}

/// Get the session token of the chain id `chain` for the currently authenticated user. It will
/// use `config_dir` as the root path to look for the session file.
fn get_at<P>(config_dir: P, chain: Felt) -> Result<Option<SessionMetadata>, Error>
where
    P: AsRef<Path>,
{
    let credentials = Credentials::load_at(&config_dir)?;
    let username = credentials.account.id;

    let user_path = get_user_relative_file_path(&username, chain);
    let file_path = config_dir.as_ref().join(user_path);

    if file_path.exists() {
        let contents = fs::read_to_string(file_path)?;
        let session = serde_json::from_str(&contents)?;
        Ok(Some(session))
    } else {
        Ok(None)
    }
}

/// Stores the session token of the chain id `chain` for the currently authenticated user. It will
/// use `config_dir` as the root path to store the session file.
fn store_at<P>(config_dir: P, chain: Felt, session: &SessionMetadata) -> Result<PathBuf, Error>
where
    P: AsRef<Path>,
{
    // TODO: maybe can store the authenticated user in a global variable so that
    // we don't have to call load again if we already did it before.
    let credentials = Credentials::load_at(&config_dir)?;
    let username = credentials.account.id;

    let path = get_user_relative_file_path(&username, chain);
    let file_path = config_dir.as_ref().join(path);

    // Create the parent directories if they don't yet exist.
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let contents = serde_json::to_string_pretty(session)?;
    fs::write(&file_path, contents)?;

    Ok(file_path)
}

// TODO(kariy): this function should probably be put in a more generic `controller` rust sdk.
/// Creates a new session token for the given user. This will open a browser to the Cartridge
/// Controller keychain page to prompt user to create a new session for the given policies and
/// network. Returns the newly created session token.
#[tracing::instrument(name = "create_session", level = "trace", skip(rpc_url), fields(
    policies = policies.len()
))]
pub async fn create_user_session<U>(
    username: &str,
    rpc_url: U,
    policies: &[Policy],
) -> Result<SessionMetadata, Error>
where
    U: Into<Url>,
{
    let rpc_url: Url = rpc_url.into();
    let mut rx = open_session_creation_page(username, rpc_url.as_str(), policies)?;
    Ok(rx.recv().await.context("Failed to received the session.")?)
}

/// Starts the session creation process by opening the browser to the Cartridge keychain to prompt
/// the user to approve the session creation.
fn open_session_creation_page(
    username: &str,
    rpc_url: &str,
    policies: &[Policy],
) -> anyhow::Result<Receiver<SessionMetadata>> {
    let params = prepare_query_params(username, rpc_url, policies)?;
    let host = vars::get_cartridge_keychain_url();
    let url = format!("{host}{SESSION_CREATION_PATH}?{params}");

    let (tx, rx) = channel::<SessionMetadata>(1);
    let server = callback_server(tx)?;

    // get the callback server url
    let port = server.local_addr()?.port();
    let mut url = Url::parse(&url)?;

    // append the callback uri to the query params
    let callback_uri = format!("http://localhost:{port}/callback");
    url.query_pairs_mut()
        .append_pair("callback_uri", &callback_uri);

    browser::open(url.as_str())?;
    tokio::spawn(server.start());

    Ok(rx)
}

fn prepare_query_params(
    username: &str,
    rpc_url: &str,
    policies: &[Policy],
) -> Result<String, serde_json::Error> {
    let policies = policies
        .iter()
        .map(serde_json::to_string)
        .map(|p| Ok(urlencoding::encode(&p?).into_owned()))
        .collect::<Result<Vec<String>, _>>()?
        .join(",");

    Ok(format!(
        "username={username}&rpc_url={rpc_url}&policies=[{policies}]",
    ))
}

#[derive(Debug, thiserror::Error)]
enum CallbackError {
    #[error("Internal server error")]
    Unexpected,
}

impl IntoResponse for CallbackError {
    fn into_response(self) -> Response {
        match self {
            Self::Unexpected => {
                let status = StatusCode::INTERNAL_SERVER_ERROR;
                (status, self.to_string()).into_response()
            }
        }
    }
}

/// Create the callback server that will receive the session token from the browser.
fn callback_server(result_sender: Sender<SessionMetadata>) -> anyhow::Result<LocalServer> {
    type HandlerState = State<(Sender<SessionMetadata>, Sender<()>)>;

    // Request handler for the /callback endpoint.
    let handler = |state: HandlerState, json: Json<SessionMetadata>| async move {
        info!("Received session token from the browser.");

        let State((res_sender, shutdown_sender)) = state;
        let Json(session) = json;

        println!("response: {session:?}");

        // Parse the session token from the json payload.
        res_sender
            .send(session)
            .await
            .map_err(|_| CallbackError::Unexpected)?;

        // send shutdown signal to the server ONLY after succesfully receiving and processing
        // the session token.
        shutdown_sender
            .send(())
            .await
            .map_err(|_| CallbackError::Unexpected)?;

        Ok::<(), CallbackError>(())
    };

    let (shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel(1);

    let router = Router::new()
        .route("/callback", post(handler))
        .with_state((result_sender, shutdown_tx));

    Ok(LocalServer::new(router)?
        .cors(CorsLayer::permissive())
        .with_shutdown_signal(shutdown_rx))
}

fn get_user_relative_file_path(username: &str, chain_id: Felt) -> PathBuf {
    let file_name = format!("{chain_id:#x}-{}", SESSION_FILE_BASE_NAME);
    PathBuf::from(username).join(file_name)
}

#[cfg(test)]
mod tests {
    use super::get;
    use crate::account::{Account, AccountCredentials};
    use crate::credential::{AccessToken, Credentials};
    use crate::error::Error::Unauthorized;
    use crate::session::{get_at, get_user_relative_file_path, store_at};
    use crate::utils;
    use account_sdk::storage::SessionMetadata;
    use starknet::{core::types::Felt, macros::felt};
    use std::ffi::OsStr;
    use std::path::{Component, Path};
    use tokio::sync::mpsc::channel;

    fn authenticate(config_dir: impl AsRef<Path>) -> &'static str {
        static USERNAME: &str = "foo";

        let token = AccessToken {
            token: "mytoken".to_string(),
            r#type: "Bearer".to_string(),
        };

        let account = Account {
            name: None,
            id: USERNAME.to_string(),
            contract_address: felt!("0x999"),
            credentials: AccountCredentials {
                webauthn: Vec::new(),
            },
        };

        let cred = Credentials::new(account, token);
        let _ = Credentials::store_at(&config_dir, &cred).unwrap();

        USERNAME
    }

    #[test]
    fn user_rel_path() {
        let chain = felt!("0x999");
        let username = "foo";
        let file_name = "0x999-session.json";

        let path = get_user_relative_file_path(username, chain);
        let mut comps = path.components();

        assert_eq!(comps.next(), Some(Component::Normal(OsStr::new(username))));
        assert_eq!(comps.next(), Some(Component::Normal(OsStr::new(file_name))));
        assert_eq!(comps.next(), None);
    }

    #[test]
    fn get_session_unauthenticated() {
        let chain = Felt::ONE;
        let err = get(chain).unwrap_err();
        let Unauthorized = err else {
            panic!("expected Unauthorized error, got {err:?}");
        };
    }

    #[test]
    fn get_non_existant_session_authenticated() {
        let config_dir = utils::config_dir();
        authenticate(&config_dir);

        let chain = felt!("0x999");
        let result = get_at(config_dir, chain).unwrap();
        assert!(result.is_none())
    }

    #[test]
    fn get_existant_session_authenticated() {
        let config_dir = utils::config_dir();
        let username = authenticate(&config_dir);

        let chain = felt!("0x999");
        let expected = SessionMetadata::default();
        let path = store_at(&config_dir, chain, &expected).unwrap();

        let user_path = get_user_relative_file_path(username, chain);
        let actual = get_at(config_dir, chain).unwrap();

        assert_eq!(Some(expected), actual);
        assert!(path.ends_with(user_path));
    }

    #[test]
    fn store_session_unauthenticated() {
        let config_dir = utils::config_dir();

        let chain = felt!("0x999");
        let session = SessionMetadata::default();

        let err = store_at(config_dir, chain, &session).unwrap_err();
        assert!(err.to_string().contains("No credentials found"))
    }

    #[tokio::test]
    async fn test_callback_server() {
        let (tx, mut rx) = channel::<SessionMetadata>(1);
        let server = super::callback_server(tx).expect("failed to create server");

        // get the callback url
        let port = server.local_addr().unwrap().port();
        let url = format!("http://localhost:{port}/callback");

        // start the callback server
        tokio::spawn(server.start());

        // call the callback url
        let session = SessionMetadata::default();
        let res = reqwest::Client::new()
            .post(url)
            .json(&session)
            .send()
            .await
            .expect("failed to call callback url");

        assert!(res.status().is_success());

        let actual = rx.recv().await.expect("failed to receive session");
        assert_eq!(session, actual)
    }
}
