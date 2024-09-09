#![allow(clippy::enum_variant_names)]

use super::services::Service;
use anyhow::Result;
use clap::Args;
use slot::graphql::deployments::describe_deployment::DescribeDeploymentDeploymentConfig::{
    KatanaConfig, MadaraConfig, SayaConfig, ToriiConfig,
};
use slot::graphql::deployments::{describe_deployment::*, DescribeDeployment};
use slot::graphql::{GraphQLQuery, Response};
use slot::{api::Client, credential::Credentials};

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
            Service::Madara => DeploymentService::madara,
            Service::Saya => DeploymentService::saya,
        };

        let request_body = DescribeDeployment::build_query(Variables {
            project: self.project.clone(),
            service,
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let res: Response<ResponseData> = client.query(&request_body).await?;
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
                        println!("  Start Block: {}", config.start_block.unwrap_or(0));
                        println!(
                            "  Indexing Pending: {}",
                            config.index_pending.unwrap_or(false)
                        );
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
                    SayaConfig(config) => {
                        println!("\nEndpoints:");
                        println!("  RPC URL: {}", config.rpc_url);
                    }
                }
            }
        }

        Ok(())
    }
}
