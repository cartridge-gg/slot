use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;

/// Controller account information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebAuthnCredential {
    pub id: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
}
