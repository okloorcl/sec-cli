use chrono::NaiveDate;
use clap::{Args, ValueEnum};
use sec_cli::sec::OutputMode;

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
pub(crate) struct EftsArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long)]
    pub(crate) query: String,
    #[arg(long, value_delimiter = ',')]
    pub(crate) form: Vec<String>,
    #[arg(long)]
    pub(crate) from: Option<NaiveDate>,
    #[arg(long)]
    pub(crate) to: Option<NaiveDate>,
    #[arg(long, default_value_t = 20)]
    pub(crate) limit: usize,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum OutputArg {
    Json,
    Pretty,
    Jsonl,
    Csv,
    Table,
}

impl From<OutputArg> for OutputMode {
    fn from(output: OutputArg) -> Self {
        match output {
            OutputArg::Json => OutputMode::Json,
            OutputArg::Pretty => OutputMode::PrettyJson,
            OutputArg::Jsonl => OutputMode::JsonLines,
            OutputArg::Csv => OutputMode::Csv,
            OutputArg::Table => OutputMode::Table,
        }
    }
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
