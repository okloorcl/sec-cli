use clap::Args;

#[derive(Args, Debug)]
pub(crate) struct AgentPackArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long, default_value = "10-K")]
    pub(crate) form: String,
    #[arg(long, default_value_t = 1)]
    pub(crate) latest: usize,
    #[arg(long, value_delimiter = ',')]
    pub(crate) sections: Vec<String>,
    #[arg(long, default_value_t = 20000)]
    pub(crate) section_limit_bytes: usize,
    #[arg(long, default_value_t = 4)]
    pub(crate) metrics_latest: usize,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}
