use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet::core::types::FieldElement;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Account {
    pub id: String,
    pub name: Option<String>,
    #[serde_as(as = "UfeHex")]
    #[serde(rename = "contractAddress")]
    pub contract_address: FieldElement,
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
