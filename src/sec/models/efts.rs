use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct EftsSearchRecord {
    pub query: String,
    pub score: Option<f64>,
    pub cik: Option<u64>,
    pub company: Option<String>,
    pub form: Option<String>,
    pub file_date: Option<String>,
    pub period_ending: Option<String>,
    pub accession: Option<String>,
    pub document: Option<String>,
    pub sequence: Option<u64>,
    pub file_description: Option<String>,
    pub file_type: Option<String>,
    pub source_url: Option<String>,
    pub document_url: Option<String>,
}
