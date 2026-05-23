use anyhow::Result;
use chrono::NaiveDate;

use crate::sec::{
    client::SecClient,
    models::{FilingQuery, FilingRecord},
    utils::nonempty,
};

use super::{
    types::{RecentFilings, SubmissionsResponse},
    urls::{accession_index_url, accession_text_url, submissions_url},
};

impl SecClient {
    pub async fn filings(&self, query: FilingQuery) -> Result<Vec<FilingRecord>> {
        let response: SubmissionsResponse = self.get_json(&submissions_url(query.cik)).await?;
        let company = response.name;
        let recent = response.filings.recent;

        let mut records = Vec::new();
        for idx in 0..recent.accession_number.len() {
            let Some(accession) = recent.accession_number.get(idx) else {
                continue;
            };
            let form = recent.form.get(idx).map(String::as_str).unwrap_or("");
            let filing_date = recent
                .filing_date
                .get(idx)
                .map(String::as_str)
                .unwrap_or("");

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
                report_date: optional_text(&recent.report_date, idx),
                primary_document: optional_text(&recent.primary_document, idx),
                primary_doc_description: optional_text(&recent.primary_doc_description, idx),
                is_xbrl: optional_bool(&recent, idx, XbrlFlag::Xbrl),
                is_inline_xbrl: optional_bool(&recent, idx, XbrlFlag::InlineXbrl),
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

fn optional_text(values: &[String], idx: usize) -> Option<String> {
    values.get(idx).and_then(|value| nonempty(value))
}

enum XbrlFlag {
    Xbrl,
    InlineXbrl,
}

fn optional_bool(recent: &RecentFilings, idx: usize, flag: XbrlFlag) -> Option<bool> {
    match flag {
        XbrlFlag::Xbrl => recent.is_xbrl.get(idx),
        XbrlFlag::InlineXbrl => recent.is_inline_xbrl.get(idx),
    }
    .copied()
    .flatten()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn matches_amended_forms_only_when_requested() {
        assert!(matches_form("10-K", Some("10-K"), false));
        assert!(!matches_form("10-K/A", Some("10-K"), false));
        assert!(matches_form("10-K/A", Some("10-K"), true));
        assert!(matches_form("sc 13g/a", Some("SC 13G"), true));
    }

    #[test]
    fn matches_date_range() {
        let from = NaiveDate::from_ymd_opt(2026, 1, 1);
        let to = NaiveDate::from_ymd_opt(2026, 3, 31);

        assert!(matches_date("2026-02-01", from, to));
        assert!(!matches_date("2025-12-31", from, to));
        assert!(!matches_date("2026-04-01", from, to));
        assert!(matches_date("not-a-date", from, to));
    }
}
