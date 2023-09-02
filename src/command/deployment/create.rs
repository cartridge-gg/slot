#![allow(non_camel_case_types)]
use anyhow::Result;
use clap::Args;
use graphql_client::{GraphQLQuery, Response};

use self::create_deployment::ServiceInput;

use super::configs::CreateCommands;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/command/deployment/create.graphql",
    response_derives = "Debug"
)]
pub struct CreateDeployment;

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Create deployment options")]
pub struct CreateOptions {
    #[arg(long = "name")]
    #[arg(help = "The name of the project.")]
    pub name: String,
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
            CreateCommands::Madara(_) => create_deployment::DeploymentService::madara,
            CreateCommands::Torii(_) => create_deployment::DeploymentService::torii,
        };
        let request_body = CreateDeployment::build_query(create_deployment::Variables {
            name: self.options.name.clone(),
            service: ServiceInput {
                type_: service_type,
                version: None,
            },
            config,
        });

        let client = reqwest::Client::new();
        let res = client.post("/graphql").json(&request_body).send().await?;
        let response_body: Response<create_deployment::ResponseData> = res.json().await?;
        println!("{:#?}", response_body);
        Ok(())
    }
}
