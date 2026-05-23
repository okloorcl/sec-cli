use std::path::PathBuf;

use clap::Args;

#[derive(Args, Debug)]
pub(crate) struct ArchiveArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long)]
    pub(crate) form: Option<String>,
    #[arg(long, default_value_t = 10)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long)]
    pub(crate) primary_only: bool,
    #[arg(long)]
    pub(crate) limit_bytes: Option<usize>,
    #[arg(long)]
    pub(crate) out_dir: PathBuf,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}
