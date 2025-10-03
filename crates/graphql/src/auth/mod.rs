use std::str::FromStr;

use graphql_client::GraphQLQuery;
use me::MeMe;
use slot_core::account::AccountInfo;
use slot_core::account::Controller;
use slot_core::account::WebAuthnCredential;
use starknet_types_core::felt::Felt;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/auth/info.graphql",
    response_derives = "Debug, Clone, Serialize, PartialEq, Eq"
)]
pub struct Me;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/auth/update-me.graphql",
    response_derives = "Debug, Clone, Serialize, PartialEq, Eq"
)]
pub struct UpdateMe;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/auth/transfer.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct Transfer;

impl From<MeMe> for AccountInfo {
    fn from(value: MeMe) -> Self {
        let id = value.id;
        let username = value.username;
        let credentials = value
            .credentials
            .webauthn
            .unwrap_or_default()
            .into_iter()
            .map(WebAuthnCredential::from)
            .collect();

        let controllers = value
            .controllers
            .edges
            .unwrap_or_default()
            .into_iter()
            .map(|c| c.unwrap())
            .map(Controller::from)
            .collect();

        Self {
            id,
            username,
            controllers,
            credentials,
        }
    }
}

impl From<me::MeMeControllersEdges> for slot_core::account::Controller {
    fn from(value: me::MeMeControllersEdges) -> Self {
        let node = value.node.unwrap();
        let id = node.id;
        let address = Felt::from_str(&node.address).expect("valid address");

        Self { id, address }
    }
}

impl From<me::MeMeCredentialsWebauthn> for slot_core::account::WebAuthnCredential {
    fn from(value: me::MeMeCredentialsWebauthn) -> Self {
        Self {
            id: value.id,
            public_key: value.public_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use slot_core::account::AccountInfo;

    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_try_from_me() {
        let me = MeMe {
            id: "id".to_string(),
            username: "username".to_string(),
            credentials: me::MeMeCredentials {
                webauthn: Some(vec![me::MeMeCredentialsWebauthn {
                    id: "id".to_string(),
                    public_key: "foo".to_string(),
                }]),
            },
            controllers: me::MeMeControllers {
                edges: Some(vec![Some(me::MeMeControllersEdges {
                    node: Some(me::MeMeControllersEdgesNode {
                        id: "id".to_string(),
                        address: "0x123".to_string(),
                        signers: Some(vec![me::MeMeControllersEdgesNodeSigners {
                            id: "id".to_string(),
                            type_: me::SignerType::webauthn,
                        }]),
                    }),
                })]),
            },
            teams: me::MeMeTeams {
                edges: Some(vec![]),
            },
            credits_plain: 0,
        };

        let account = AccountInfo::from(me);

        assert_eq!(account.id, "id");
        assert_eq!(account.username, "username".to_string());
        assert_eq!(account.credentials.len(), 1);
        assert_eq!(account.credentials[0].id, "id".to_string());
        assert_eq!(account.credentials[0].public_key, "foo".to_string());
    }
}
