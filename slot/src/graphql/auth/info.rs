#![allow(clippy::all, warnings)]
pub struct Me;
pub mod me {
    #![allow(dead_code)]
    use std::result::Result;
    pub const OPERATION_NAME: &str = "Me";
    pub const QUERY : & str = "query Me {\n  me {\n    id\n    username\n    creditsPlain\n\n    teams {\n      edges {\n        node {\n          id\n          name\n          credits\n          deleted\n\n          membership {\n            edges {\n              node {\n                account {\n                  id\n                  username\n                }\n                role\n              }\n            }\n          }\n\n          deployments {\n            edges {\n              node {\n                id\n                project\n                branch\n                serviceID\n                status\n                deprecated\n              }\n            }\n          }\n        }\n      }\n    }\n\n    controllers {\n      edges {\n        node {\n          id\n          address\n\n          signers {\n            id\n            type\n          }\n        }\n      }\n    }\n\n    credentials {\n      webauthn {\n        id\n        publicKey\n      }\n    }\n  }\n}\n\n" ;
    use super::*;
    use serde::{Deserialize, Serialize};
    #[allow(dead_code)]
    type Boolean = bool;
    #[allow(dead_code)]
    type Float = f64;
    #[allow(dead_code)]
    type Int = i64;
    #[allow(dead_code)]
    type ID = String;
    #[derive()]
    pub enum AccountTeamRole {
        owner,
        Other(String),
    }
    impl ::serde::Serialize for AccountTeamRole {
        fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
            ser.serialize_str(match *self {
                AccountTeamRole::owner => "owner",
                AccountTeamRole::Other(ref s) => &s,
            })
        }
    }
    impl<'de> ::serde::Deserialize<'de> for AccountTeamRole {
        fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s: String = ::serde::Deserialize::deserialize(deserializer)?;
            match s.as_str() {
                "owner" => Ok(AccountTeamRole::owner),
                _ => Ok(AccountTeamRole::Other(s)),
            }
        }
    }
    #[derive()]
    pub enum DeploymentStatus {
        active,
        disabled,
        error,
        deleted,
        Other(String),
    }
    impl ::serde::Serialize for DeploymentStatus {
        fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
            ser.serialize_str(match *self {
                DeploymentStatus::active => "active",
                DeploymentStatus::disabled => "disabled",
                DeploymentStatus::error => "error",
                DeploymentStatus::deleted => "deleted",
                DeploymentStatus::Other(ref s) => &s,
            })
        }
    }
    impl<'de> ::serde::Deserialize<'de> for DeploymentStatus {
        fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s: String = ::serde::Deserialize::deserialize(deserializer)?;
            match s.as_str() {
                "active" => Ok(DeploymentStatus::active),
                "disabled" => Ok(DeploymentStatus::disabled),
                "error" => Ok(DeploymentStatus::error),
                "deleted" => Ok(DeploymentStatus::deleted),
                _ => Ok(DeploymentStatus::Other(s)),
            }
        }
    }
    #[derive()]
    pub enum SignerType {
        starknet_account,
        webauthn,
        starknet,
        secp256k1,
        secp256r1,
        eip191,
        siws,
        Other(String),
    }
    impl ::serde::Serialize for SignerType {
        fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
            ser.serialize_str(match *self {
                SignerType::starknet_account => "starknet_account",
                SignerType::webauthn => "webauthn",
                SignerType::starknet => "starknet",
                SignerType::secp256k1 => "secp256k1",
                SignerType::secp256r1 => "secp256r1",
                SignerType::eip191 => "eip191",
                SignerType::siws => "siws",
                SignerType::Other(ref s) => &s,
            })
        }
    }
    impl<'de> ::serde::Deserialize<'de> for SignerType {
        fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s: String = ::serde::Deserialize::deserialize(deserializer)?;
            match s.as_str() {
                "starknet_account" => Ok(SignerType::starknet_account),
                "webauthn" => Ok(SignerType::webauthn),
                "starknet" => Ok(SignerType::starknet),
                "secp256k1" => Ok(SignerType::secp256k1),
                "secp256r1" => Ok(SignerType::secp256r1),
                "eip191" => Ok(SignerType::eip191),
                "siws" => Ok(SignerType::siws),
                _ => Ok(SignerType::Other(s)),
            }
        }
    }
    #[derive(Serialize)]
    pub struct Variables;
    #[derive(Deserialize)]
    pub struct ResponseData {
        pub me: Option<MeMe>,
    }
    #[derive(Deserialize)]
    pub struct MeMe {
        pub id: ID,
        pub username: String,
        #[serde(rename = "creditsPlain")]
        pub credits_plain: Int,
        pub teams: MeMeTeams,
        pub controllers: MeMeControllers,
        pub credentials: MeMeCredentials,
    }
    #[derive(Deserialize)]
    pub struct MeMeTeams {
        pub edges: Option<Vec<Option<MeMeTeamsEdges>>>,
    }
    #[derive(Deserialize)]
    pub struct MeMeTeamsEdges {
        pub node: Option<MeMeTeamsEdgesNode>,
    }
    #[derive(Deserialize)]
    pub struct MeMeTeamsEdgesNode {
        pub id: ID,
        pub name: String,
        pub credits: Int,
        pub deleted: Boolean,
        pub membership: MeMeTeamsEdgesNodeMembership,
        pub deployments: MeMeTeamsEdgesNodeDeployments,
    }
    #[derive(Deserialize)]
    pub struct MeMeTeamsEdgesNodeMembership {
        pub edges: Option<Vec<Option<MeMeTeamsEdgesNodeMembershipEdges>>>,
    }
    #[derive(Deserialize)]
    pub struct MeMeTeamsEdgesNodeMembershipEdges {
        pub node: Option<MeMeTeamsEdgesNodeMembershipEdgesNode>,
    }
    #[derive(Deserialize)]
    pub struct MeMeTeamsEdgesNodeMembershipEdgesNode {
        pub account: MeMeTeamsEdgesNodeMembershipEdgesNodeAccount,
        pub role: AccountTeamRole,
    }
    #[derive(Deserialize)]
    pub struct MeMeTeamsEdgesNodeMembershipEdgesNodeAccount {
        pub id: ID,
        pub username: String,
    }
    #[derive(Deserialize)]
    pub struct MeMeTeamsEdgesNodeDeployments {
        pub edges: Option<Vec<Option<MeMeTeamsEdgesNodeDeploymentsEdges>>>,
    }
    #[derive(Deserialize)]
    pub struct MeMeTeamsEdgesNodeDeploymentsEdges {
        pub node: Option<MeMeTeamsEdgesNodeDeploymentsEdgesNode>,
    }
    #[derive(Deserialize)]
    pub struct MeMeTeamsEdgesNodeDeploymentsEdgesNode {
        pub id: ID,
        pub project: String,
        pub branch: Option<String>,
        #[serde(rename = "serviceID")]
        pub service_id: ID,
        pub status: DeploymentStatus,
        pub deprecated: Option<Boolean>,
    }
    #[derive(Deserialize)]
    pub struct MeMeControllers {
        pub edges: Option<Vec<Option<MeMeControllersEdges>>>,
    }
    #[derive(Deserialize)]
    pub struct MeMeControllersEdges {
        pub node: Option<MeMeControllersEdgesNode>,
    }
    #[derive(Deserialize)]
    pub struct MeMeControllersEdgesNode {
        pub id: ID,
        pub address: String,
        pub signers: Option<Vec<MeMeControllersEdgesNodeSigners>>,
    }
    #[derive(Deserialize)]
    pub struct MeMeControllersEdgesNodeSigners {
        pub id: ID,
        #[serde(rename = "type")]
        pub type_: SignerType,
    }
    #[derive(Deserialize)]
    pub struct MeMeCredentials {
        pub webauthn: Option<Vec<MeMeCredentialsWebauthn>>,
    }
    #[derive(Deserialize)]
    pub struct MeMeCredentialsWebauthn {
        pub id: ID,
        #[serde(rename = "publicKey")]
        pub public_key: String,
    }
}
impl graphql_client::GraphQLQuery for Me {
    type Variables = me::Variables;
    type ResponseData = me::ResponseData;
    fn build_query(variables: Self::Variables) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: me::QUERY,
            operation_name: me::OPERATION_NAME,
        }
    }
}
