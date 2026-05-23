use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate};

use crate::sec::{
    SecClient,
    edgar::urls::accession_index_url,
    models::{DailyFilingRecord, DailyIndexQuery},
};

const DAILY_INDEX_ROOT: &str = "https://www.sec.gov/Archives/edgar/daily-index";
const ARCHIVES_ROOT: &str = "https://www.sec.gov/Archives";

impl SecClient {
    pub async fn daily_filings(&self, query: DailyIndexQuery) -> Result<Vec<DailyFilingRecord>> {
        let text = self.get_text(&daily_index_url(query.date)).await?;
        let mut records = parse_master_index(&text)
            .with_context(|| format!("failed to parse SEC daily index for {}", query.date))?;
        records.retain(|record| matches_daily_filter(record, &query));
        if let Some(limit) = query.limit {
            records.truncate(limit);
        }
        Ok(records)
    }
}

pub fn latest_sec_index_date(today: NaiveDate) -> NaiveDate {
    match today.weekday() {
        chrono::Weekday::Sat => today - chrono::Duration::days(1),
        chrono::Weekday::Sun => today - chrono::Duration::days(2),
        _ => today,
    }
}

pub fn daily_index_url(date: NaiveDate) -> String {
    format!(
        "{}/{}/QTR{}/master.{}.idx",
        DAILY_INDEX_ROOT,
        date.year(),
        quarter(date),
        date.format("%Y%m%d")
    )
}

pub fn parse_master_index(text: &str) -> Result<Vec<DailyFilingRecord>> {
    let mut records = Vec::new();
    let mut in_rows = false;

    for line in text.lines() {
        if !in_rows {
            in_rows = line.starts_with("-----");
            continue;
        }
        let parts = line.split('|').collect::<Vec<_>>();
        if parts.len() != 5 {
            continue;
        }
        let Ok(cik) = parts[0].trim().parse::<u64>() else {
            continue;
        };
        let company = parts[1].trim().to_string();
        let form = parts[2].trim().to_string();
        let filing_date = parts[3].trim().to_string();
        let filename = parts[4].trim().to_string();
        let accession = accession_from_filename(&filename);
        let source_url = accession
            .as_deref()
            .map(|accession| accession_index_url(cik, accession));

        records.push(DailyFilingRecord {
            cik,
            company,
            form,
            filing_date,
            accession,
            text_url: format!("{ARCHIVES_ROOT}/{filename}"),
            filename,
            source_url,
        });
    }

    Ok(records)
}

fn matches_daily_filter(record: &DailyFilingRecord, query: &DailyIndexQuery) -> bool {
    if let Some(form) = query.form.as_deref()
        && !matches_form(&record.form, form, query.include_amends)
    {
        return false;
    }
    if let Some(company) = query.company.as_deref()
        && !record
            .company
            .to_ascii_lowercase()
            .contains(&company.to_ascii_lowercase())
    {
        return false;
    }
    true
}

fn matches_form(actual: &str, expected: &str, include_amends: bool) -> bool {
    actual.eq_ignore_ascii_case(expected)
        || include_amends && actual.eq_ignore_ascii_case(&format!("{expected}/A"))
}

fn accession_from_filename(filename: &str) -> Option<String> {
    filename
        .rsplit('/')
        .next()
        .and_then(|name| name.strip_suffix(".txt"))
        .filter(|value| value.contains('-'))
        .map(str::to_string)
}

fn quarter(date: NaiveDate) -> u32 {
    (date.month0() / 3) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_daily_index_url() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
        assert_eq!(
            daily_index_url(date),
            "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR2/master.20260515.idx"
        );
    }

    #[test]
    fn rolls_weekend_to_latest_sec_index_day() {
        let saturday = NaiveDate::from_ymd_opt(2026, 5, 23).unwrap();
        let sunday = NaiveDate::from_ymd_opt(2026, 5, 24).unwrap();

        assert_eq!(
            latest_sec_index_date(saturday),
            NaiveDate::from_ymd_opt(2026, 5, 22).unwrap()
        );
        assert_eq!(
            latest_sec_index_date(sunday),
            NaiveDate::from_ymd_opt(2026, 5, 22).unwrap()
        );
    }

    #[test]
    fn parses_master_index_rows() {
        let text = "\
Description: Master Index of EDGAR Dissemination Feed
-----
CIK|Company Name|Form Type|Date Filed|Filename
-----
320193|Apple Inc.|10-K|2025-10-31|edgar/data/320193/0000320193-25-000079.txt
1067983|BERKSHIRE HATHAWAY INC|13F-HR/A|2026-05-15|edgar/data/1067983/0001193125-26-226661.txt
";

        let records = parse_master_index(text).unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(
            records[0].accession.as_deref(),
            Some("0000320193-25-000079")
        );
        assert_eq!(
            records[0].source_url.as_deref(),
            Some(
                "https://www.sec.gov/Archives/edgar/data/320193/000032019325000079/0000320193-25-000079-index.html"
            )
        );
    }

    #[test]
    fn filters_forms_and_amendments() {
        let record = DailyFilingRecord {
            cik: 1,
            company: "Example".to_string(),
            form: "10-K/A".to_string(),
            filing_date: "2026-01-01".to_string(),
            accession: None,
            filename: "edgar/data/1/example.txt".to_string(),
            text_url: "https://www.sec.gov/Archives/edgar/data/1/example.txt".to_string(),
            source_url: None,
        };

        assert!(!matches_form(&record.form, "10-K", false));
        assert!(matches_form(&record.form, "10-K", true));
    }
}
