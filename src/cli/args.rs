use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand, ValueEnum};

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
    /// Build standardized financial statement rows from SEC CompanyFacts.
    Statements(StatementsArgs),
    /// Stream Inline XBRL facts from primary filing HTML.
    Ixbrl(InlineXbrlArgs),
    /// Extract HTML tables from primary filing documents.
    Tables(TablesArgs),
    /// Parse DEF 14A proxy statement governance and compensation signals.
    Proxy(ProxyArgs),
    /// Parse S-1/F-1/424B registration statements and prospectuses.
    Prospectus(ProspectusArgs),
    /// Search filing submission text and return source-backed snippets.
    Search(SearchArgs),
    /// Extract a named 10-K/10-Q section from a primary filing document.
    Section(SectionArgs),
    /// Generate a source-backed Markdown report.
    Report(ReportArgs),
    /// Resolve an investor/fund/person name to SEC 13F manager candidates.
    #[command(name = "resolve", alias = "investor")]
    Resolve(ResolveArgs),
    /// List documents and attachments inside SEC complete submissions.
    Docs(DocsArgs),
    /// Read one document from a complete SEC submission.
    Doc(DocArgs),
    /// Parse Form 4 insider ownership transactions.
    Form4(Form4Args),
    /// Summarize Form 4 issuer, owners, signatures, footnotes, and net transactions.
    #[command(name = "form4-summary")]
    Form4Summary(Form4Args),
    /// Parse Form 8-K current-report events by item.
    #[command(name = "8k")]
    EightK(EightKArgs),
    /// Parse Schedule 13D/13G beneficial ownership reports.
    #[command(name = "13d", aliases = ["13g", "schedule13"])]
    Schedule13(Schedule13Args),
    /// Parse 13F institutional holdings information tables.
    #[command(name = "13f")]
    ThirteenF(ThirteenFArgs),
    /// Parse and aggregate 13F holdings by CUSIP/class/put-call.
    #[command(name = "13f-aggregate")]
    ThirteenFAggregate(ThirteenFArgs),
    /// Compare the latest two 13F portfolios and classify position changes.
    #[command(name = "13f-diff")]
    ThirteenFDiff(ThirteenFArgs),
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

#[derive(Clone, Debug, ValueEnum)]
pub(crate) enum StatementPeriodArg {
    Annual,
    Quarterly,
    All,
}

#[derive(Args, Debug)]
pub(crate) struct InlineXbrlArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long, default_value = "10-K")]
    pub(crate) form: String,
    #[arg(long)]
    pub(crate) concept: Option<String>,
    #[arg(long, default_value_t = 1)]
    pub(crate) latest: usize,
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
pub(crate) struct TablesArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long, default_value = "10-K")]
    pub(crate) form: String,
    #[arg(long, default_value_t = 1)]
    pub(crate) latest: usize,
    #[arg(long, default_value_t = 20)]
    pub(crate) limit_tables: usize,
    #[arg(long, default_value_t = 25)]
    pub(crate) limit_rows: usize,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Args, Debug)]
pub(crate) struct ProxyArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long, default_value_t = 1)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long, default_value_t = 12)]
    pub(crate) limit_rows: usize,
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
pub(crate) struct SectionArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long, default_value = "10-K")]
    pub(crate) form: String,
    #[arg(long)]
    pub(crate) item: String,
    #[arg(long, default_value_t = 1)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) accession: Option<String>,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long)]
    pub(crate) limit_bytes: Option<usize>,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Args, Debug)]
pub(crate) struct ReportArgs {
    #[arg(long, conflicts_with_all = ["cik", "investor", "manager"])]
    pub(crate) ticker: Option<String>,
    #[arg(long, conflicts_with_all = ["ticker", "investor", "manager"])]
    pub(crate) cik: Option<u64>,
    #[arg(long, conflicts_with_all = ["ticker", "cik", "manager"])]
    pub(crate) investor: Option<String>,
    #[arg(long, conflicts_with_all = ["ticker", "cik", "investor"])]
    pub(crate) manager: Option<String>,
    #[arg(long, value_enum)]
    pub(crate) kind: ReportKindArg,
    #[arg(long, default_value_t = 5)]
    pub(crate) latest: usize,
    #[arg(long, default_value_t = 10)]
    pub(crate) limit: usize,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long, default_value_t = 4000)]
    pub(crate) limit_bytes: usize,
}

#[derive(Clone, Debug, ValueEnum)]
pub(crate) enum ReportKindArg {
    Insider,
    Portfolio,
    Risk,
}

#[derive(Args, Debug)]
pub(crate) struct ResolveArgs {
    #[arg(long, conflicts_with_all = ["cik", "manager"])]
    pub(crate) query: Option<String>,
    #[arg(long, conflicts_with_all = ["query", "manager"])]
    pub(crate) cik: Option<u64>,
    #[arg(long, conflicts_with_all = ["query", "cik"])]
    pub(crate) manager: Option<String>,
    #[arg(long)]
    pub(crate) no_verify: bool,
    #[arg(long, value_enum)]
    pub(crate) llm_provider: Option<LlmProviderArg>,
    #[arg(long)]
    pub(crate) llm_base_url: Option<String>,
    #[arg(long)]
    pub(crate) llm_model: Option<String>,
    #[arg(long)]
    pub(crate) llm_api_key_env: Option<String>,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Clone, Debug, ValueEnum)]
pub(crate) enum LlmProviderArg {
    Openai,
    Anthropic,
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
pub(crate) struct EightKArgs {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long)]
    pub(crate) item: Option<String>,
    #[arg(long, default_value_t = 5)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) limit: Option<usize>,
    #[arg(long)]
    pub(crate) limit_bytes: Option<usize>,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Args, Debug)]
pub(crate) struct Schedule13Args {
    #[arg(long, conflicts_with = "cik")]
    pub(crate) ticker: Option<String>,
    #[arg(long)]
    pub(crate) cik: Option<u64>,
    #[arg(long)]
    pub(crate) form: Option<String>,
    #[arg(long, default_value_t = 5)]
    pub(crate) latest: usize,
    #[arg(long)]
    pub(crate) include_amends: bool,
    #[arg(long)]
    pub(crate) limit_bytes: Option<usize>,
    #[arg(long)]
    pub(crate) jsonl: bool,
    #[arg(long)]
    pub(crate) pretty: bool,
}

#[derive(Args, Debug)]
pub(crate) struct ThirteenFArgs {
    #[arg(long, conflicts_with_all = ["cik", "investor", "manager"])]
    pub(crate) ticker: Option<String>,
    #[arg(long, conflicts_with_all = ["ticker", "investor", "manager"])]
    pub(crate) cik: Option<u64>,
    #[arg(long, conflicts_with_all = ["ticker", "cik", "manager"])]
    pub(crate) investor: Option<String>,
    #[arg(long, conflicts_with_all = ["ticker", "cik", "investor"])]
    pub(crate) manager: Option<String>,
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
