use core::fmt;

use clap::{Args, ValueEnum};
use serde::Serialize;

#[derive(Debug, Args, Serialize)]
#[command(next_help_heading = "Madara create options")]
pub struct MadaraCreateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,

    #[arg(long, value_name = "dev")]
    #[arg(
        help = "Specify the development chain. This flag sets `--chain=dev`, `--force-authoring`, `--rpc-cors=all`, `--alice`, and `--tmp` flags, unless explicitly overridden"
    )]
    pub dev: bool,

    #[arg(long, value_name = "name")]
    #[arg(help = "The human-readable name for this node. It's used as network node name.")]
    pub name: Option<String>,

    #[arg(long, value_name = "validator")]
    #[arg(
        help = "Enable validator mode. The node will be started with the authority role and actively participate in any consensus task that it can (e.g. depending on availability of local keys)"
    )]
    pub validator: bool,

    #[arg(long, value_name = "no_grandpa")]
    #[arg(
        help = "Disable GRANDPA voter when running in validator mode, otherwise disable the GRANDPA observer"
    )]
    pub no_grandpa: bool,

    #[arg(long, value_name = "chain")]
    #[arg(help = "Specify the chain specification. It can be one of the predefined ones")]
    pub chain: Option<ChainOption>,

    #[arg(long, value_name = "base_path")]
    #[arg(help = "Specify custom base path")]
    pub base_path: Option<String>,

    #[arg(long, value_name = "chain")]
    #[arg(help = "Choose sealing method")]
    pub sealing: Option<SealingMethod>,

    #[arg(long, value_name = "from_remote")]
    #[arg(
        help = "Prior to starting the node, the setup cmd will be executed using this value. If none is provided, setup will use the default config."
    )]
    pub from_remote: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize)]
pub enum SealingMethod {
    Manual,
    Instant,
    InstantFinality,
}

impl fmt::Display for SealingMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SealingMethod::Manual => write!(f, "manual"),
            SealingMethod::Instant => write!(f, "instant"),
            SealingMethod::InstantFinality => write!(f, "instant-finality"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize)]
pub enum ChainOption {
    Dev,
    Local,
    Staging,
}

impl fmt::Display for ChainOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChainOption::Dev => write!(f, "dev"),
            ChainOption::Local => write!(f, "local"),
            ChainOption::Staging => write!(f, "staging"),
        }
    }
}
