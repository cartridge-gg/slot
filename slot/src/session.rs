use std::path::Path;
use std::{fs, path::PathBuf};

use account_sdk::account::session::hash::{AllowedMethod, Session};
use account_sdk::account::session::SessionAccount;
use account_sdk::signers::{HashSigner, Signer};
use anyhow::Context;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, routing::post, Router};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use starknet::core::types::Felt;
use starknet::core::utils::{get_selector_from_name, NonAsciiNameError};
use starknet::macros::short_string;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use starknet::signers::SigningKey;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tower_http::cors::CorsLayer;
use tracing::info;
use url::Url;

use crate::credential::Credentials;
use crate::error::Error;
use crate::utils::{self};
use crate::{browser, server::LocalServer, vars};

// Taken from: https://github.com/cartridge-gg/controller/blob/1d7352fce437ccd0b992ca5420aeb3719427e348/packages/account-wasm/src/lib.rs#L92-L95
const GUARDIAN: Felt = short_string!("CARTRIDGE_GUARDIAN");
pub const SESSION_GUARDIAN_SIGNING_KEY: SigningKey = SigningKey::from_secret_scalar(GUARDIAN);

// Taken from: https://github.com/cartridge-gg/controller/blob/046f3b98f410f71e4d14b8f40efaae57f6c5483e/packages/keychain/src/components/connect/CreateSession.tsx#L24
const DEFAULT_SESSION_EXPIRES_AT: u64 = 3000000000;
const SESSION_CREATION_PATH: &str = "/session";
const SESSION_FILE_BASE_NAME: &str = "session.json";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SessionAuth {
    /// The username of the Controller account.
    pub username: String,
    /// The address of the Controller account associated with the username.
    pub address: Felt,

    pub owner_guid: Felt,
    /// The private key of the signer who is authorized to use the session.
    pub signer: Felt,
}

/// A session object that has all the necessary information for creating the
/// [Session] object and the [SessionAccount](account_sdk::account::session::SessionAccount)
/// for using the session.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FullSessionInfo {
    pub chain_id: Felt,
    pub auth: SessionAuth,
    pub session: Session,
}

impl FullSessionInfo {
    /// Convert the session info into a [`SessionAccount`] instance.
    pub fn into_account<P>(self, provider: P) -> SessionAccount<P>
    where
        P: Provider + Send,
    {
        let session_guardian = Signer::Starknet(SESSION_GUARDIAN_SIGNING_KEY);
        let session_signer = Signer::Starknet(SigningKey::from_secret_scalar(self.auth.signer));

        SessionAccount::new_as_registered(
            provider,
            session_signer,
            session_guardian,
            self.auth.address,
            self.chain_id,
            self.auth.owner_guid,
            self.session,
        )
    }
}

/// A policy defines what action can be performed by the session key.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyMethod {
    /// The target contract address.
    pub target: Felt,
    /// The name of the contract method that the session can operate on.
    pub method: String,
}

/// Retrieves the session for the given chain id of the currently authenticated user.
/// Returns `None` if no session can be found for the chain id.
///
/// # Errors
///
/// This function will return an error if there is no authenticated user.
///
pub fn get(chain: Felt) -> Result<Option<FullSessionInfo>, Error> {
    get_at(utils::config_dir(), chain)
}

