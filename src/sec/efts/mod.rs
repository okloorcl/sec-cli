use anyhow::{Result, anyhow};
use reqwest::Url;
use serde::Deserialize;

use crate::sec::{
    SecClient,
    edgar::urls::{accession_document_url, accession_index_url},
    models::{EftsSearchQuery, EftsSearchRecord},
};

const EFTS_SEARCH_URL: &str = "https://efts.sec.gov/LATEST/search-index";

impl SecClient {
    pub async fn efts_search(&self, query: EftsSearchQuery) -> Result<Vec<EftsSearchRecord>> {
        let url = efts_search_url(&query)?;
        let response: EftsResponse = self.get_json(url.as_str()).await?;
        let mut records = response
            .hits
            .hits
            .into_iter()
            .map(|hit| hit.into_record(&query.query))
            .collect::<Vec<_>>();
        if let Some(limit) = query.limit {
            records.truncate(limit);
        }
        Ok(records)
    }
}

pub fn efts_search_url(query: &EftsSearchQuery) -> Result<Url> {
    let mut url = Url::parse(EFTS_SEARCH_URL)?;
    {
        let mut params = url.query_pairs_mut();
        params.append_pair("q", query.query.trim());
        params.append_pair("from", "0");
        if let Some(limit) = query.limit {
            params.append_pair("size", &limit.min(100).to_string());
        }
        if query.from.is_some() || query.to.is_some() {
            params.append_pair("dateRange", "custom");
            if let Some(from) = query.from {
                params.append_pair("startdt", &from.to_string());
            }
            if let Some(to) = query.to {
                params.append_pair("enddt", &to.to_string());
            }
        } else {
            params.append_pair("dateRange", "all");
        }
        if !query.forms.is_empty() {
            params.append_pair("forms", &query.forms.join(","));
        }
        if !query.ciks.is_empty() {
            let ciks = query
                .ciks
                .iter()
                .map(|cik| format!("{cik:010}"))
                .collect::<Vec<_>>()
                .join(",");
            params.append_pair("ciks", &ciks);
        }
    }
    Ok(url)
}

#[derive(Debug, Deserialize)]
struct EftsResponse {
    hits: EftsHits,
}

#[derive(Debug, Deserialize)]
struct EftsHits {
    hits: Vec<EftsHit>,
}

#[derive(Debug, Deserialize)]
struct EftsHit {
    #[serde(rename = "_id")]
    id: Option<String>,
    #[serde(rename = "_score")]
    score: Option<f64>,
    #[serde(rename = "_source")]
    source: EftsSource,
}

impl EftsHit {
    fn into_record(self, query: &str) -> EftsSearchRecord {
        let document = self
            .id
            .as_deref()
            .and_then(|id| id.split_once(':').map(|(_, document)| document.to_string()));
        let cik = self.source.first_cik();
        let accession = self.source.adsh.clone();
        let source_url = cik
            .zip(accession.as_deref())
            .map(|(cik, accession)| accession_index_url(cik, accession));
        let document_url = cik
            .zip(accession.as_deref())
            .zip(document.as_deref())
            .map(|((cik, accession), document)| accession_document_url(cik, accession, document));

        EftsSearchRecord {
            query: query.to_string(),
            score: self.score,
            cik,
            company: self.source.company_name(),
            form: self.source.form,
            file_date: self.source.file_date,
            period_ending: self.source.period_ending,
            accession,
            document,
            sequence: self.source.sequence,
            file_description: self.source.file_description,
            file_type: self.source.file_type,
            source_url,
            document_url,
        }
    }
}

#[derive(Debug, Deserialize)]
struct EftsSource {
    ciks: Option<Vec<String>>,
    display_names: Option<Vec<String>>,
    form: Option<String>,
    file_date: Option<String>,
    period_ending: Option<String>,
    adsh: Option<String>,
    sequence: Option<u64>,
    file_description: Option<String>,
    file_type: Option<String>,
}

impl EftsSource {
    fn first_cik(&self) -> Option<u64> {
        self.ciks
            .as_ref()
            .and_then(|values| values.first())
            .and_then(|value| value.parse::<u64>().ok())
    }

    fn company_name(&self) -> Option<String> {
        self.display_names
            .as_ref()
            .and_then(|values| values.first())
            .map(|value| {
                value
                    .split("(CIK")
                    .next()
                    .unwrap_or(value)
                    .trim()
                    .to_string()
            })
            .filter(|value| !value.is_empty())
    }
}

pub fn parse_forms(values: &[String]) -> Vec<String> {
    values
        .iter()
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn require_query(query: &str) -> Result<String> {
    let query = query.trim();
    if query.is_empty() {
        return Err(anyhow!("EFTS query cannot be empty"));
    }
    Ok(query.to_string())
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn builds_efts_search_url() {
        let url = efts_search_url(&EftsSearchQuery {
            query: "supply chain risk".to_string(),
            ciks: vec![320193],
            forms: vec!["10-K".to_string(), "10-Q".to_string()],
            from: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            to: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            limit: Some(20),
        })
        .unwrap();
        let url = url.as_str();

        assert!(url.contains("q=supply+chain+risk"));
        assert!(url.contains("forms=10-K%2C10-Q"));
        assert!(url.contains("ciks=0000320193"));
        assert!(url.contains("startdt=2024-01-01"));
    }

    #[test]
    fn parses_comma_separated_forms() {
        let forms = parse_forms(&["10-K,10-Q".to_string(), "8-K".to_string()]);

        assert_eq!(forms, vec!["10-K", "10-Q", "8-K"]);
    }
}
