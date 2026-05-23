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
pub(super) struct DailyParams {
    pub(super) date: Option<chrono::NaiveDate>,
    pub(super) form: Option<String>,
    pub(super) company: Option<String>,
    pub(super) limit: Option<usize>,
    pub(super) include_amends: Option<bool>,
}

#[derive(Deserialize)]
pub(super) struct EftsParams {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) query: String,
    pub(super) form: Option<String>,
    pub(super) from: Option<chrono::NaiveDate>,
    pub(super) to: Option<chrono::NaiveDate>,
    pub(super) limit: Option<usize>,
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
pub(super) struct MetricsParams {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) period: Option<String>,
    pub(super) unit: Option<String>,
    pub(super) latest: Option<usize>,
}

#[derive(Deserialize)]
pub(super) struct AgentPackParams {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) form: Option<String>,
    pub(super) latest: Option<usize>,
    pub(super) sections: Option<String>,
    pub(super) section_limit_bytes: Option<usize>,
    pub(super) metrics_latest: Option<usize>,
}

#[derive(Deserialize)]
pub(super) struct CompanyReportParams {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) form: Option<String>,
    pub(super) topic: Option<String>,
    pub(super) latest: Option<usize>,
    pub(super) include_amends: Option<bool>,
    pub(super) limit_tables: Option<usize>,
    pub(super) limit_rows: Option<usize>,
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
pub(super) struct EightKExhibitParams {
    pub(super) ticker: Option<String>,
    pub(super) cik: Option<u64>,
    pub(super) latest: Option<usize>,
    pub(super) include_amends: Option<bool>,
    pub(super) category: Option<String>,
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

pub(super) fn sections_list(value: Option<String>) -> Vec<String> {
    let sections = value
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if sections.is_empty() {
        vec!["risk-factors".to_string(), "mda".to_string()]
    } else {
        sections
    }
}
