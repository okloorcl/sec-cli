use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DailyFilingRecord {
    pub cik: u64,
    pub company: String,
    pub form: String,
    pub filing_date: String,
    pub accession: Option<String>,
    pub filename: String,
    pub text_url: String,
    pub source_url: Option<String>,
}