/// Stores the session on-disk. Returns the path to the file where the `session` has been written to.
///
/// # Errors
///
/// This function will return an error if there is no authenticated user.
///
pub fn store(chain: Felt, session: &FullSessionInfo) -> Result<PathBuf, Error> {
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
pub async fn create(rpc_url: Url, policies: &[PolicyMethod]) -> Result<FullSessionInfo, Error> {
    // TODO: allow user configurable.
    let signer = SigningKey::from_random();
    let pubkey = signer.verifying_key().scalar();

    let credentials = Credentials::load()?;
    let username = credentials.account.id;
    let response = create_user_session(pubkey, &username, rpc_url.clone(), policies).await?;

    let auth = SessionAuth {
        address: response.address,
        username: response.username,
        owner_guid: response.owner_guid,
        signer: signer.secret_scalar(),
    };

    let methods = policies
        .iter()
        .map(AllowedMethod::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Error::InvalidMethodName)?;

    let session = Session::new(methods, DEFAULT_SESSION_EXPIRES_AT, &signer.signer())?;
    let chain_id = get_network_chain_id(rpc_url).await?;

    Ok(FullSessionInfo {
        auth,
        session,
        chain_id,
    })
}

/// Get the session token of the chain id `chain` for the currently authenticated user. It will
/// use `config_dir` as the root path to look for the session file.
fn get_at(config_dir: impl AsRef<Path>, chain: Felt) -> Result<Option<FullSessionInfo>, Error> {
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
fn store_at(
    config_dir: impl AsRef<Path>,
    chain: Felt,
    session: &FullSessionInfo,
) -> Result<PathBuf, Error> {
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

/// The response object to the session creation request.
//
// A reflection of https://github.com/cartridge-gg/controller/blob/90b767bcc6478f0e02973f7237bc2a974f745adf/packages/keychain/src/pages/session.tsx#L15-L21
#[cfg_attr(test, derive(PartialEq, Serialize))]
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCreationResponse {
    /// The username of the Controller account.
    pub username: String,
    /// The address of the Controller account associated with the username.
    pub address: Felt,

    pub owner_guid: Felt,
    /// The hash of the session creation transaction. `None` is the session
    /// was not registered (already exist).
    pub transaction_hash: Option<Felt>,
    /// A flag indicating whether the session was already registered.
    ///
    /// Meaning similar seesion has already been created and registered to the Controller
    /// before.
    #[serde(default)]
    pub already_registered: bool,
}

impl SessionCreationResponse {
    // Following how the server serialize the response object:
    // https://github.com/cartridge-gg/controller/blob/90b767bcc6478f0e02973f7237bc2a974f745adf/packages/keychain/src/pages/session.tsx#L58-L60
    pub fn from_encoded(encoded: &str) -> anyhow::Result<Self> {
        use base64::{engine::general_purpose, Engine as _};

        // Decode the Base64 string
        let bytes = general_purpose::STANDARD.decode(encoded)?;
        let decoded = String::from_utf8(bytes)?;

        Ok(serde_json::from_str(&decoded)?)
    }
}

// TODO(kariy): this function should probably be put in a more generic `controller` rust sdk.
/// Creates a new session token for the given user. This will open a browser to the Cartridge
/// Controller keychain page to prompt user to create a new session for the given policies and
/// network. Returns the newly created session token.
#[tracing::instrument(name = "create_session", level = "trace", skip(rpc_url), fields(
    policies = policies.len()
))]
pub async fn create_user_session(
    public_key: Felt,
    username: &str,
    rpc_url: impl Into<Url>,
    policies: &[PolicyMethod],
) -> Result<SessionCreationResponse, Error> {
    let rpc_url: Url = rpc_url.into();
    let input = SessionCreationInput {
        policies,
        username,
        public_key,
        rpc_url: rpc_url.as_str(),
    };

    let mut rx = open_session_creation_page(input)?;
    let encoded_response = rx.recv().await.context("Failed to received the session.")?;
    let response = SessionCreationResponse::from_encoded(&encoded_response)?;

    Ok(response)
}

/// Input parameters for creating a new session.
struct SessionCreationInput<'a> {
    public_key: Felt,
    username: &'a str,
    rpc_url: &'a str,
    policies: &'a [PolicyMethod],
}

