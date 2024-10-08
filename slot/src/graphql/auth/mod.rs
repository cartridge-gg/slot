use graphql_client::GraphQLQuery;
use me::MeMe;

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
    MissingCredentials,

    #[error("Missing contract address")]
    MissingContractAddress,
}

impl TryFrom<MeMe> for Account {
    type Error = AccountTryFromGraphQLError;

    fn try_from(value: MeMe) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            name: value.name,
            credentials: value.credentials.try_into()?,
        })
    }
}

impl TryFrom<me::MeMeCredentials> for AccountCredentials {
    type Error = AccountTryFromGraphQLError;

    fn try_from(value: me::MeMeCredentials) -> Result<Self, Self::Error> {
        let webauthn = value
            .webauthn
            .ok_or(AccountTryFromGraphQLError::MissingCredentials)?
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
            controllers: None,
        };

        let account = Account::try_from(me).unwrap();

        assert_eq!(account.id, "id");
        assert_eq!(account.name, Some("name".to_string()));
        assert_eq!(account.credentials.webauthn.len(), 1);
        assert_eq!(account.credentials.webauthn[0].id, "id".to_string());
        assert_eq!(
            account.credentials.webauthn[0].public_key,
            "foo".to_string()
        );
    }
}
