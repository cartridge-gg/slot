#[allow(clippy::all, warnings)]
pub struct CreateDeployment;
pub mod create_deployment {
    #![allow(dead_code)]
    use std::result::Result;
    pub const OPERATION_NAME: &str = "CreateDeployment";
    pub const QUERY : & str = "mutation CreateDeployment(\n\t$project: String!\n\t$service: CreateServiceInput!\n\t$tier: DeploymentTier!\n\t$wait: Boolean\n\t$regions: [String!]\n) {\n\tcreateDeployment(\n\t\tname: $project\n\t\tservice: $service\n\t\ttier: $tier\n\t\twait: $wait\n\t\tregions: $regions\n\t) {\n\t\t__typename\n\n\t\t... on KatanaConfig {\n\t\t\tconfigFile\n\t\t\tgenesis\n\t\t}\n\n\t\t... on ToriiConfig {\n\t\t\tconfigFile\n\t\t}\n\n\t\t... on SayaConfig {\n\t\t\trpcUrl\n\t\t}\n\t}\n}\n" ;
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
    pub struct CreateKatanaConfigInput {
        pub genesis: Option<String>,
        #[serde(rename = "configFile")]
        pub config_file: Option<String>,
    }
    #[derive(Serialize)]
    pub struct CreateSayaConfigInput {
        pub mode: String,
        #[serde(rename = "rpcUrl")]
        pub rpc_url: String,
        pub registry: String,
        #[serde(rename = "settlementContract")]
        pub settlement_contract: String,
        pub world: String,
        #[serde(rename = "proverUrl")]
        pub prover_url: String,
        #[serde(rename = "storeProofs")]
        pub store_proofs: Boolean,
        #[serde(rename = "starknetUrl")]
        pub starknet_url: String,
        #[serde(rename = "signerKey")]
        pub signer_key: String,
        #[serde(rename = "signerAddress")]
        pub signer_address: String,
        #[serde(rename = "privateKey")]
        pub private_key: String,
        #[serde(rename = "batchSize")]
        pub batch_size: Int,
        #[serde(rename = "startBlock")]
        pub start_block: Int,
    }
    #[derive(Serialize)]
    pub struct CreateServiceConfigInput {
        pub katana: Option<CreateKatanaConfigInput>,
        pub torii: Option<CreateToriiConfigInput>,
        pub saya: Option<CreateSayaConfigInput>,
    }
    #[derive(Serialize)]
    pub struct CreateServiceInput {
        #[serde(rename = "type")]
        pub type_: DeploymentService,
        pub version: Option<String>,
        pub config: Option<CreateServiceConfigInput>,
    }
    #[derive(Serialize)]
    pub struct CreateToriiConfigInput {
        #[serde(rename = "configFile")]
        pub config_file: Option<String>,
    }
    #[derive(Serialize)]
    pub struct Variables {
        pub project: String,
        pub service: CreateServiceInput,
        pub tier: DeploymentTier,
        pub wait: Option<Boolean>,
        pub regions: Option<Vec<String>>,
    }
    impl Variables {}
    #[derive(Deserialize)]
    pub struct ResponseData {
        #[serde(rename = "createDeployment")]
        pub create_deployment: CreateDeploymentCreateDeployment,
    }
    #[derive(Deserialize)]
    #[serde(tag = "__typename")]
    pub enum CreateDeploymentCreateDeployment {
        KatanaConfig(CreateDeploymentCreateDeploymentOnKatanaConfig),
        ToriiConfig(CreateDeploymentCreateDeploymentOnToriiConfig),
        SayaConfig(CreateDeploymentCreateDeploymentOnSayaConfig),
    }
    #[derive(Deserialize)]
    pub struct CreateDeploymentCreateDeploymentOnKatanaConfig {
        #[serde(rename = "configFile")]
        pub config_file: Option<String>,
        pub genesis: Option<String>,
    }
    #[derive(Deserialize)]
    pub struct CreateDeploymentCreateDeploymentOnToriiConfig {
        #[serde(rename = "configFile")]
        pub config_file: Option<String>,
    }
    #[derive(Deserialize)]
    pub struct CreateDeploymentCreateDeploymentOnSayaConfig {
        #[serde(rename = "rpcUrl")]
        pub rpc_url: String,
    }
}
impl graphql_client::GraphQLQuery for CreateDeployment {
    type Variables = create_deployment::Variables;
    type ResponseData = create_deployment::ResponseData;
    fn build_query(variables: Self::Variables) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: create_deployment::QUERY,
            operation_name: create_deployment::OPERATION_NAME,
        }
    }
}
