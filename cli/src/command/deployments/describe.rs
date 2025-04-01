#![allow(clippy::enum_variant_names)]

use crate::command::deployments::print_config_file;

use super::services::Service;
use anyhow::Result;
use clap::Args;
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
            println!("Version: {}", deployment.version);

            if deployment.deprecated.unwrap_or(false) {
                println!("This deployment is deprecated and immutable.");
                println!("Please delete it and re-create. Note that this will reset the storage.");
            }

            println!(
                "Branch: {}",
                deployment.branch.unwrap_or_else(|| String::from("Default"))
            );
            println!("Tier: {:?}", deployment.tier);

            println!(
                "Url: {}",
                super::service_url(&deployment.project, &self.service.to_string())
            );

            // convert config of type String to &str
            print_config_file(&deployment.config.config_file);

            if deployment.error.is_some() {
                println!("\n─────────────── ERROR INFO ───────────────");
                println!("Error: {}", deployment.error.unwrap());
                println!("\n─────────────── ERROR INFO ───────────────");
            }
        }

        Ok(())
    }
}
