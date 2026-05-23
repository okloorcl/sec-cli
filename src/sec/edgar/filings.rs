use anyhow::{Result, anyhow};
use chrono::NaiveDate;
use serde_json::Value;

use crate::sec::{
    client::SecClient,
    models::{FilingQuery, FilingRecord},
    utils::nonempty,
};

use super::urls::{accession_index_url, accession_text_url, submissions_url};

impl SecClient {
    pub async fn filings(&self, query: FilingQuery) -> Result<Vec<FilingRecord>> {
        let json: Value = self.get_json(&submissions_url(query.cik)).await?;
        let company = json
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let recent = json
            .get("filings")
            .and_then(|v| v.get("recent"))
            .ok_or_else(|| anyhow!("submissions JSON missing filings.recent"))?;

        let accessions = as_str_array(recent, "accessionNumber")?;
        let forms = as_str_array(recent, "form")?;
        let filing_dates = as_str_array(recent, "filingDate")?;
        let report_dates = optional_str_array(recent, "reportDate");
        let primary_documents = optional_str_array(recent, "primaryDocument");
        let descriptions = optional_str_array(recent, "primaryDocDescription");
        let is_xbrl = optional_bool_array(recent, "isXBRL");
        let is_inline_xbrl = optional_bool_array(recent, "isInlineXBRL");

        let mut records = Vec::new();
        for idx in 0..accessions.len() {
            let Some(accession) = accessions.get(idx) else {
                continue;
            };
            let form = forms.get(idx).copied().unwrap_or("");
            let filing_date = filing_dates.get(idx).copied().unwrap_or("");

            if !matches_form(form, query.form.as_deref(), query.include_amends) {
                continue;
            }
            if !matches_date(filing_date, query.from, query.to) {
                continue;
            }

            records.push(FilingRecord {
                accession: (*accession).to_string(),
                cik: query.cik,
                company: company.clone(),
                form: form.to_string(),
                filing_date: filing_date.to_string(),
                report_date: report_dates.get(idx).and_then(|v| nonempty(*v)),
                primary_document: primary_documents.get(idx).and_then(|v| nonempty(*v)),
                primary_doc_description: descriptions.get(idx).and_then(|v| nonempty(*v)),
                is_xbrl: is_xbrl.get(idx).copied().flatten(),
                is_inline_xbrl: is_inline_xbrl.get(idx).copied().flatten(),
                source_url: accession_index_url(query.cik, accession),
                text_url: accession_text_url(query.cik, accession),
            });

            if records.len() >= query.latest {
                break;
            }
        }

        Ok(records)
    }
}

pub(crate) fn matches_form(actual: &str, expected: Option<&str>, include_amends: bool) -> bool {
    let Some(expected) = expected else {
        return true;
    };
    if actual.eq_ignore_ascii_case(expected) {
        return true;
    }
    include_amends && actual.eq_ignore_ascii_case(&format!("{expected}/A"))
}

fn matches_date(filing_date: &str, from: Option<NaiveDate>, to: Option<NaiveDate>) -> bool {
    let Ok(date) = NaiveDate::parse_from_str(filing_date, "%Y-%m-%d") else {
        return true;
    };
    if from.is_some_and(|start| date < start) {
        return false;
    }
    if to.is_some_and(|end| date > end) {
        return false;
    }
    true
}

fn as_str_array<'a>(root: &'a Value, key: &str) -> Result<Vec<&'a str>> {
    root.get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("submissions JSON missing {}", key))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| anyhow!("{} contains non-string value", key))
        })
        .collect()
}

fn optional_str_array<'a>(root: &'a Value, key: &str) -> Vec<&'a str> {
    root.get(key)
        .and_then(Value::as_array)
        .map(|arr| arr.iter().map(|v| v.as_str().unwrap_or("")).collect())
        .unwrap_or_default()
}

fn optional_bool_array(root: &Value, key: &str) -> Vec<Option<bool>> {
    root.get(key)
        .and_then(Value::as_array)
        .map(|arr| arr.iter().map(Value::as_bool).collect())
        .unwrap_or_default()
}
