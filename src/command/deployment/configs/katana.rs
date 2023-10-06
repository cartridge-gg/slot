use clap::Args;

#[derive(Debug, Args, serde::Serialize)]
#[command(next_help_heading = "Katana options")]
pub struct Katana {
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

    #[arg(long, short, value_name = "total_accounts")]
    #[arg(help = "Total accounts.")]
    pub total_accounts: Option<i64>,
}
