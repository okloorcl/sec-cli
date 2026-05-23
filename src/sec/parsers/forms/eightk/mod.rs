use std::collections::BTreeMap;

use anyhow::{Context, Result};
use regex::Regex;

use crate::sec::{
    client::SecClient,
    documents::{DocumentSet, SubmissionDocument, read::plain_text},
    edgar::accession_document_url,
    models::{EightKEventRecord, EightKQuery, FilingQuery, FilingRecord},
    utils::{nonempty, truncate_utf8},
};

impl SecClient {
    pub async fn eightk_events(&self, query: EightKQuery) -> Result<Vec<EightKEventRecord>> {
        let filings = self
            .filings(FilingQuery {
                cik: query.cik,
                form: Some("8-K".to_string()),
                latest: query.latest,
                from: None,
                to: None,
                include_amends: query.include_amends,
            })
            .await?;

        let item_filter = query.item.as_deref().map(normalize_item);
        let mut records = Vec::new();
        for filing in filings {
            let docs = self.filing_documents(&filing).await?;
            let Some(doc) = DocumentSet::new(&docs).primary_documents().next() else {
                continue;
            };
            records.extend(parse_8k_events(
                &filing,
                doc,
                item_filter.as_deref(),
                query.limit_bytes,
            )?);
        }
        Ok(records)
    }
}

pub fn parse_8k_events(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
    item_filter: Option<&str>,
    limit_bytes: Option<usize>,
) -> Result<Vec<EightKEventRecord>> {
    let text = plain_text(&doc.content);
    let headings = item_headings(&text)?;
    let mut best_by_item: BTreeMap<String, EventSpan> = BTreeMap::new();

    for (idx, heading) in headings.iter().enumerate() {
        if item_filter.is_some_and(|filter| filter != heading.item) {
            continue;
        }
        let end = headings
            .get(idx + 1)
            .map(|next| next.start)
            .unwrap_or(text.len());
        if end <= heading.start {
            continue;
        }
        let content = text[heading.start..end].trim().to_string();
        if content.len() < 24 {
            continue;
        }
        let span = EventSpan {
            item: heading.item.clone(),
            title: official_item_title(&heading.item)
                .or_else(|| nonempty(&heading.title))
                .unwrap_or_else(|| "Unclassified 8-K Item".to_string()),
            start: heading.start,
            end,
            content,
        };
        best_by_item
            .entry(span.item.clone())
            .and_modify(|existing| {
                if span.content.len() > existing.content.len() {
                    *existing = span.clone();
                }
            })
            .or_insert(span);
    }

    let mut spans: Vec<EventSpan> = best_by_item.into_values().collect();
    spans.sort_by_key(|span| span.start);

    Ok(spans
        .into_iter()
        .map(|span| event_record(filing, doc, span, limit_bytes))
        .collect())
}

fn item_headings(text: &str) -> Result<Vec<ItemHeading>> {
    let regex = Regex::new(r"(?i)\bitem\s+([1-9]\.\d{2})\b").context("invalid 8-K item regex")?;
    let mut headings = Vec::new();
    for capture in regex.captures_iter(text) {
        let Some(full) = capture.get(0) else {
            continue;
        };
        let item = normalize_item(capture.get(1).map(|m| m.as_str()).unwrap_or_default());
        if !is_known_8k_item(&item) {
            continue;
        }
        headings.push(ItemHeading {
            item,
            title: extract_inline_title(text, full.end()),
            start: full.start(),
        });
    }
    headings.sort_by_key(|heading| heading.start);
    headings.dedup_by(|left, right| left.item == right.item && left.start == right.start);
    Ok(headings)
}

fn event_record(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
    span: EventSpan,
    limit_bytes: Option<usize>,
) -> EightKEventRecord {
    let byte_length = span.content.len();
    let (content, truncated) = truncate_utf8(&span.content, limit_bytes);
    EightKEventRecord {
        accession: filing.accession.clone(),
        cik: filing.cik,
        company: filing.company.clone(),
        filing_date: filing.filing_date.clone(),
        report_date: filing.report_date.clone(),
        item: span.item.clone(),
        item_title: span.title,
        category: item_category(&span.item).to_string(),
        is_furnished_item: matches!(span.item.as_str(), "2.02" | "7.01"),
        start_offset: span.start,
        end_offset: span.end,
        byte_length,
        returned_bytes: content.len(),
        truncated,
        document: doc.filename.clone(),
        document_sequence: doc.sequence.clone(),
        document_description: doc.description.clone(),
        document_url: doc
            .filename
            .as_deref()
            .map(|filename| accession_document_url(filing.cik, &filing.accession, filename)),
        source_url: filing.source_url.clone(),
        content,
    }
}

#[derive(Debug, Clone)]
struct EventSpan {
    item: String,
    title: String,
    start: usize,
    end: usize,
    content: String,
}

#[derive(Debug)]
struct ItemHeading {
    item: String,
    title: String,
    start: usize,
}

fn normalize_item(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("Item")
        .trim()
        .trim_end_matches('.')
        .to_string()
}

