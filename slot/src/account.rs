pub use crate::graphql::auth::me::MeMeCredentialsWebauthn as WebAuthnCredential;
use serde::{Deserialize, Serialize};
use starknet::core::types::Felt;

/// Controller account information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct AccountInfo {
    pub id: String,
    pub username: String,
    pub controllers: Vec<Controller>,
    pub credentials: Vec<WebAuthnCredential>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Controller {
    pub id: String,
    /// The address of the Controller contract.
    pub address: Felt,
}
