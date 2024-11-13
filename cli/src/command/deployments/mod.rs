use anyhow::Result;
use clap::Subcommand;
use colored::*;

use self::{
    accounts::AccountsArgs, create::CreateArgs, delete::DeleteArgs, describe::DescribeArgs,
    fork::ForkArgs, list::ListArgs, logs::LogsArgs, update::UpdateArgs,
};

mod accounts;
mod create;
mod delete;
mod describe;
mod fork;
mod list;
mod logs;
mod services;
mod update;

#[derive(Subcommand, Debug)]
pub enum Deployments {
    #[command(about = "Create a new deployment.")]
    Create(CreateArgs),
    #[command(about = "Delete a deployment.")]
    Delete(DeleteArgs),
    #[command(about = "Update a deployment.")]
    Update(UpdateArgs),
    #[command(about = "Fork a deployment.")]
    Fork(ForkArgs),
    #[command(about = "Describe a deployment's configuration.")]
    Describe(DescribeArgs),
    #[command(about = "List all deployments.", aliases = ["ls"])]
    List(ListArgs),
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
            Deployments::Fork(args) => args.run().await,
            Deployments::Describe(args) => args.run().await,
            Deployments::List(args) => args.run().await,
            Deployments::Logs(args) => args.run().await,
            Deployments::Accounts(args) => args.run().await,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Debug, serde::Serialize)]
pub enum Tier {
    Basic,
    Common,
    Rare,
    Epic,
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
