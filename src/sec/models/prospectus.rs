use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ProspectusRecord {
    pub accession: String,
    pub cik: u64,
    pub company: String,
    pub form: String,
    pub filing_date: String,
    pub prospectus_type: String,
    pub is_amendment: bool,
    pub is_ipo_related: bool,
    pub securities_offered: Vec<String>,
    pub proposed_ticker: Option<String>,
    pub exchange: Option<String>,
    pub price_range: Option<String>,
    pub shares_offered: Option<String>,
    pub offering_amount: Option<String>,
    pub underwriters: Vec<String>,
    pub auditor: Option<String>,
    pub use_of_proceeds: Option<ProspectusExcerptRecord>,
    pub risk_factors: Option<ProspectusExcerptRecord>,
    pub business: Option<ProspectusExcerptRecord>,
    pub dilution: Option<ProspectusExcerptRecord>,
    pub tables: Vec<ProspectusTableRecord>,
    pub document: Option<String>,
    pub document_sequence: Option<String>,
    pub document_description: Option<String>,
    pub document_url: Option<String>,
    pub source_url: String,
}

#[derive(Debug, Serialize)]
pub struct ProspectusExcerptRecord {
    pub title: String,
    pub content: String,
    pub byte_length: usize,
    pub returned_bytes: usize,
    pub truncated: bool,
}

#[derive(Debug, Serialize)]
pub struct ProspectusTableRecord {
    pub table_index: usize,
    pub title_hint: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
    pub column_count: usize,
    pub truncated: bool,
}
