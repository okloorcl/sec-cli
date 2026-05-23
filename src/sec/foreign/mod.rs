use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;

use crate::sec::{
    client::SecClient,
    documents::{DocumentSet, SubmissionDocument, read::plain_text},
    edgar::accession_document_url,
    models::{
        FilingQuery, FilingRecord, ForeignExcerptRecord, ForeignIssuerQuery, ForeignIssuerRecord,
    },
    parsers::text_helpers,
};

const FOREIGN_FORMS: &[&str] = &["20-F", "20-F/A", "6-K", "6-K/A", "40-F", "40-F/A"];

impl SecClient {
    pub async fn foreign_issuer_reports(
        &self,
        query: ForeignIssuerQuery,
    ) -> Result<Vec<ForeignIssuerRecord>> {
        let filings = self.foreign_filings(&query).await?;
        let mut records = Vec::new();
        for filing in filings {
            let docs = self.filing_documents(&filing).await?;
            let Some(doc) = DocumentSet::new(&docs).primary_documents().next() else {
                continue;
            };
            records.push(parse_foreign_issuer_report(&filing, doc, query.limit_bytes));
        }
        Ok(records)
    }

    async fn foreign_filings(&self, query: &ForeignIssuerQuery) -> Result<Vec<FilingRecord>> {
        let requested = query
            .form
            .as_deref()
            .filter(|value| !value.eq_ignore_ascii_case("all"));
        let fetch_latest = if requested.is_some() {
            query.latest
        } else {
            query.latest.saturating_mul(20).max(50)
        };
        let filings = self
            .filings(FilingQuery {
                cik: query.cik,
                form: requested.map(str::to_string),
                latest: fetch_latest,
                from: None,
                to: None,
                include_amends: query.include_amends,
            })
            .await?;
        let mut filtered = filings
            .into_iter()
            .filter(|filing| is_foreign_form(&filing.form, query.include_amends))
            .collect::<Vec<_>>();
        filtered.truncate(query.latest);
        Ok(filtered)
    }
}

pub fn parse_foreign_issuer_report(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
    limit_bytes: Option<usize>,
) -> ForeignIssuerRecord {
    let text = plain_text(&doc.content);
    ForeignIssuerRecord {
        accession: filing.accession.clone(),
        cik: filing.cik,
        company: filing.company.clone(),
        form: filing.form.clone(),
        filing_date: filing.filing_date.clone(),
        report_type: report_type(&filing.form).to_string(),
        is_amendment: filing.form.ends_with("/A"),
        exchange: exchange(&text),
        ticker_or_symbol: ticker_or_symbol(&text),
        auditor: auditor(&text),
        event_signals: event_signals(&text),
        risk_factors: excerpt(&text, "Risk Factors", limit_bytes),
        business: excerpt(&text, "Business", limit_bytes),
        operating_review: excerpt(&text, "Operating and Financial Review", limit_bytes)
            .or_else(|| excerpt(&text, "Management's Discussion and Analysis", limit_bytes)),
        controls: excerpt(&text, "Controls and Procedures", limit_bytes),
        financial_statements: excerpt(&text, "Financial Statements", limit_bytes),
        document: doc.filename.clone(),
        document_sequence: doc.sequence.clone(),
        document_description: doc.description.clone(),
        document_url: doc
            .filename
            .as_deref()
            .map(|filename| accession_document_url(filing.cik, &filing.accession, filename)),
        source_url: filing.source_url.clone(),
    }
}

fn is_foreign_form(form: &str, include_amends: bool) -> bool {
    FOREIGN_FORMS
        .iter()
        .any(|candidate| form.eq_ignore_ascii_case(candidate))
        || include_amends
            && ["20-F", "6-K", "40-F"]
                .iter()
                .any(|base| form.eq_ignore_ascii_case(&format!("{base}/A")))
}

fn report_type(form: &str) -> &'static str {
    if form.starts_with("6-K") {
        "foreign_private_issuer_current_report"
    } else if form.starts_with("40-F") {
        "canadian_annual_report"
    } else {
        "foreign_private_issuer_annual_report"
    }
}

