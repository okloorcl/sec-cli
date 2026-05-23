use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct FilingQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub latest: usize,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub include_amends: bool,
}

#[derive(Debug, Clone)]
pub struct DailyIndexQuery {
    pub date: NaiveDate,
    pub form: Option<String>,
    pub company: Option<String>,
    pub limit: Option<usize>,
    pub include_amends: bool,
}

#[derive(Debug, Clone)]
pub struct EftsSearchQuery {
    pub query: String,
    pub ciks: Vec<u64>,
    pub forms: Vec<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct FactQuery {
    pub cik: u64,
    pub concept: String,
    pub form: Option<String>,
    pub unit: Option<String>,
    pub latest: usize,
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub context: usize,
}

#[derive(Debug, Clone)]
pub struct DocumentQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub latest: usize,
    pub include_amends: bool,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct DocumentReadQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub latest: usize,
    pub include_amends: bool,
    pub accession: Option<String>,
    pub filename: Option<String>,
    pub sequence: Option<String>,
    pub primary: bool,
    pub limit_bytes: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct SectionQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub latest: usize,
    pub include_amends: bool,
    pub accession: Option<String>,
    pub item: String,
    pub limit_bytes: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ReportQuery {
    pub cik: u64,
    pub subject: String,
    pub latest: usize,
    pub limit: usize,
    pub include_amends: bool,
    pub limit_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct Form4Query {
    pub cik: u64,
    pub latest: usize,
    pub include_amends: bool,
}

#[derive(Debug, Clone)]
pub struct EightKQuery {
    pub cik: u64,
    pub latest: usize,
    pub include_amends: bool,
    pub item: Option<String>,
    pub limit_bytes: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct EightKExhibitQuery {
    pub cik: u64,
    pub latest: usize,
    pub include_amends: bool,
    pub category: Option<String>,
    pub limit_bytes: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Schedule13Query {
    pub cik: u64,
    pub form: Option<String>,
    pub latest: usize,
    pub include_amends: bool,
    pub limit_bytes: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct InlineXbrlQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub latest: usize,
    pub include_amends: bool,
    pub concept: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct HtmlTableQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub latest: usize,
    pub include_amends: bool,
    pub limit_tables: Option<usize>,
    pub limit_rows: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct XbrlLinkbaseQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub latest: usize,
    pub include_amends: bool,
    pub linkbase: Option<String>,
    pub role: Option<String>,
    pub concept: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct XbrlTreeQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub latest: usize,
    pub include_amends: bool,
    pub role: Option<String>,
    pub concept: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct XbrlCalculationQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub latest: usize,
    pub include_amends: bool,
    pub role: Option<String>,
    pub concept: Option<String>,
    pub unit: Option<String>,
    pub tolerance: f64,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct XbrlStatementQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub latest: usize,
    pub include_amends: bool,
    pub role: Option<String>,
    pub concept: Option<String>,
    pub unit: Option<String>,
    pub tolerance: f64,
    pub values_only: bool,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct StatementStitchQuery {
    pub cik: u64,
    pub statement: String,
    pub unit: Option<String>,
    pub latest: usize,
}

#[derive(Debug, Clone)]
pub struct CompanyReportQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub latest: usize,
    pub include_amends: bool,
    pub topic: Option<String>,
    pub limit_tables: Option<usize>,
    pub limit_rows: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ProxyQuery {
    pub cik: u64,
    pub latest: usize,
    pub include_amends: bool,
    pub limit_rows: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ProspectusQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub latest: usize,
    pub include_amends: bool,
    pub limit_bytes: Option<usize>,
    pub limit_tables: Option<usize>,
    pub limit_rows: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ForeignIssuerQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub latest: usize,
    pub include_amends: bool,
    pub limit_bytes: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct FundDisclosureQuery {
    pub cik: u64,
    pub form: Option<String>,
    pub latest: usize,
    pub include_amends: bool,
    pub limit_holdings: Option<usize>,
    pub limit_bytes: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct StatementQuery {
    pub cik: u64,
    pub statement: String,
    pub form: Option<String>,
    pub unit: Option<String>,
    pub latest: usize,
}

#[derive(Debug, Clone)]
pub struct ThirteenFQuery {
    pub cik: u64,
    pub latest: usize,
    pub include_amends: bool,
}

#[derive(Debug, Clone)]
pub struct ParseQuery {
    pub cik: u64,
    pub form: String,
    pub latest: usize,
    pub include_amends: bool,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputMode {
    Json,
    PrettyJson,
    JsonLines,
    Csv,
    Table,
}
