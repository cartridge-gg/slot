#![allow(non_camel_case_types)]
use anyhow::Result;
use clap::Args;
use graphql_client::{GraphQLQuery, Response};

use crate::api::ApiClient;

use self::create_deployment::ServiceInput;

use super::configs::CreateCommands;

#[allow(clippy::upper_case_acronyms)]
type JSON = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/command/deployment/create.graphql",
    response_derives = "Debug"
)]
pub struct CreateDeployment;

#[derive(clap::ValueEnum, Clone, Debug, serde::Serialize)]
pub enum Tier {
    Basic,
}

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Create deployment options")]
pub struct CreateOptions {
    #[arg(short, long = "name")]
    #[arg(help = "The name of the project.")]
    pub name: String,
    #[arg(short, long, default_value = "basic")]
    #[arg(value_name = "tier")]
    #[arg(help = "Deployment tier.")]
    pub tier: Tier,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Create options")]
pub struct CreateArgs {
    #[command(flatten)]
    options: CreateOptions,

    #[command(subcommand)]
    create_commands: CreateCommands,
}

impl CreateArgs {
    pub async fn run(&self) -> Result<()> {
        let config = serde_json::to_string(&self.create_commands)?;
        let service_type = match &self.create_commands {
            CreateCommands::Katana(_) => create_deployment::DeploymentService::katana,
            CreateCommands::Torii(_) => create_deployment::DeploymentService::torii,
        };
        let tier = match &self.options.tier {
            Tier::Basic => create_deployment::DeploymentTier::basic,
        };
        let request_body = CreateDeployment::build_query(create_deployment::Variables {
            name: self.options.name.clone(),
            tier,
            service: ServiceInput {
                type_: service_type,
                version: None,
            },
            config,
        });

        let client = ApiClient::new();
        let res: Response<create_deployment::ResponseData> = client.post(&request_body).await?;
        if res.errors.is_some() {}

        println!("{:#?}", res.errors);
        Ok(())
    }
}
