use clap::Args;

#[derive(Args, Debug)]
pub(crate) struct CompanyReportArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long, default_value = "10-K")]
    pub(crate) form: String,
    #[arg(long)]
    pub(crate) topic: Option<String>,
    #[arg(long, default_value_t = 1)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long, default_value_t = 20)]
    pub(crate) limit_tables: usize,
    #[arg(long, default_value_t = 15)]
    pub(crate) limit_rows: usize,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Args, Debug)]
pub(crate) struct ProspectusArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long, default_value = "all")]
    pub(crate) form: String,
    #[arg(long, default_value_t = 1)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long, default_value_t = 1200)]
    pub(crate) limit_bytes: usize,
    #[arg(long, default_value_t = 8)]
    pub(crate) limit_tables: usize,
    #[arg(long, default_value_t = 8)]
    pub(crate) limit_rows: usize,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Args, Debug)]
pub(crate) struct ForeignArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long, default_value = "all")]
    pub(crate) form: String,
    #[arg(long, default_value_t = 1)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long, default_value_t = 1200)]
    pub(crate) limit_bytes: usize,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Args, Debug)]
pub(crate) struct FundArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long, default_value = "all")]
    pub(crate) form: String,
    #[arg(long, default_value_t = 1)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long, default_value_t = 25)]
    pub(crate) limit_holdings: usize,
    #[arg(long, default_value_t = 1200)]
    pub(crate) limit_bytes: usize,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}
