use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize, Clone)]
pub struct FilingRecord {
    pub accession: String,
    pub cik: u64,
    pub company: String,
    pub form: String,
    pub filing_date: String,
    pub report_date: Option<String>,
    pub primary_document: Option<String>,
    pub primary_doc_description: Option<String>,
    pub is_xbrl: Option<bool>,
    pub is_inline_xbrl: Option<bool>,
    pub source_url: String,
    pub text_url: String,
}

#[derive(Debug, Serialize)]
pub struct FactRecord {
    pub concept: String,
    pub taxonomy: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub value: Value,
    pub unit: String,
    pub fy: Option<i64>,
    pub fp: Option<String>,
    pub form: Option<String>,
    pub filed: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub frame: Option<String>,
    pub accession: Option<String>,
    pub source_url: Option<String>,
    pub fact_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchMatch {
    pub accession: String,
    pub cik: u64,
    pub company: String,
    pub form: String,
    pub filing_date: String,
    pub query: String,
    pub document: String,
    pub section: Option<String>,
    pub offset: usize,
    pub snippet: String,
    pub source_url: String,
}

#[derive(Debug, Serialize)]
pub struct DocumentRecord {
    pub accession: String,
    pub cik: u64,
    pub company: String,
    pub form: String,
    pub filing_date: String,
    pub document_type: Option<String>,
    pub sequence: Option<String>,
    pub filename: Option<String>,
    pub description: Option<String>,
    pub content_type: String,
    pub byte_length: usize,
    pub is_primary: bool,
    pub source_url: String,
    pub document_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Form4TransactionRecord {
    pub accession: String,
    pub cik: u64,
    pub company: String,
    pub filing_date: String,
    pub issuer: Option<String>,
    pub issuer_cik: Option<String>,
    pub reporting_owner: Option<String>,
    pub owner_cik: Option<String>,
    pub is_director: Option<bool>,
    pub is_officer: Option<bool>,
    pub is_ten_percent_owner: Option<bool>,
    pub officer_title: Option<String>,
    pub transaction_date: Option<String>,
    pub transaction_code: Option<String>,
    pub acquired_disposed: Option<String>,
    pub transaction_type: Option<String>,
    pub security_title: Option<String>,
    pub shares: Option<f64>,
    pub price: Option<f64>,
    pub value: Option<f64>,
    pub shares_owned_after: Option<f64>,
    pub direct_or_indirect: Option<String>,
    pub derivative: bool,
    pub document: Option<String>,
    pub document_sequence: Option<String>,
    pub document_description: Option<String>,
    pub source_url: String,
}

#[derive(Debug, Serialize)]
pub struct ThirteenFHoldingRecord {
    pub accession: String,
    pub cik: u64,
    pub manager: String,
    pub filing_date: String,
    pub report_date: Option<String>,
    pub issuer: Option<String>,
    pub class: Option<String>,
    pub cusip: Option<String>,
    pub value_reported: Option<u64>,
    pub value_scale: String,
    pub value_usd: Option<u64>,
    pub shares: Option<f64>,
    pub share_type: Option<String>,
    pub put_call: Option<String>,
    pub investment_discretion: Option<String>,
    pub other_manager: Option<String>,
    pub voting_sole: Option<u64>,
    pub voting_shared: Option<u64>,
    pub voting_none: Option<u64>,
    pub document: Option<String>,
    pub document_sequence: Option<String>,
    pub document_description: Option<String>,
    pub source_url: String,
}

#[derive(Debug, Serialize)]
pub struct ThirteenFReportRecord {
    pub accession: String,
    pub cik: u64,
    pub manager: String,
    pub filing_date: String,
    pub report_date: Option<String>,
    pub report_calendar_or_quarter: Option<String>,
    pub report_type: Option<String>,
    pub other_included_managers_count: Option<u64>,
    pub total_holdings_reported: Option<u64>,
    pub total_value_reported: Option<u64>,
    pub value_scale: String,
    pub total_value_usd: Option<u64>,
    pub filing_manager_name: Option<String>,
    pub filing_manager_city: Option<String>,
    pub filing_manager_state_or_country: Option<String>,
    pub filing_manager_zipcode: Option<String>,
    pub signature_name: Option<String>,
    pub signature_title: Option<String>,
    pub signature_phone: Option<String>,
    pub signature_city: Option<String>,
    pub signature_state_or_country: Option<String>,
    pub signature_date: Option<String>,
    pub additional_information: Option<String>,
    pub other_managers: Vec<ThirteenFOtherManagerRecord>,
    pub document: Option<String>,
    pub document_sequence: Option<String>,
    pub document_description: Option<String>,
    pub source_url: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ThirteenFOtherManagerRecord {
    pub sequence_number: Option<u64>,
    pub cik: Option<String>,
    pub name: Option<String>,
    pub form13f_file_number: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "record", rename_all = "snake_case")]
pub enum ParsedRecord {
    Form4Transaction(Form4TransactionRecord),
    ThirteenfHolding(ThirteenFHoldingRecord),
}
