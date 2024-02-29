#![allow(clippy::enum_variant_names)]

use anyhow::Result;
use clap::Args;
use graphql_client::{GraphQLQuery, Response};

use crate::api::ApiClient;

use self::describe_deployment::{
    DeploymentService,
    DescribeDeploymentDeploymentConfig::{KatanaConfig, ToriiConfig, MadaraConfig},
    ResponseData, Variables,
};

use super::services::Service;

type Long = u64;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/command/deployments/describe.graphql",
    response_derives = "Debug"
)]
pub struct DescribeDeployment;

#[derive(Debug, Args)]
#[command(next_help_heading = "Describe options")]
pub struct DescribeArgs {
    #[arg(help = "The project of the project.")]
    pub project: String,

    #[arg(help = "The service of the project.")]
    pub service: Service,
}

impl DescribeArgs {
    pub async fn run(&self) -> Result<()> {
        let service = match self.service {
            Service::Torii => DeploymentService::torii,
            Service::Katana => DeploymentService::katana,
        };

        let request_body = DescribeDeployment::build_query(Variables {
            project: self.project.clone(),
            service,
        });

        let client = ApiClient::new();
        let res: Response<ResponseData> = client.post(&request_body).await?;
        if let Some(errors) = res.errors.clone() {
            for err in errors {
                println!("Error: {}", err.message);
            }
        }

        if let Some(data) = res.data {
            if let Some(deployment) = data.deployment {
                println!("Project: {}", deployment.project);
                println!(
                    "Branch: {}",
                    deployment.branch.unwrap_or_else(|| String::from("Default"))
                );
                println!("Tier: {:?}", deployment.tier);

                match deployment.config {
                    ToriiConfig(config) => {
                        println!("\nConfiguration:");
                        println!("  Version: {}", config.version);
                        println!("  World: {}", config.world);
                        println!("  RPC: {}", config.rpc);
                        println!("  Start Block: {}", config.start_block);
                        println!("\nEndpoints:");
                        println!("  GraphQL: {}", config.graphql);
                        println!("  GRPC: {}", config.grpc);
                    }
                    KatanaConfig(config) => {
                        println!("\nEndpoints:");
                        println!("  Version: {}", config.version);
                        println!("  RPC: {}", config.rpc);
                    }
                    MadaraConfig(config) => {
                        println!("\nEndpoints:");
                        println!("  Version: {}", config.version);
                        println!("  RPC: {}", config.rpc);
                    }
                }
            }
        }

        Ok(())
    }
}
