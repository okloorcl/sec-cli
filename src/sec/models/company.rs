use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CompanyReportRecord {
    pub accession: String,
    pub cik: u64,
    pub company: String,
    pub form: String,
    pub filing_date: String,
    pub report_date: Option<String>,
    pub topics: Vec<CompanyTopicTableRecord>,
    pub matched_table_count: usize,
    pub scanned_table_count: usize,
    pub document: Option<String>,
    pub document_url: Option<String>,
    pub source_url: String,
}

#[derive(Debug, Serialize)]
pub struct CompanyTopicTableRecord {
    pub topic: String,
    pub confidence: f64,
    pub table_index: usize,
    pub title_hint: Option<String>,
    pub row_count: usize,
    pub column_count: usize,
    pub returned_rows: usize,
    pub truncated: bool,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}
