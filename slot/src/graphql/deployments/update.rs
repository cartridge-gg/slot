#![allow(clippy::all, warnings)]
pub struct UpdateDeployment;
pub mod update_deployment {
    #![allow(dead_code)]
    use std::result::Result;
    pub const OPERATION_NAME: &str = "UpdateDeployment";
    pub const QUERY : & str = "mutation UpdateDeployment(\n  $project: String!\n  $service: UpdateServiceInput!\n  $tier: DeploymentTier!\n  $wait: Boolean\n) {\n  updateDeployment(\n    name: $project\n    service: $service\n    tier: $tier\n    wait: $wait\n  ) {\n    __typename\n\n    ... on KatanaConfig {\n      configFile\n    }\n\n    ... on ToriiConfig {\n      configFile\n    }\n\n    ... on SayaConfig {\n      rpcUrl\n    }\n  }\n}\n" ;
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
    pub enum DeploymentService {
        katana,
        torii,
        saya,
        Other(String),
    }
    impl ::serde::Serialize for DeploymentService {
        fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
            ser.serialize_str(match *self {
                DeploymentService::katana => "katana",
                DeploymentService::torii => "torii",
                DeploymentService::saya => "saya",
                DeploymentService::Other(ref s) => &s,
            })
        }
    }
    impl<'de> ::serde::Deserialize<'de> for DeploymentService {
        fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s: String = ::serde::Deserialize::deserialize(deserializer)?;
            match s.as_str() {
                "katana" => Ok(DeploymentService::katana),
                "torii" => Ok(DeploymentService::torii),
                "saya" => Ok(DeploymentService::saya),
                _ => Ok(DeploymentService::Other(s)),
            }
        }
    }
    #[derive()]
    pub enum DeploymentTier {
        basic,
        common,
        uncommon,
        rare,
        epic,
        legendary,
        insane,
        Other(String),
    }
    impl ::serde::Serialize for DeploymentTier {
        fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
            ser.serialize_str(match *self {
                DeploymentTier::basic => "basic",
                DeploymentTier::common => "common",
                DeploymentTier::uncommon => "uncommon",
                DeploymentTier::rare => "rare",
                DeploymentTier::epic => "epic",
                DeploymentTier::legendary => "legendary",
                DeploymentTier::insane => "insane",
                DeploymentTier::Other(ref s) => &s,
            })
        }
    }
    impl<'de> ::serde::Deserialize<'de> for DeploymentTier {
        fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s: String = ::serde::Deserialize::deserialize(deserializer)?;
            match s.as_str() {
                "basic" => Ok(DeploymentTier::basic),
                "common" => Ok(DeploymentTier::common),
                "uncommon" => Ok(DeploymentTier::uncommon),
                "rare" => Ok(DeploymentTier::rare),
                "epic" => Ok(DeploymentTier::epic),
                "legendary" => Ok(DeploymentTier::legendary),
                "insane" => Ok(DeploymentTier::insane),
                _ => Ok(DeploymentTier::Other(s)),
            }
        }
    }
    #[derive(Serialize)]
    pub struct UpdateKatanaConfigInput {
        #[serde(rename = "configFile")]
        pub config_file: Option<String>,
    }
    #[derive(Serialize)]
    pub struct UpdateServiceConfigInput {
        pub katana: Option<UpdateKatanaConfigInput>,
        pub torii: Option<UpdateToriiConfigInput>,
    }
    #[derive(Serialize)]
    pub struct UpdateServiceInput {
        #[serde(rename = "type")]
        pub type_: DeploymentService,
        pub version: Option<String>,
        pub config: Option<UpdateServiceConfigInput>,
    }
    #[derive(Serialize)]
    pub struct UpdateToriiConfigInput {
        #[serde(rename = "configFile")]
        pub config_file: Option<String>,
    }
    #[derive(Serialize)]
    pub struct Variables {
        pub project: String,
        pub service: UpdateServiceInput,
        pub tier: DeploymentTier,
        pub wait: Option<Boolean>,
    }
    impl Variables {}
    #[derive(Deserialize)]
    pub struct ResponseData {
        #[serde(rename = "updateDeployment")]
        pub update_deployment: UpdateDeploymentUpdateDeployment,
    }
    #[derive(Deserialize)]
    #[serde(tag = "__typename")]
    pub enum UpdateDeploymentUpdateDeployment {
        KatanaConfig(UpdateDeploymentUpdateDeploymentOnKatanaConfig),
        ToriiConfig(UpdateDeploymentUpdateDeploymentOnToriiConfig),
        SayaConfig(UpdateDeploymentUpdateDeploymentOnSayaConfig),
    }
    #[derive(Deserialize)]
    pub struct UpdateDeploymentUpdateDeploymentOnKatanaConfig {
        #[serde(rename = "configFile")]
        pub config_file: Option<String>,
    }
    #[derive(Deserialize)]
    pub struct UpdateDeploymentUpdateDeploymentOnToriiConfig {
        #[serde(rename = "configFile")]
        pub config_file: Option<String>,
    }
    #[derive(Deserialize)]
    pub struct UpdateDeploymentUpdateDeploymentOnSayaConfig {
        #[serde(rename = "rpcUrl")]
        pub rpc_url: String,
    }
}
impl graphql_client::GraphQLQuery for UpdateDeployment {
    type Variables = update_deployment::Variables;
    type ResponseData = update_deployment::ResponseData;
    fn build_query(variables: Self::Variables) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: update_deployment::QUERY,
            operation_name: update_deployment::OPERATION_NAME,
        }
    }
}
