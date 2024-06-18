use std::str::FromStr;

use graphql_client::GraphQLQuery;
use me::MeMe;
use starknet::core::types::{FieldElement, FromStrError};

use crate::account::{Account, AccountCredentials, WebAuthnCredential};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/auth/info.graphql",
    response_derives = "Debug, Clone, Serialize, PartialEq, Eq"
)]
pub struct Me;

#[derive(Debug, thiserror::Error)]
pub enum AccountTryFromGraphQLError {
    #[error("Missing WebAuthn credentials")]
    MissingCredendentials,

    #[error("Missing contract address")]
    MissingContractAddress,

    #[error("Invalid contract address: {0}")]
    InvalidContractAddress(#[from] FromStrError),
}

impl TryFrom<MeMe> for Account {
    type Error = AccountTryFromGraphQLError;

    fn try_from(value: MeMe) -> Result<Self, Self::Error> {
        let address = value
            .contract_address
            .ok_or(AccountTryFromGraphQLError::MissingContractAddress)?;

        Ok(Self {
            id: value.id,
            name: value.name,
            credentials: value.credentials.try_into()?,
            contract_address: FieldElement::from_str(&address)?,
        })
    }
}

impl TryFrom<me::MeMeCredentials> for AccountCredentials {
    type Error = AccountTryFromGraphQLError;

    fn try_from(value: me::MeMeCredentials) -> Result<Self, Self::Error> {
        let webauthn = value
            .webauthn
            .ok_or(AccountTryFromGraphQLError::MissingCredendentials)?
            .into_iter()
            .map(WebAuthnCredential::from)
            .collect();

        Ok(Self { webauthn })
    }
}

impl From<me::MeMeCredentialsWebauthn> for WebAuthnCredential {
    fn from(value: me::MeMeCredentialsWebauthn) -> Self {
        Self {
            id: value.id,
            public_key: value.public_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_me() {
        let me = MeMe {
            id: "id".to_string(),
            name: Some("name".to_string()),
            credentials: me::MeMeCredentials {
                webauthn: Some(vec![me::MeMeCredentialsWebauthn {
                    id: "id".to_string(),
                    public_key: "foo".to_string(),
                }]),
            },
            contract_address: Some("0x1".to_string()),
        };

        let account = Account::try_from(me).unwrap();

        assert_eq!(account.id, "id");
        assert_eq!(account.name, Some("name".to_string()));
        assert_eq!(account.contract_address, FieldElement::ONE);
        assert_eq!(account.credentials.webauthn.len(), 1);
        assert_eq!(account.credentials.webauthn[0].id, "id".to_string());
        assert_eq!(
            account.credentials.webauthn[0].public_key,
            "foo".to_string()
        );
    }
}
