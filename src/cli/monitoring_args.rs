use chrono::NaiveDate;
use clap::Args;

#[derive(Args, Debug)]
pub(crate) struct DailyArgs {
    /// SEC daily index date. Defaults to the latest weekday in UTC.
    #[arg(long)]
    pub(crate) date: Option<NaiveDate>,
    /// Exact form type, for example 8-K, 10-K, 13F-HR, N-PX.
    #[arg(long)]
    pub(crate) form: Option<String>,
    /// Case-insensitive company-name substring filter.
    #[arg(long)]
    pub(crate) company: Option<String>,
    #[arg(long, default_value_t = 100)]
    pub(crate) limit: usize,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Args, Debug)]
pub(crate) struct ServeArgs {
    #[arg(long, default_value = "127.0.0.1")]
    pub(crate) host: String,
    #[arg(long, default_value_t = 8716)]
    pub(crate) port: u16,
}

#[derive(Args, Debug)]
pub(crate) struct McpArgs {}
