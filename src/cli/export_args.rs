use std::path::PathBuf;

use clap::{Args, ValueEnum};

use super::analysis_args::StatementPeriodArg;

#[derive(Args, Debug)]
pub(crate) struct ExportArgs {
    #[arg(long, value_enum)]
    pub(crate) kind: ExportKindArg,
    #[arg(long, value_enum)]
    pub(crate) format: ExportFormatArg,
    #[arg(long)]
    pub(crate) out: PathBuf,
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long)]
    pub(crate) form: Option<String>,
    #[arg(long)]
    pub(crate) concept: Option<String>,
    #[arg(long, default_value = "all")]
    pub(crate) statement: String,
    #[arg(long, value_enum, default_value_t = StatementPeriodArg::Annual)]
    pub(crate) period: StatementPeriodArg,
    #[arg(long)]
    pub(crate) unit: Option<String>,
    #[arg(long, default_value_t = 20)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) include_amends: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum ExportKindArg {
    Filings,
    Facts,
    Statements,
    Stitch,
    Metrics,
    Scores,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum ExportFormatArg {
    Arrow,
    Parquet,
}
