use clap::Args;

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Katana create options")]
pub struct KatanaCreateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,

    #[arg(long, short, value_name = "block_time")]
    #[arg(help = "Block time.")]
    pub block_time: Option<i64>,

    #[arg(long, value_name = "fork_rpc_url")]
    #[arg(help = "Fork RPC URL.")]
    pub fork_rpc_url: Option<String>,

    #[arg(long, value_name = "fork_block_number")]
    #[arg(help = "Fork block number.")]
    pub fork_block_number: Option<u64>,

    #[arg(long, short, value_name = "seed")]
    #[arg(help = "Seed.")]
    pub seed: Option<String>,

    #[arg(long, short, value_name = "accounts")]
    #[arg(help = "Accounts.")]
    pub accounts: Option<i64>,

    #[arg(long, value_name = "invoke_max_steps")]
    #[arg(help = "Invoke Max Steps.")]
    pub invoke_max_steps: Option<u64>,

    #[arg(long, value_name = "validate_max_steps")]
    #[arg(help = "Validate Max Steps.")]
    pub validate_max_steps: Option<u64>,

    #[arg(long, value_name = "disable_fee")]
    #[arg(help = "Disable Fee.")]
    pub disable_fee: Option<bool>,

    #[arg(long, value_name = "gas_price")]
    #[arg(help = "Gas Price.")]
    pub gas_price: Option<u64>,
}

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Katana update options")]
pub struct KatanaUpdateArgs {
    #[arg(long, short, value_name = "version")]
    #[arg(help = "Service version to use.")]
    pub version: Option<String>,

    #[arg(long, short, value_name = "block_time")]
    #[arg(help = "Block time.")]
    pub block_time: Option<i64>,

    #[arg(long, value_name = "invoke_max_steps")]
    #[arg(help = "Invoke Max Steps.")]
    pub invoke_max_steps: Option<u64>,

    #[arg(long, value_name = "validate_max_steps")]
    #[arg(help = "Validate Max Steps.")]
    pub validate_max_steps: Option<u64>,

    #[arg(long, value_name = "disable_fee")]
    #[arg(help = "Disable Fee.")]
    pub disable_fee: Option<bool>,

    #[arg(long, value_name = "gas_price")]
    #[arg(help = "Gas Price.")]
    pub gas_price: Option<u64>,
}
