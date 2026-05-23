use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "sec")]
#[command(bin_name = "sec")]
#[command(version)]
#[command(about = "Agent-ready SEC EDGAR parser and query CLI, powered by Rust.")]
pub(crate) struct Cli {
    #[arg(long, global = true)]
    pub(crate) identity: Option<String>,

    #[arg(long, global = true)]
    pub(crate) cache_dir: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    /// Find SEC filings by ticker/CIK, form, and date.
    Filings(FilingsArgs),
    /// Query SEC company facts by concept alias or XBRL concept name.
    Facts(FactsArgs),
    /// Search filing submission text and return source-backed snippets.
    Search(SearchArgs),
    /// List documents and attachments inside SEC complete submissions.
    Docs(DocsArgs),
    /// Read one document from a complete SEC submission.
    Doc(DocArgs),
    /// Parse Form 4 insider ownership transactions.
    Form4(Form4Args),
    /// Summarize Form 4 issuer, owners, signatures, footnotes, and net transactions.
    #[command(name = "form4-summary")]
    Form4Summary(Form4Args),
    /// Parse 13F institutional holdings information tables.
    #[command(name = "13f")]
    ThirteenF(ThirteenFArgs),
    /// Parse and aggregate 13F holdings by CUSIP/class/put-call.
    #[command(name = "13f-aggregate")]
    ThirteenFAggregate(ThirteenFArgs),
    /// Parse 13F cover, summary, signature, and manager metadata.
    #[command(name = "13f-summary")]
    ThirteenFSummary(ThirteenFArgs),
    /// Parse a supported filing form through the unified parser pipeline.
    Parse(ParseArgs),
    /// List supported structured form parsers.
    Forms(OutputArgs),
}

#[derive(Args, Debug)]
pub(crate) struct FilingsArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long)]
    pub(crate) form: Option<String>,
    #[arg(long, default_value_t = 10)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) from: Option<NaiveDate>,
    #[arg(long)]
    pub(crate) to: Option<NaiveDate>,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Args, Debug)]
pub(crate) struct FactsArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long)]
    pub(crate) concept: String,
    #[arg(long)]
    pub(crate) form: Option<String>,
    #[arg(long)]
    pub(crate) unit: Option<String>,
    #[arg(long, default_value_t = 20)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Args, Debug)]
pub(crate) struct SearchArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long)]
    pub(crate) form: Option<String>,
    #[arg(long)]
    pub(crate) query: String,
    #[arg(long, default_value_t = 1)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long, default_value_t = 220)]
    pub(crate) context: usize,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Args, Debug)]
pub(crate) struct DocsArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long)]
    pub(crate) form: Option<String>,
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
pub(crate) struct DocArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long)]
    pub(crate) form: Option<String>,
    #[arg(long, default_value_t = 1)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) accession: Option<String>,
    #[arg(long)]
    pub(crate) filename: Option<String>,
    #[arg(long)]
    pub(crate) sequence: Option<String>,
    #[arg(long)]
    pub(crate) primary: bool,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long)]
    pub(crate) limit_bytes: Option<usize>,
    #[arg(long, conflicts_with_all = ["text", "jsonl", "pretty"])]
    pub(crate) raw: bool,
    #[arg(long, conflicts_with_all = ["raw", "jsonl", "pretty"])]
    pub(crate) text: bool,
    #[arg(long, conflicts_with_all = ["raw", "text"])]
    pub(crate) jsonl: bool,
    #[arg(long, conflicts_with_all = ["raw", "text"])]
    pub(crate) pretty: bool,
}

#[derive(Args, Debug)]
pub(crate) struct Form4Args {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long, default_value_t = 5)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) limit: Option<usize>,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Args, Debug)]
pub(crate) struct ThirteenFArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long, default_value_t = 1)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) limit: Option<usize>,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

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
