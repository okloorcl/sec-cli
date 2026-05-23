pub mod aggregate;
pub mod diff;
pub mod holdings;
pub mod summary;

use anyhow::Result;

use crate::sec::{
    client::SecClient,
    models::{
        FilingQuery, FilingRecord, ThirteenFAggregateHoldingRecord, ThirteenFDiffRecord,
        ThirteenFHoldingRecord, ThirteenFQuery, ThirteenFReportRecord,
    },
};

// SEC 13F XML information tables historically report `value` in thousands of
// dollars. Modern 13F XML filings observed after this report date use dollar
// units directly, so older reports are scaled for comparable `value_usd`.
pub(crate) const VALUE_SCALE_CUTOFF: &str = "2022-09-30";

impl SecClient {
    pub async fn thirteenf_holdings(
        &self,
        query: ThirteenFQuery,
    ) -> Result<Vec<ThirteenFHoldingRecord>> {
        let filings = self.thirteenf_filings(&query).await?;
        let mut records = Vec::new();
        for (filing, docs) in self.filing_documents_batch(filings).await? {
            records.extend(holdings::parse_13f_documents(&filing, &docs)?);
        }
        Ok(records)
    }

    pub async fn thirteenf_aggregate_holdings(
        &self,
        query: ThirteenFQuery,
    ) -> Result<Vec<ThirteenFAggregateHoldingRecord>> {
        let holdings = self.thirteenf_holdings(query).await?;
        Ok(aggregate::aggregate_holdings(holdings))
    }

    pub async fn thirteenf_reports(
        &self,
        query: ThirteenFQuery,
    ) -> Result<Vec<ThirteenFReportRecord>> {
        let filings = self.thirteenf_filings(&query).await?;
        let mut records = Vec::new();
        for (filing, docs) in self.filing_documents_batch(filings).await? {
            records.extend(summary::parse_13f_report_documents(&filing, &docs)?);
        }
        Ok(records)
    }

    pub async fn thirteenf_diff_holdings(
        &self,
        query: ThirteenFQuery,
    ) -> Result<Vec<ThirteenFDiffRecord>> {
        let diff_query = ThirteenFQuery {
            latest: query.latest.max(2),
            ..query
        };
        let filings = self.thirteenf_filings(&diff_query).await?;
        let Some(current) = filings.first() else {
            return Ok(Vec::new());
        };
        let Some(previous) = filings.get(1) else {
            return Ok(Vec::new());
        };

        let current_docs = self.filing_documents(current).await?;
        let previous_docs = self.filing_documents(previous).await?;
        let current_holdings = holdings::parse_13f_documents(current, &current_docs)?;
        let previous_holdings = holdings::parse_13f_documents(previous, &previous_docs)?;
        Ok(diff::diff_holdings(
            aggregate::aggregate_holdings(current_holdings),
            aggregate::aggregate_holdings(previous_holdings),
        ))
    }

    pub(crate) async fn thirteenf_filings(
        &self,
        query: &ThirteenFQuery,
    ) -> Result<Vec<FilingRecord>> {
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
