use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FundDisclosureRecord {
    pub accession: String,
    pub cik: u64,
    pub company: String,
    pub form: String,
    pub filing_date: String,
    pub disclosure_type: String,
    pub is_amendment: bool,
    pub registrant_name: Option<String>,
    pub series_name: Option<String>,
    pub class_name: Option<String>,
    pub period_end: Option<String>,
    pub fiscal_year_end: Option<String>,
    pub total_assets: Option<f64>,
    pub total_liabilities: Option<f64>,
    pub net_assets: Option<f64>,
    pub holdings_count: usize,
    pub holdings: Vec<FundHoldingRecord>,
    pub proxy_votes_count: usize,
    pub proxy_votes: Vec<FundProxyVoteRecord>,
    pub shareholder_report: Option<FundExcerptRecord>,
    pub portfolio_summary: Option<FundExcerptRecord>,
    pub proxy_voting_record: Option<FundExcerptRecord>,
    pub summary_prospectus: Option<FundExcerptRecord>,
    pub registration_fee_notice: Option<FundExcerptRecord>,
    pub financial_statements: Option<FundExcerptRecord>,
    pub controls: Option<FundExcerptRecord>,
    pub document: Option<String>,
    pub document_sequence: Option<String>,
    pub document_description: Option<String>,
    pub document_url: Option<String>,
    pub source_url: String,
}

#[derive(Debug, Default, Serialize)]
pub struct FundHoldingRecord {
    pub name: Option<String>,
    pub title: Option<String>,
    pub cusip: Option<String>,
    pub lei: Option<String>,
    pub balance: Option<f64>,
    pub units: Option<String>,
    pub currency: Option<String>,
    pub value_usd: Option<f64>,
    pub pct_value: Option<f64>,
    pub asset_category: Option<String>,
    pub issuer_category: Option<String>,
    pub country: Option<String>,
    pub is_restricted: Option<bool>,
    pub liquidity_category: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct FundProxyVoteRecord {
    pub issuer_name: Option<String>,
    pub cusip: Option<String>,
    pub meeting_date: Option<String>,
    pub matter: Option<String>,
    pub vote_cast: Option<String>,
    pub management_recommendation: Option<String>,
    pub shares_voted: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct FundExcerptRecord {
    pub title: String,
    pub content: String,
    pub byte_length: usize,
    pub returned_bytes: usize,
    pub truncated: bool,
}
