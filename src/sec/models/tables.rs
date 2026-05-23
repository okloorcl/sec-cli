use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HtmlTableRecord {
    pub accession: String,
    pub cik: u64,
    pub company: String,
    pub form: String,
    pub filing_date: String,
    pub report_date: Option<String>,
    pub table_index: usize,
    pub title_hint: Option<String>,
    pub row_count: usize,
    pub column_count: usize,
    pub returned_rows: usize,
    pub truncated: bool,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub document: Option<String>,
    pub document_sequence: Option<String>,
    pub document_description: Option<String>,
    pub document_url: Option<String>,
    pub source_url: String,
}
