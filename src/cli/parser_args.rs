use clap::Args;

#[derive(Args, Debug)]
pub(crate) struct ParseArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long)]
    pub(crate) form: String,
    #[arg(long, default_value_t = 1)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long)]
    pub(crate) limit: Option<usize>,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Args, Debug)]
pub(crate) struct OutputArgs {
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}