/// Starts the session creation process by opening the browser to the Cartridge keychain to prompt
/// the user to approve the session creation.
fn open_session_creation_page(
    input: SessionCreationInput<'_>,
) -> anyhow::Result<Receiver<EncodedResponse>> {
    let params = prepare_query_params(input)?;
    let host = vars::get_cartridge_keychain_url();
    let url = format!("{host}{SESSION_CREATION_PATH}?{params}");

    let (tx, rx) = mpsc::channel(1);
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

fn prepare_query_params(input: SessionCreationInput<'_>) -> Result<String, serde_json::Error> {
    let policies = input
        .policies
        .iter()
        .map(serde_json::to_string)
        .map(|p| Ok(urlencoding::encode(&p?).into_owned()))
        .collect::<Result<Vec<String>, _>>()?
        .join(",");

    Ok(format!(
        "username={}&public_key={}&rpc_url={}&policies=[{}]",
        input.username, input.public_key, input.rpc_url, policies
    ))
}

// Base64 encoded response sent from the internal server.
type EncodedResponse = String;

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
fn callback_server(result_sender: Sender<EncodedResponse>) -> anyhow::Result<LocalServer> {
    type HandlerState = State<(Sender<EncodedResponse>, Sender<()>)>;

    // Request handler for the /callback endpoint.
    let handler = |state: HandlerState, encoded_response: EncodedResponse| async move {
        info!("Received session token from the browser.");

        let State((res_sender, shutdown_sender)) = state;

        // Parse the session token from the json payload.
        res_sender
            .send(encoded_response)
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

async fn get_network_chain_id(url: Url) -> anyhow::Result<Felt> {
    let provider = JsonRpcClient::new(HttpTransport::new(url));
    Ok(provider.chain_id().await?)
}

impl TryFrom<PolicyMethod> for AllowedMethod {
    type Error = NonAsciiNameError;

    fn try_from(value: PolicyMethod) -> Result<Self, Self::Error> {
        Ok(Self::new(
            value.target,
            get_selector_from_name(&value.method)?,
        ))
    }
}

impl TryFrom<&PolicyMethod> for AllowedMethod {
    type Error = NonAsciiNameError;

    fn try_from(value: &PolicyMethod) -> Result<Self, Self::Error> {
        Ok(Self::new(
            value.target,
            get_selector_from_name(&value.method)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::{Account, AccountCredentials};
    use crate::credential::{AccessToken, Credentials};
    use crate::error::Error::Unauthorized;
    use crate::session::{get_at, get_user_relative_file_path, store_at};
    use crate::utils;
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
        let expected = FullSessionInfo::default();
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
        let session = FullSessionInfo::default();

        let err = store_at(config_dir, chain, &session).unwrap_err();
        assert!(err.to_string().contains("No credentials found"))
    }

    #[tokio::test]
    async fn test_callback_server() {
        let (tx, mut rx) = channel(1);
        let server = super::callback_server(tx).expect("failed to create server");

        // get the callback url
        let port = server.local_addr().unwrap().port();
        let url = format!("http://localhost:{port}/callback");

        // start the callback server
        tokio::spawn(server.start());

        // call the callback url
        let response = SessionCreationResponse::default();
        let res = reqwest::Client::new()
            .post(url)
            .json(&response)
            .send()
            .await
            .expect("failed to call callback url");

        assert!(res.status().is_success());

        let actual_encoded = rx.recv().await.expect("failed to receive session");
        let actual: SessionCreationResponse = serde_json::from_str(&actual_encoded).unwrap();

        assert_eq!(response, actual)
    }

    #[test]
    fn deserialize_backend_encoded_response() {
        let encoded_response = "eyJ1c2VybmFtZSI6ImpvaG5zbWl0aCIsImFkZHJlc3MiOiIweDM5NzMzM2U5OTNhZTE2MmI0NzY2OTBlMTQwMTU0OGFlOTdhODgxOTk1NTUwNmI4YmM5MThlMDY3YmRhZmMzIiwib3duZXJHdWlkIjoiMHg1ZDc3MDliMGE0ODVlNjRhNTQ5YWRhOWJkMTRkMzA0MTkzNjQxMjdkZmQzNTFlMDFmMzg4NzFjODI1MDBjZDciLCJ0cmFuc2FjdGlvbkhhc2giOiIweDRlOTY4ZWRkODFiYTQ2MjI0Zjc2MjNmNDA5NWQ3NTRkYzgwZjZjYmQ1NTU4M2NkZTBlZDJhMTQzYWViNzMyMSJ9";
        let response = SessionCreationResponse::from_encoded(encoded_response).unwrap();

        assert_eq!(response.username, "johnsmith");
        assert_eq!(
            response.address,
            felt!("0x397333e993ae162b476690e1401548ae97a8819955506b8bc918e067bdafc3")
        );
        assert_eq!(
            response.owner_guid,
            felt!("0x5d7709b0a485e64a549ada9bd14d30419364127dfd351e01f38871c82500cd7")
        );
        assert_eq!(
            response.transaction_hash,
            Some(felt!(
                "0x4e968edd81ba46224f7623f4095d754dc80f6cbd55583cde0ed2a143aeb7321"
            ))
        );
        assert!(!response.already_registered);
    }
}
