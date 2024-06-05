use anyhow::{bail, Result};
use clap::{Subcommand, ValueEnum};

use self::{
    katana::{KatanaAccountArgs, KatanaCreateArgs, KatanaForkArgs, KatanaUpdateArgs},
    madara::MadaraCreateArgs,
    torii::{ToriiCreateArgs, ToriiUpdateArgs},
};

use super::create::create_deployment::{
    CreateKatanaConfigInput, CreateMadaraConfigInput, CreateServiceConfigInput, CreateServiceInput,
    CreateToriiConfigInput, DeploymentService,
};

pub mod katana;
mod madara;
mod torii;

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum CreateServiceCommands {
    #[command(about = "Katana deployment.")]
    Katana(KatanaCreateArgs),
    #[command(about = "Torii deployment.")]
    Torii(ToriiCreateArgs),
    #[command(about = "Madara deployment.")]
    Madara(MadaraCreateArgs),
}

impl CreateServiceCommands {
    /// Run the against the local environment.
    pub(crate) async fn run_local(&self) -> Result<()> {
        match self {
            CreateServiceCommands::Katana(args) => args.execute_local().await,
            _ => bail!("Only Katana is supported for local deployments at the moment"),
        }
    }

    pub(crate) fn local(&self) -> bool {
        match &self {
            CreateServiceCommands::Katana(config) => config.local,
            _ => false,
        }
    }

    pub(crate) fn service_input(&self) -> CreateServiceInput {
        match &self {
            CreateServiceCommands::Katana(config) => CreateServiceInput {
                type_: DeploymentService::katana,
                version: config.version.clone(),
                config: Some(CreateServiceConfigInput {
                    torii: None,
                    madara: None,
                    katana: Some(CreateKatanaConfigInput {
                        accounts: config.accounts,
                        gas_price: config.gas_price,
                        block_time: config.block_time,
                        genesis: config.genesis.clone(),
                        disable_fee: config.disable_fee,
                        fork_rpc_url: config.fork_rpc_url.clone(),
                        invoke_max_steps: config.invoke_max_steps,
                        fork_block_number: config.fork_block_number,
                        validate_max_steps: config.validate_max_steps,
                        seed: Some(match &config.seed {
                            Some(seed) => seed.clone(),
                            None => rand::random::<u64>().to_string(),
                        }),
                    }),
                }),
            },

            CreateServiceCommands::Torii(config) => CreateServiceInput {
                type_: DeploymentService::torii,
                version: config.version.clone(),
                config: Some(CreateServiceConfigInput {
                    katana: None,
                    madara: None,
                    torii: Some(CreateToriiConfigInput {
                        rpc: Some(config.rpc.clone().unwrap_or("".to_string())),
                        world: format!("{:#x}", config.world),
                        start_block: Some(config.start_block.unwrap_or(0)),
                        index_pending: config.index_pending,
                    }),
                }),
            },

            CreateServiceCommands::Madara(config) => CreateServiceInput {
                type_: DeploymentService::madara,
                version: config.version.clone(),
                config: Some(CreateServiceConfigInput {
                    katana: None,
                    torii: None,
                    madara: Some(CreateMadaraConfigInput {
                        name: config.name.clone(),
                        base_path: config.base_path.clone(),
                        dev: config.dev.then_some(true),
                        no_grandpa: config.no_grandpa.then_some(true),
                        validator: config.validator.then_some(true),
                        sealing: config.sealing.clone().map(|s| s.to_string()),
                        chain: config.chain.clone().map(|c| c.to_string()),
                    }),
                }),
            },
        }
    }
}

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum UpdateServiceCommands {
    #[command(about = "Katana deployment.")]
    Katana(KatanaUpdateArgs),
    #[command(about = "Torii deployment.")]
    Torii(ToriiUpdateArgs),
}

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum ForkServiceCommands {
    #[command(about = "Katana deployment.")]
    Katana(KatanaForkArgs),
    // #[command(about = "Torii deployment.")]
    // Torii(ToriiUpdateArgs),
}

#[derive(Debug, Subcommand, serde::Serialize)]
#[serde(untagged)]
pub enum KatanaAccountCommands {
    #[command(about = "Katana deployment.")]
    Katana(KatanaAccountArgs),
}

#[derive(Clone, Debug, ValueEnum, serde::Serialize)]
pub enum Service {
    Katana,
    Torii,
    Madara,
}
