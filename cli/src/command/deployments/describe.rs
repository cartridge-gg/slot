#![allow(clippy::enum_variant_names)]

use super::services::Service;
use anyhow::Result;
use clap::Args;
use slot::graphql::deployments::describe_deployment::DescribeDeploymentDeploymentConfig::{
    KatanaConfig, SayaConfig, ToriiConfig,
};
use slot::graphql::deployments::{describe_deployment::*, DescribeDeployment};
use slot::graphql::GraphQLQuery;
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
            Service::Saya => DeploymentService::saya,
        };

        let request_body = DescribeDeployment::build_query(Variables {
            project: self.project.clone(),
            service,
        });

        let user = Credentials::load()?;
        let client = Client::new_with_token(user.access_token);

        let data: ResponseData = client.query(&request_body).await?;

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
                    println!(
                        "  World: {}",
                        config.world.unwrap_or_else(|| "0x0".to_string())
                    );
                    println!("  RPC: {}", config.rpc);
                    if let Some(contracts) = config.contracts {
                        println!("  Contracts: {}", contracts);
                    }
                    if let Some(start_block) = config.start_block {
                        println!("  Start Block: {}", start_block);
                    }
                    if let Some(index_pending) = config.index_pending {
                        println!("  Index Pending: {}", index_pending);
                    }
                    if let Some(index_raw_events) = config.index_raw_events {
                        println!("  Index Raw Events: {}", index_raw_events);
                    }
                    if let Some(index_transactions) = config.index_transactions {
                        println!("  Index Transactions: {}", index_transactions);
                    }
                    println!("\nEndpoints:");
                    println!("  GraphQL: {}", config.graphql);
                    println!("  GRPC: {}", config.grpc);
                }
                KatanaConfig(config) => {
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

        Ok(())
    }
}
