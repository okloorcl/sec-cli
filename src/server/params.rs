use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct CompanyParams {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) latest: Option<usize>,
    pub(super) include_amends: Option<bool>,
    pub(super) limit: Option<usize>,
}

#[derive(Deserialize)]
pub(super) struct FilingsParams {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) form: Option<String>,
    pub(super) latest: Option<usize>,
    pub(super) from: Option<chrono::NaiveDate>,
    pub(super) to: Option<chrono::NaiveDate>,
    pub(super) include_amends: Option<bool>,
}

#[derive(Deserialize)]
pub(super) struct FactsParams {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) concept: String,
    pub(super) form: Option<String>,
    pub(super) unit: Option<String>,
    pub(super) latest: Option<usize>,
}

#[derive(Deserialize)]
pub(super) struct StatementsParams {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) statement: Option<String>,
    pub(super) period: Option<String>,
    pub(super) unit: Option<String>,
    pub(super) latest: Option<usize>,
}

#[derive(Deserialize)]
pub(super) struct IxbrlParams {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) form: Option<String>,
    pub(super) concept: Option<String>,
    pub(super) latest: Option<usize>,
    pub(super) limit: Option<usize>,
    pub(super) include_amends: Option<bool>,
}

#[derive(Deserialize)]
pub(super) struct SectionParams {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) form: Option<String>,
    pub(super) latest: Option<usize>,
    pub(super) include_amends: Option<bool>,
    pub(super) accession: Option<String>,
    pub(super) item: String,
    pub(super) limit_bytes: Option<usize>,
}

#[derive(Deserialize)]
pub(super) struct DocumentParams {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) form: Option<String>,
    pub(super) latest: Option<usize>,
    pub(super) include_amends: Option<bool>,
    pub(super) limit: Option<usize>,
}

#[derive(Deserialize)]
pub(super) struct EightKParams {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) latest: Option<usize>,
    pub(super) include_amends: Option<bool>,
    pub(super) item: Option<String>,
    pub(super) limit_bytes: Option<usize>,
}

#[derive(Deserialize)]
pub(super) struct Schedule13Params {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) form: Option<String>,
    pub(super) latest: Option<usize>,
    pub(super) include_amends: Option<bool>,
    pub(super) limit_bytes: Option<usize>,
}

#[derive(Deserialize)]
pub(super) struct DisclosureParams {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) form: Option<String>,
    pub(super) latest: Option<usize>,
    pub(super) include_amends: Option<bool>,
    pub(super) limit_bytes: Option<usize>,
    pub(super) limit_tables: Option<usize>,
    pub(super) limit_rows: Option<usize>,
}

#[derive(Deserialize)]
pub(super) struct FundParams {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) form: Option<String>,
    pub(super) latest: Option<usize>,
    pub(super) include_amends: Option<bool>,
    pub(super) limit_holdings: Option<usize>,
    pub(super) limit_bytes: Option<usize>,
}

#[derive(Deserialize)]
pub(super) struct ParseParams {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) form: String,
    pub(super) latest: Option<usize>,
    pub(super) include_amends: Option<bool>,
    pub(super) limit: Option<usize>,
}