fn clean_heading_title(value: &str) -> String {
    value
        .split('|')
        .next()
        .unwrap_or(value)
        .trim()
        .trim_matches('.')
        .trim()
        .to_string()
}

fn extract_inline_title(text: &str, start: usize) -> String {
    let rest = &text[start..text.len().min(start + 180)];
    let next_item = Regex::new(r"(?i)\bitem\s+[1-9]\.\d{2}\b")
        .ok()
        .and_then(|regex| regex.find(rest).map(|m| m.start()))
        .unwrap_or(rest.len());
    let line_end = rest.find(['\n', '\r']).unwrap_or(rest.len());
    clean_heading_title(&rest[..next_item.min(line_end)])
}

fn is_known_8k_item(item: &str) -> bool {
    official_item_title(item).is_some()
}

fn official_item_title(item: &str) -> Option<String> {
    let title = match item {
        "1.01" => "Entry into a Material Definitive Agreement",
        "1.02" => "Termination of a Material Definitive Agreement",
        "1.03" => "Bankruptcy or Receivership",
        "1.04" => "Mine Safety - Reporting of Shutdowns and Patterns of Violations",
        "1.05" => "Material Cybersecurity Incidents",
        "2.01" => "Completion of Acquisition or Disposition of Assets",
        "2.02" => "Results of Operations and Financial Condition",
        "2.03" => "Creation of a Direct Financial Obligation",
        "2.04" => "Triggering Events That Accelerate or Increase a Direct Financial Obligation",
        "2.05" => "Costs Associated with Exit or Disposal Activities",
        "2.06" => "Material Impairments",
        "3.01" => "Notice of Delisting or Failure to Satisfy a Continued Listing Rule",
        "3.02" => "Unregistered Sales of Equity Securities",
        "3.03" => "Material Modification to Rights of Security Holders",
        "4.01" => "Changes in Registrant's Certifying Accountant",
        "4.02" => "Non-Reliance on Previously Issued Financial Statements",
        "5.01" => "Changes in Control of Registrant",
        "5.02" => "Departure or Appointment of Directors or Certain Officers",
        "5.03" => "Amendments to Articles of Incorporation or Bylaws; Change in Fiscal Year",
        "5.04" => "Temporary Suspension of Trading Under Registrant's Employee Benefit Plans",
        "5.05" => "Amendments to Code of Ethics or Waiver of a Provision",
        "5.06" => "Change in Shell Company Status",
        "5.07" => "Submission of Matters to a Vote of Security Holders",
        "5.08" => "Shareholder Director Nominations",
        "6.01" => "ABS Informational and Computational Material",
        "6.02" => "Change of Servicer or Trustee",
        "6.03" => "Change in Credit Enhancement or Other External Support",
        "6.04" => "Failure to Make a Required Distribution",
        "6.05" => "Securities Act Updating Disclosure",
        "7.01" => "Regulation FD Disclosure",
        "8.01" => "Other Events",
        "9.01" => "Financial Statements and Exhibits",
        _ => return None,
    };
    Some(title.to_string())
}

fn item_category(item: &str) -> &'static str {
    match item.as_bytes().first().copied() {
        Some(b'1') => "business_and_operations",
        Some(b'2') => "financial_information",
        Some(b'3') => "securities_and_trading_markets",
        Some(b'4') => "accountants_and_financial_statements",
        Some(b'5') => "corporate_governance_and_management",
        Some(b'6') => "asset_backed_securities",
        Some(b'7') => "regulation_fd",
        Some(b'8') => "other_events",
        Some(b'9') => "financial_statements_and_exhibits",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_8k_items_and_prefers_body_over_toc() {
        let filing = FilingRecord {
            accession: "0000320193-26-000001".to_string(),
            cik: 320193,
            company: "Apple Inc.".to_string(),
            form: "8-K".to_string(),
            filing_date: "2026-01-30".to_string(),
            report_date: Some("2026-01-29".to_string()),
            primary_document: Some("aapl-8k.htm".to_string()),
            primary_doc_description: Some("8-K".to_string()),
            is_xbrl: None,
            is_inline_xbrl: None,
            source_url: "https://example.test/index.html".to_string(),
            text_url: "https://example.test/submission.txt".to_string(),
        };
        let doc = SubmissionDocument {
            document_type: Some("8-K".to_string()),
            sequence: Some("1".to_string()),
            filename: Some("aapl-8k.htm".to_string()),
            description: Some("8-K".to_string()),
            content: r#"
                <html><body>
                Item 2.02 Results of Operations and Financial Condition.
                Item 9.01 Financial Statements and Exhibits.
                Item 2.02 Results of Operations and Financial Condition.
                The company announced quarterly results and furnished Exhibit 99.1.
                This is the longer body that should win over the table of contents.
                Item 9.01 Financial Statements and Exhibits.
                (d) Exhibits. 99.1 Press release.
                </body></html>
            "#
            .to_string(),
        };

        let records = parse_8k_events(&filing, &doc, None, Some(160)).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].item, "2.02");
        assert!(records[0].is_furnished_item);
        assert!(records[0].truncated);
        assert!(records[0].content.contains("quarterly results"));
        assert_eq!(records[1].item, "9.01");
    }
}
