use anyhow::{Result, anyhow};

use super::params::CompanyParams;
use crate::sec::{Form4Query, SecClient, ThirteenFQuery};

pub(super) async fn resolve_cik(
    client: &SecClient,
    ticker: Option<&str>,
    cik: Option<u64>,
) -> Result<u64> {
    match (ticker, cik) {
        (Some(ticker), None) => client.cik_for_ticker(ticker).await,
        (None, Some(cik)) => Ok(cik),
        (Some(_), Some(_)) => Err(anyhow!("provide either ticker or cik, not both")),
        (None, None) => Err(anyhow!("provide ticker or cik")),
    }
}

pub(super) async fn resolve_optional_cik(
    client: &SecClient,
    ticker: Option<&str>,
    cik: Option<u64>,
) -> Result<Option<u64>> {
    match (ticker, cik) {
        (Some(ticker), None) => client.cik_for_ticker(ticker).await.map(Some),
        (None, Some(cik)) => Ok(Some(cik)),
        (Some(_), Some(_)) => Err(anyhow!("provide either ticker or cik, not both")),
        (None, None) => Ok(None),
    }
}

pub(super) async fn form4_query(client: &SecClient, params: &CompanyParams) -> Result<Form4Query> {
    Ok(Form4Query {
        cik: resolve_cik(client, params.ticker.as_deref(), params.cik).await?,
        latest: params.latest.unwrap_or(3),
        include_amends: params.include_amends.unwrap_or(false),
    })
}

pub(super) async fn thirteenf_query(
    client: &SecClient,
    params: &CompanyParams,
) -> Result<ThirteenFQuery> {
    Ok(ThirteenFQuery {
        cik: resolve_cik(client, params.ticker.as_deref(), params.cik).await?,
        latest: params.latest.unwrap_or(1),
        include_amends: params.include_amends.unwrap_or(false),
    })
}

pub(super) fn period_form(period: Option<&str>) -> Option<String> {
    match period.unwrap_or("annual").to_ascii_lowercase().as_str() {
        "annual" => Some("10-K".to_string()),
        "quarterly" => Some("10-Q".to_string()),
        "all" => None,
        other => Some(other.to_string()),
    }
}
