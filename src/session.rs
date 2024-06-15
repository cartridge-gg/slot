use std::io;
use std::path::Path;
use std::{fs, path::PathBuf};

use anyhow::Context;
use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use starknet::core::types::FieldElement;
use thiserror::Error;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tower_http::cors::CorsLayer;
use tracing::trace;
use url::Url;

use crate::credential::{self, Credentials};
use crate::utils::{self};
use crate::{browser, server::LocalServer};

const SESSION_CREATION_PAGE: &str = "https://x.cartridge.gg/slot/session";
const SESSION_FILE_BASE_NAME: &str = "session.json";

/// A policy defines what action can be performed by the session key.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Policy {
    /// The target contract address.
    pub target: FieldElement,
    /// The method name.
    pub method: String,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetails {
    /// The expiration date of the session.
    pub expires_at: String,
    /// The session's policies.
    pub policies: Vec<Policy>,
    pub credentials: SessionCredentials,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionCredentials {
    /// The signing key of the session.
    pub private_key: FieldElement,
    pub authorization: Vec<FieldElement>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    Credentials(#[from] credential::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Retrieves the session for the given chain id of the currently authenticated user.
/// Returns `None` if no session can be found for the chain id.
///
/// # Errors
///
/// This function will return an error if there is no authenticated user.
///
pub fn get(chain: FieldElement) -> Result<Option<SessionDetails>, Error> {
    get_at(utils::config_dir(), chain)
}

/// Stores the session on-disk. Returns the path to the file where the `session` has been written to.
///
/// # Errors
///
/// This function will return an error if there is no authenticated user.
///
pub fn store(chain: FieldElement, session: &SessionDetails) -> Result<PathBuf, Error> {
    store_at(utils::config_dir(), chain, session)
}

/// Creates a new session token. This will open a browser to the Cartridge Controller keychain page
/// to prompt user to create a new session for the given policies and network. Returns the newly
/// created session token.
///
/// # Errors
///
/// This function will return an error if there is no authenticated user.
///
#[tracing::instrument(level = "trace", skip(rpc_url), fields(policies = policies.len()))]
pub async fn create<U>(rpc_url: U, policies: &[Policy]) -> Result<SessionDetails, Error>
where
    U: Into<Url>,
{
    let credentials = Credentials::load()?;
    let username = credentials.account.expect("id must exist").id;

    let rpc_url: Url = rpc_url.into();
    let mut rx = open_session_creation_page(&username, rpc_url.as_str(), policies)?;

    Ok(rx.recv().await.context("Channel dropped.")?)
}

fn get_at<P>(config_dir: P, chain: FieldElement) -> Result<Option<SessionDetails>, Error>
where
    P: AsRef<Path>,
{
    let credentials = Credentials::load_at(&config_dir)?;
    let username = credentials.account.expect("id must exist").id;

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

fn store_at<P>(
    config_dir: P,
    chain: FieldElement,
    session: &SessionDetails,
) -> Result<PathBuf, Error>
where
    P: AsRef<Path>,
{
    // TODO: maybe can store the authenticated user in a global variable so that
    // we don't have to call load again if we already did it before.
    let credentials = Credentials::load_at(&config_dir)?;
    let username = credentials.account.expect("id must exist").id;

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

/// Starts the session creation process by opening the browser to the Cartridge keychain to prompt
/// the user to approve the session creation.
fn open_session_creation_page(
    username: &str,
    rpc_url: &str,
    policies: &[Policy],
) -> anyhow::Result<Receiver<SessionDetails>> {
    let params = prepare_query_params(username, rpc_url, policies)?;
    let url = format!("{SESSION_CREATION_PAGE}?{params}");

    let (tx, rx) = channel::<SessionDetails>(1);
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

/// Create the callback server that will receive the session token from the browser.
fn callback_server(tx: Sender<SessionDetails>) -> anyhow::Result<LocalServer> {
    let handler = move |tx: State<Sender<SessionDetails>>, session: Json<SessionDetails>| async move {
        trace!("Received session token from the browser.");
        tx.0.send(session.0).await.expect("qed; channel closed");
    };

    let router = Router::new()
        .route("/callback", post(handler))
        .with_state(tx);

    Ok(LocalServer::new(router)?.cors(CorsLayer::permissive()))
}

fn get_user_relative_file_path(username: &str, chain_id: FieldElement) -> PathBuf {
    let file_name = format!("{chain_id:#x}-{}", SESSION_FILE_BASE_NAME);
    PathBuf::from(username).join(file_name)
}

#[cfg(test)]
mod tests {
    use super::{get, Error};
    use crate::credential::{AccessToken, Credentials, Error::Unauthorized};
    use crate::graphql::auth::me::{MeMe, MeMeCredentials};
    use crate::session::{get_at, store_at, SessionDetails};
    use crate::utils;
    use starknet::{core::types::FieldElement, macros::felt};
    use std::path::Path;
    use tokio::sync::mpsc::channel;

    fn authenticate(config_dir: impl AsRef<Path>) {
        let token = AccessToken {
            token: "mytoken".to_string(),
            r#type: "Bearer".to_string(),
        };

        let me = MeMe {
            name: None,
            id: "foo".to_string(),
            contract_address: None,
            credentials: MeMeCredentials { webauthn: None },
        };

        let cred = Credentials::new(Some(me), token);
        let _ = Credentials::store_at(&config_dir, &cred).unwrap();
    }

    #[test]
    fn get_session_unauthenticated() {
        let chain = FieldElement::ONE;
        let err = get(chain).unwrap_err();
        let Error::Credentials(Unauthorized) = err else {
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
        authenticate(&config_dir);

        let chain = felt!("0x999");
        let expected = SessionDetails::default();
        store_at(&config_dir, chain, &expected).unwrap();

        let actual = get_at(config_dir, chain).unwrap();
        assert_eq!(Some(expected), actual)
    }

    #[test]
    fn store_session_unauthenticated() {
        let config_dir = utils::config_dir();

        let chain = felt!("0x999");
        let session = SessionDetails::default();

        let err = store_at(config_dir, chain, &session).unwrap_err();
        assert!(err.to_string().contains("No credentials found"))
    }

    #[tokio::test]
    async fn test_callback_server() {
        let (tx, mut rx) = channel::<SessionDetails>(1);
        let server = super::callback_server(tx).expect("failed to create server");

        // get the callback url
        let port = server.local_addr().unwrap().port();
        let url = format!("http://localhost:{port}/callback");

        // start the callback server
        tokio::spawn(server.start());

        // call the callback url
        let session = SessionDetails::default();
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