fn exchange(text: &str) -> Option<String> {
    capture_first(
        text,
        r"(?i)(Nasdaq Global Select Market|Nasdaq Global Market|New York Stock Exchange|NYSE|London Stock Exchange|Toronto Stock Exchange)",
    )
}

fn ticker_or_symbol(text: &str) -> Option<String> {
    capture_first(
        text,
        r#"(?i)(?:ticker|symbol)[:\s]+["“]?([A-Z][A-Z0-9.\-]{0,9})["”]?"#,
    )
    .map(|value| value.trim_end_matches(['.', ',', ';', ':']).to_string())
    .filter(|value| !value.is_empty())
}

fn auditor(text: &str) -> Option<String> {
    for pattern in [
        r"(?i)(Ernst\s*&\s*Young\s+LLP)",
        r"(?i)(Deloitte\s*&\s*Touche\s+LLP)",
        r"(?i)(PricewaterhouseCoopers\s+LLP|PwC)",
        r"(?i)(KPMG\s+LLP)",
    ] {
        if let Some(value) = capture_first(text, pattern) {
            return Some(value);
        }
    }
    None
}

fn event_signals(text: &str) -> Vec<String> {
    let mut signals = Vec::new();
    for (needle, label) in [
        ("press release", "press_release"),
        ("financial results", "financial_results"),
        ("interim report", "interim_report"),
        ("annual report", "annual_report"),
        ("dividend", "dividend"),
        ("acquisition", "acquisition"),
        ("material change", "material_change"),
    ] {
        if contains_ci(text, needle) {
            signals.push(label.to_string());
        }
    }
    signals
}

fn excerpt(text: &str, title: &str, limit_bytes: Option<usize>) -> Option<ForeignExcerptRecord> {
    let start = text_helpers::section_start(text, title, 101)?;
    let excerpt = text_helpers::excerpt_from_range(
        text,
        title,
        start,
        next_section_start(text, start + title.len()),
        limit_bytes,
    )?;
    Some(ForeignExcerptRecord {
        title: title.to_string(),
        byte_length: excerpt.byte_length,
        returned_bytes: excerpt.returned_bytes,
        truncated: excerpt.truncated,
        content: excerpt.content,
    })
}

fn next_section_start(text: &str, from: usize) -> Option<usize> {
    static SECTION_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(?i)\b(Risk Factors|Business|Operating and Financial Review|Management's Discussion and Analysis|Controls and Procedures|Financial Statements|Item\s+\d+[A-Z]?)\b",
        )
        .expect("valid foreign issuer section regex")
    });
    SECTION_RE.find(&text[from..]).map(|m| from + m.start())
}

fn capture_first(text: &str, pattern: &str) -> Option<String> {
    text_helpers::capture_first(text, pattern)
}

fn contains_ci(text: &str, needle: &str) -> bool {
    text_helpers::contains_ci(text, needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_foreign_issuer_signals() {
        let text = "FORM 20-F Annual Report. This introduction is intentionally long enough to avoid treating the first heading match as a table of contents hit. The securities trade on the New York Stock Exchange under symbol TSM. Risk Factors Our operations face geopolitical risks. Operating and Financial Review Revenue increased. Controls and Procedures Disclosure controls were effective. Financial Statements KPMG LLP audited the consolidated statements.";

        assert_eq!(report_type("20-F"), "foreign_private_issuer_annual_report");
        assert_eq!(exchange(text).as_deref(), Some("New York Stock Exchange"));
        assert_eq!(ticker_or_symbol(text).as_deref(), Some("TSM"));
        assert_eq!(auditor(text).as_deref(), Some("KPMG LLP"));
        assert!(excerpt(text, "Risk Factors", Some(80)).is_some());
    }

    #[test]
    fn classifies_6k_events() {
        let text =
            "The company furnished a press release with interim report and financial results.";
        let signals = event_signals(text);

        assert!(signals.contains(&"press_release".to_string()));
        assert!(signals.contains(&"financial_results".to_string()));
        assert!(signals.contains(&"interim_report".to_string()));
    }
}
