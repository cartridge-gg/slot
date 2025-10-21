use anyhow::Result;
use clap::Subcommand;
use colored::*;
use std::fs;
use strum_macros::Display;
use toml::Value;

use self::{
    accounts::AccountsArgs, create::CreateArgs, delete::DeleteArgs, describe::DescribeArgs,
    list::ListArgs, logs::LogsArgs, update::UpdateArgs,
};
use crate::command::deployments::transfer::TransferArgs;

mod accounts;
mod create;
mod delete;
mod describe;
mod list;
mod logs;
mod services;
mod transfer;
mod update;

pub const CARTRIDGE_BASE_URL: &str = "https://api.cartridge.gg/x";

#[derive(Subcommand, Debug)]
pub enum Deployments {
    #[command(about = "Create a new deployment.")]
    Create(CreateArgs),

    #[command(about = "Delete a deployment.")]
    Delete(DeleteArgs),

    #[command(about = "Update a deployment.")]
    Update(UpdateArgs),

    #[command(about = "Describe a deployment's configuration.")]
    Describe(DescribeArgs),

    #[command(about = "List all deployments.", aliases = ["ls"])]
    List(ListArgs),

    #[command(about = "Transfer a deployment.")]
    Transfer(TransferArgs),

    #[command(about = "Fetch logs for a deployment.")]
    Logs(LogsArgs),

    #[command(about = "Fetch Katana accounts.")]
    Accounts(AccountsArgs),
}

impl Deployments {
    pub async fn run(&self) -> Result<()> {
        match &self {
            Deployments::Create(args) => args.run().await,
            Deployments::Delete(args) => args.run().await,
            Deployments::Update(args) => args.run().await,
            Deployments::Describe(args) => args.run().await,
            Deployments::List(args) => args.run().await,
            Deployments::Transfer(args) => args.run().await,
            Deployments::Logs(args) => args.run().await,
            Deployments::Accounts(args) => args.run().await,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Debug, serde::Serialize, PartialEq, Eq, Hash, Display)]
pub enum Tier {
    Basic,
    Pro,
    Epic,
    Legendary,
    #[clap(skip)]
    Insane,
}

/// Returns the service url for a given project and service.
pub(crate) fn service_url(project: &str, service: &str) -> String {
    format!("{}/{}/{}", CARTRIDGE_BASE_URL, project, service)
}

/// Prints the configuration file for a given project and service.
pub(crate) fn print_config_file(config: &str) {
    println!("\n─────────────── Configuration ───────────────");
    pretty_print_toml(config);
    println!("──────────────────────────────────────────────");
}

/// Pretty prints a TOML string.
pub(crate) fn pretty_print_toml(str: &str) {
    let mut first_line = true;
    for line in str.lines() {
        if line.starts_with("[") {
            // Print section headers.
            if !first_line {
                println!();
                first_line = false;
            }
            println!("{}", line.bright_blue());
        } else if line.contains('=') {
            // Print key-value pairs with keys in green and values.
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let value = parts[1].trim().replace("\"", "");

                println!("{}: {}", key.bright_black(), value);
            } else {
                println!("{}", line);
            }
        } else {
            // Remove line that are empty to have more compact output.
            if line.trim().is_empty() {
                continue;
            }

            // Print other lines normally.
            println!("{}", line);
        }
    }
}

pub(crate) fn warn_checks(config_path: &std::path::Path) -> Result<()> {
    if let Ok(file_content) = fs::read_to_string(config_path) {
        if let Ok(parsed) = toml::from_str::<Value>(&file_content) {
            if parsed.get("db_dir").is_some() {
                println!("⚠️  Warning: 'db_dir' option found in config file but is ignored. Slot is managing and persisting the database.");
            }
            if let Some(indexing) = parsed.get("indexing") {
                if indexing.get("blocks_chunk_size").is_some() {
                    println!("⚠️  Warning: 'blocks_chunk_size' option found in config file but is ignored and overridden in slot.");
                }
                if indexing.get("events_chunk_size").is_some() {
                    println!("⚠️  Warning: 'events_chunk_size' option found in config file but is ignored and overridden in slot.");
                }
            }
            if parsed.get("metrics").is_some() {
                println!("⚠️  Warning: 'metrics' section found in config file but is ignored and overridden in slot. Metrics are always collected and available at /metrics of your slot deployment URL.");
            }
            if let Some(sql) = parsed.get("sql") {
                println!("⚠️  Warning: '[sql]' section found in config file but is ignored and overridden in slot. Database configuration is managed by slot.");
                if sql.get("cache_size").is_some() {
                    println!("⚠️  Warning: 'cache_size' option found in config file but is ignored and overridden in slot.");
                }
                if sql.get("page_size").is_some() {
                    println!("⚠️  Warning: 'page_size' option found in config file but is ignored and overridden in slot.");
                }
            }
            if let Some(erc) = parsed.get("erc") {
                println!("⚠️  Warning: '[erc]' section found in config file but is ignored and overridden in slot. ERC configuration is managed by slot.");
                if erc.get("max_metadata_tasks").is_some() {
                    println!("⚠️  Warning: 'max_metadata_tasks' option found in config file but is ignored and overridden in slot.");
                }
            }
        }
    }
    Ok(())
}
