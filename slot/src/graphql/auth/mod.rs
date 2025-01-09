use std::str::FromStr;

use graphql_client::GraphQLQuery;
use me::MeMe;
use starknet::core::types::Felt;

use crate::account::{self};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/auth/info.graphql",
    response_derives = "Debug, Clone, Serialize, PartialEq, Eq"
)]
pub struct Me;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/graphql/auth/update-me.graphql",
    response_derives = "Debug, Clone, Serialize, PartialEq, Eq"
)]
pub struct UpdateMe;

impl From<MeMe> for account::AccountInfo {
    fn from(value: MeMe) -> Self {
        let id = value.id;
        let name = value.name;
        let credentials = value.credentials.webauthn.unwrap_or_default();
        let controllers = value
            .controllers
            .edges
            .unwrap_or_default()
            .into_iter()
            .map(|c| c.unwrap())
            .map(account::Controller::from)
            .collect();

        Self {
            id,
            name,
            controllers,
            credentials,
        }
    }
}

impl From<me::MeMeControllersEdges> for account::Controller {
    fn from(value: me::MeMeControllersEdges) -> Self {
        let node = value.node.unwrap();
        let id = node.id;
        let address = Felt::from_str(&node.address).expect("valid address");
        let signers = node
            .signers
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.into())
            .collect::<Vec<_>>();

        Self {
            id,
            address,
            signers,
        }
    }
}

impl From<me::MeMeControllersEdgesNodeSigners> for account::ControllerSigner {
    fn from(value: me::MeMeControllersEdgesNodeSigners) -> Self {
        Self {
            id: value.id,
            r#type: value.type_.into(),
        }
    }
}

impl From<me::SignerType> for account::SignerType {
    fn from(value: me::SignerType) -> Self {
        match value {
            me::SignerType::webauthn => Self::WebAuthn,
            me::SignerType::starknet_account => Self::StarknetAccount,
            me::SignerType::Other(other) => Self::Other(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::account::AccountInfo;

    #[allow(unused_imports)]
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
        };

        let account = AccountInfo::from(me);

        assert_eq!(account.id, "id");
        assert_eq!(account.name, Some("name".to_string()));
        assert_eq!(account.credentials.len(), 1);
        assert_eq!(account.credentials[0].id, "id".to_string());
        assert_eq!(account.credentials[0].public_key, "foo".to_string());
    }
}
