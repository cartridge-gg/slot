use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Account {
    pub id: String,
    pub name: Option<String>,
    pub credentials: AccountCredentials,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AccountCredentials {
    pub webauthn: Vec<WebAuthnCredential>,
}

#[derive(Deserialize, Debug, Clone, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebAuthnCredential {
    pub id: String,
    pub public_key: String,
}
