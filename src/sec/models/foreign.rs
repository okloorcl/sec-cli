use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ForeignIssuerRecord {
    pub accession: String,
    pub cik: u64,
    pub company: String,
    pub form: String,
    pub filing_date: String,
    pub report_type: String,
    pub is_amendment: bool,
    pub exchange: Option<String>,
    pub ticker_or_symbol: Option<String>,
    pub auditor: Option<String>,
    pub event_signals: Vec<String>,
    pub risk_factors: Option<ForeignExcerptRecord>,
    pub business: Option<ForeignExcerptRecord>,
    pub operating_review: Option<ForeignExcerptRecord>,
    pub controls: Option<ForeignExcerptRecord>,
    pub financial_statements: Option<ForeignExcerptRecord>,
    pub document: Option<String>,
    pub document_sequence: Option<String>,
    pub document_description: Option<String>,
    pub document_url: Option<String>,
    pub source_url: String,
}

#[derive(Debug, Serialize)]
pub struct ForeignExcerptRecord {
    pub title: String,
    pub content: String,
    pub byte_length: usize,
    pub returned_bytes: usize,
    pub truncated: bool,
}
