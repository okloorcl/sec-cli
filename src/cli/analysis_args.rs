use clap::{Args, ValueEnum};

#[derive(Args, Debug)]
pub(crate) struct StatementsArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long, default_value = "all")]
    pub(crate) statement: String,
    #[arg(long, value_enum, default_value_t = StatementPeriodArg::Annual)]
    pub(crate) period: StatementPeriodArg,
    #[arg(long)]
    pub(crate) unit: Option<String>,
    #[arg(long, default_value_t = 4)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Args, Debug)]
pub(crate) struct MetricsArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long, value_enum, default_value_t = StatementPeriodArg::Annual)]
    pub(crate) period: StatementPeriodArg,
    #[arg(long)]
    pub(crate) unit: Option<String>,
    #[arg(long, default_value_t = 4)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Args, Debug)]
pub(crate) struct XbrlLinkbaseArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long, default_value = "10-K")]
    pub(crate) form: String,
    #[arg(long)]
    pub(crate) linkbase: Option<String>,
    #[arg(long)]
    pub(crate) role: Option<String>,
    #[arg(long)]
    pub(crate) concept: Option<String>,
    #[arg(long, default_value_t = 1)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long, default_value_t = 200)]
    pub(crate) limit: usize,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Clone, Debug, ValueEnum)]
pub(crate) enum StatementPeriodArg {
    Annual,
    Quarterly,
    All,
}
