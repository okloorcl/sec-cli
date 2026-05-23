use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ProxyStatementRecord {
    pub accession: String,
    pub cik: u64,
    pub company: String,
    pub form: String,
    pub filing_date: String,
    pub meeting_date: Option<String>,
    pub meeting_time: Option<String>,
    pub meeting_site: Option<String>,
    pub record_date: Option<String>,
    pub materials_available_date: Option<String>,
    pub proposals: Vec<ProxyProposalRecord>,
    pub director_nominees: Vec<String>,
    pub auditor: Option<String>,
    pub named_executive_officers: Vec<String>,
    pub summary_compensation_table: Option<ProxyTableRecord>,
    pub document: Option<String>,
    pub document_sequence: Option<String>,
    pub document_description: Option<String>,
    pub document_url: Option<String>,
    pub source_url: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProxyProposalRecord {
    pub proposal_number: u64,
    pub title: String,
    pub category: String,
    pub board_recommendation: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProxyTableRecord {
    pub table_index: usize,
    pub title_hint: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
    pub column_count: usize,
    pub truncated: bool,
}
