pub mod holdings;
pub mod summary;

use anyhow::Result;

use crate::sec::{
    client::SecClient,
    models::{
        FilingQuery, FilingRecord, ThirteenFHoldingRecord, ThirteenFQuery, ThirteenFReportRecord,
    },
};

pub(crate) const VALUE_SCALE_CUTOFF: &str = "2022-09-30";

impl SecClient {
    pub async fn thirteenf_holdings(
        &self,
        query: ThirteenFQuery,
    ) -> Result<Vec<ThirteenFHoldingRecord>> {
        let filings = self.thirteenf_filings(&query).await?;
        let mut records = Vec::new();
        for filing in filings {
            let docs = self.filing_documents(&filing).await?;
            records.extend(holdings::parse_13f_documents(&filing, &docs)?);
        }
        Ok(records)
    }

    pub async fn thirteenf_reports(
        &self,
        query: ThirteenFQuery,
    ) -> Result<Vec<ThirteenFReportRecord>> {
        let filings = self.thirteenf_filings(&query).await?;
        let mut records = Vec::new();
        for filing in filings {
            let docs = self.filing_documents(&filing).await?;
            records.extend(summary::parse_13f_report_documents(&filing, &docs)?);
        }
        Ok(records)
    }

    async fn thirteenf_filings(&self, query: &ThirteenFQuery) -> Result<Vec<FilingRecord>> {
        self.filings(FilingQuery {
            cik: query.cik,
            form: Some("13F-HR".to_string()),
            latest: query.latest,
            from: None,
            to: None,
            include_amends: query.include_amends,
        })
        .await
    }
}

pub(crate) fn value_scale(report_date: Option<&str>) -> &'static str {
    match report_date {
        Some(date) if date <= VALUE_SCALE_CUTOFF => "usd_thousands",
        _ => "usd",
    }
}

pub(crate) fn scale_value_to_usd(value: u64, scale: &str) -> u64 {
    if scale == "usd_thousands" {
        value.saturating_mul(1000)
    } else {
        value
    }
}

pub(crate) fn normalize_mmddyyyy(value: &str) -> String {
    let trimmed = value.trim();
    let parts: Vec<&str> = trimmed.split('-').collect();
    if parts.len() == 3 && parts[2].len() == 4 {
        format!("{}-{}-{}", parts[2], parts[0], parts[1])
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
pub(crate) fn sample_filing() -> FilingRecord {
    FilingRecord {
        accession: "0000000000-00-000013".to_string(),
        cik: 13,
        company: "ACME CAPITAL".to_string(),
        form: "13F-HR".to_string(),
        filing_date: "2026-02-14".to_string(),
        report_date: Some("2025-12-31".to_string()),
        primary_document: None,
        primary_doc_description: None,
        is_xbrl: None,
        is_inline_xbrl: None,
        source_url: "https://example.test/index.html".to_string(),
        text_url: "https://example.test/submission.txt".to_string(),
    }
}

#[cfg(test)]
pub(crate) fn sample_doc(
    document_type: &str,
    sequence: &str,
    filename: &str,
    description: &str,
    content: &str,
) -> crate::sec::documents::SubmissionDocument {
    use crate::sec::documents::SubmissionDocument;

    SubmissionDocument {
        document_type: Some(document_type.to_string()),
        sequence: Some(sequence.to_string()),
        filename: Some(filename.to_string()),
        description: Some(description.to_string()),
        content: content.to_string(),
    }
}
