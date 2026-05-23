use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;

use crate::sec::{
    client::SecClient,
    documents::{DocumentSet, SubmissionDocument, read::plain_text},
    edgar::accession_document_url,
    models::{
        FilingQuery, FilingRecord, HtmlTableRecord, ProspectusExcerptRecord, ProspectusQuery,
        ProspectusRecord, ProspectusTableRecord,
    },
    parsers::text_helpers,
    tables::extract_html_tables,
};

const PROSPECTUS_FORMS: &[&str] = &[
    "S-1", "S-1/A", "F-1", "F-1/A", "424B1", "424B2", "424B3", "424B4", "424B5", "424B7",
];

impl SecClient {
    pub async fn prospectuses(&self, query: ProspectusQuery) -> Result<Vec<ProspectusRecord>> {
        let filings = self.prospectus_filings(&query).await?;
        let mut records = Vec::new();
        for (filing, docs) in self.filing_documents_batch(filings).await? {
            let Some(doc) = DocumentSet::new(&docs).primary_documents().next() else {
                continue;
            };
            records.push(parse_prospectus(&filing, doc, &query)?);
        }
        Ok(records)
    }

    async fn prospectus_filings(&self, query: &ProspectusQuery) -> Result<Vec<FilingRecord>> {
        let requested_form = query
            .form
            .as_deref()
            .filter(|value| !value.eq_ignore_ascii_case("all"));
        let sec_form = requested_form.filter(|value| !value.eq_ignore_ascii_case("424B"));
        let fetch_latest = if sec_form.is_some() {
            query.latest
        } else {
            query.latest.saturating_mul(20).max(50)
        };
        let filings = self
            .filings(FilingQuery {
                cik: query.cik,
                form: sec_form.map(str::to_string),
                latest: fetch_latest,
                from: None,
                to: None,
                include_amends: query.include_amends,
            })
            .await?;

        let mut filtered = filings
            .into_iter()
            .filter(|filing| {
                is_prospectus_form(&filing.form, query.include_amends)
                    && matches_requested_form(&filing.form, requested_form, query.include_amends)
            })
            .collect::<Vec<_>>();
        filtered.truncate(query.latest);
        Ok(filtered)
    }
}

pub fn parse_prospectus(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
    query: &ProspectusQuery,
) -> Result<ProspectusRecord> {
    let text = plain_text(&doc.content);
    let tables = extract_html_tables(filing, doc, query.limit_tables, query.limit_rows)?;
    Ok(ProspectusRecord {
        accession: filing.accession.clone(),
        cik: filing.cik,
        company: filing.company.clone(),
        form: filing.form.clone(),
        filing_date: filing.filing_date.clone(),
        prospectus_type: prospectus_type(&filing.form).to_string(),
        is_amendment: filing.form.ends_with("/A"),
        is_ipo_related: is_ipo_related(&text, &filing.form),
        securities_offered: securities_offered(&text),
        proposed_ticker: proposed_ticker(&text),
        exchange: exchange(&text),
        price_range: price_range(&text),
        shares_offered: shares_offered(&text),
        offering_amount: offering_amount(&text),
        underwriters: underwriters(&text),
        auditor: auditor(&text),
        use_of_proceeds: excerpt(&text, "Use of Proceeds", query.limit_bytes),
        risk_factors: excerpt(&text, "Risk Factors", query.limit_bytes),
        business: excerpt(&text, "Business", query.limit_bytes),
        dilution: excerpt(&text, "Dilution", query.limit_bytes),
        tables: prospectus_tables(&tables),
        document: doc.filename.clone(),
        document_sequence: doc.sequence.clone(),
        document_description: doc.description.clone(),
        document_url: doc
            .filename
            .as_deref()
            .map(|filename| accession_document_url(filing.cik, &filing.accession, filename)),
        source_url: filing.source_url.clone(),
    })
}

fn is_prospectus_form(form: &str, include_amends: bool) -> bool {
    PROSPECTUS_FORMS
        .iter()
        .any(|candidate| form.eq_ignore_ascii_case(candidate))
        || include_amends
            && ["S-1", "F-1"]
                .iter()
                .any(|base| form.eq_ignore_ascii_case(&format!("{base}/A")))
}

fn matches_requested_form(form: &str, requested: Option<&str>, include_amends: bool) -> bool {
    let Some(requested) = requested else {
        return true;
    };
    if requested.eq_ignore_ascii_case("424B") {
        return form.to_ascii_uppercase().starts_with("424B");
    }
    form.eq_ignore_ascii_case(requested)
        || include_amends && form.eq_ignore_ascii_case(&format!("{requested}/A"))
}

fn prospectus_type(form: &str) -> &'static str {
    if form.starts_with("424B") {
        "final_or_supplemental_prospectus"
    } else if form.starts_with("F-1") {
        "foreign_issuer_registration"
    } else {
        "registration_statement"
    }
}

fn is_ipo_related(text: &str, form: &str) -> bool {
    form.starts_with("S-1")
        || form.starts_with("F-1")
        || contains_ci(text, "initial public offering")
        || contains_ci(text, "this is our initial public offering")
}

fn securities_offered(text: &str) -> Vec<String> {
    let mut values = Vec::new();
    for pattern in [
        r"(?i)(Class\s+[A-Z]\s+common stock)",
        r"(?i)(common stock)",
        r"(?i)(ordinary shares)",
        r"(?i)(American depositary shares)",
        r"(?i)(preferred stock)",
        r"(?i)(notes due \d{4})",
    ] {
        push_capture_unique(text, pattern, &mut values);
    }
    values
}

fn proposed_ticker(text: &str) -> Option<String> {
    capture_first(
        text,
        r#"(?i)under the symbol ["“]?([A-Z][A-Z0-9.\-]{0,9})["”]?"#,
    )
    .or_else(|| {
        capture_first(
            text,
            r#"(?i)trading symbol ["“]?([A-Z][A-Z0-9.\-]{0,9})["”]?"#,
        )
    })
}

fn exchange(text: &str) -> Option<String> {
    capture_first(
        text,
        r"(?i)(Nasdaq Global Select Market|Nasdaq Capital Market|Nasdaq Global Market|New York Stock Exchange|NYSE American)",
    )
}

fn price_range(text: &str) -> Option<String> {
    capture_first(
        text,
        r"(?i)(\$[0-9][0-9,.]*\.?[0-9]*\s+(?:and|to)\s+\$[0-9][0-9,.]*\.?[0-9]*)",
    )
}

fn shares_offered(text: &str) -> Option<String> {
    capture_first(text, r"(?i)([0-9][0-9,]{2,}\s+shares)")
}

fn offering_amount(text: &str) -> Option<String> {
    capture_first(
        text,
        r"(?i)(?:aggregate offering price|maximum aggregate offering price|offering amount)[^$]{0,80}(\$[0-9][0-9,]*(?:\.[0-9]+)?)",
    )
}

fn underwriters(text: &str) -> Vec<String> {
    let known = [
        "Morgan Stanley",
        "Goldman Sachs",
        "J.P. Morgan",
        "JP Morgan",
        "BofA Securities",
        "Barclays",
        "Citigroup",
        "Deutsche Bank",
        "Evercore",
        "Jefferies",
        "RBC Capital Markets",
        "UBS",
        "Wells Fargo",
    ];
    known
        .iter()
        .filter(|name| contains_ci(text, name))
        .map(|name| (*name).to_string())
        .collect()
}

fn auditor(text: &str) -> Option<String> {
    for pattern in [
        r"(?i)(Ernst\s*&\s*Young\s+LLP)",
        r"(?i)(Deloitte\s*&\s*Touche\s+LLP)",
        r"(?i)(PricewaterhouseCoopers\s+LLP|PwC)",
        r"(?i)(KPMG\s+LLP)",
        r"(?i)(BDO\s+USA,\s+P\.?C\.?)",
    ] {
        if let Some(value) = capture_first(text, pattern) {
            return Some(value);
        }
    }
    None
}

fn excerpt(text: &str, title: &str, limit_bytes: Option<usize>) -> Option<ProspectusExcerptRecord> {
    let start = text_helpers::section_start(text, title, 101)?;
    let excerpt = text_helpers::excerpt_from_range(
        text,
        title,
        start,
        next_section_start(text, start + title.len()),
        limit_bytes,
    )?;
    Some(ProspectusExcerptRecord {
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
            r"(?i)\b(Risk Factors|Use of Proceeds|Dividend Policy|Capitalization|Dilution|Management|Business|Underwriting|Plan of Distribution|Principal Stockholders)\b",
        )
        .expect("valid prospectus section regex")
    });
    SECTION_RE.find(&text[from..]).map(|m| from + m.start())
}

fn prospectus_tables(tables: &[HtmlTableRecord]) -> Vec<ProspectusTableRecord> {
    tables
        .iter()
        .filter(|table| is_relevant_table(table))
        .map(|table| ProspectusTableRecord {
            table_index: table.table_index,
            title_hint: table.title_hint.clone(),
            headers: table.headers.clone(),
            rows: table.rows.clone(),
            row_count: table.row_count,
            column_count: table.column_count,
            truncated: table.truncated,
        })
        .collect()
}

fn is_relevant_table(table: &HtmlTableRecord) -> bool {
    let haystack = format!(
        "{} {}",
        table.title_hint.as_deref().unwrap_or(""),
        table.headers.join(" ")
    )
    .to_ascii_lowercase();
    [
        "offering",
        "underwriting",
        "dilution",
        "capitalization",
        "proceeds",
        "shares",
        "price",
    ]
    .iter()
    .any(|needle| haystack.contains(needle))
}

fn capture_first(text: &str, pattern: &str) -> Option<String> {
    text_helpers::capture_first(text, pattern)
}

fn push_capture_unique(text: &str, pattern: &str, values: &mut Vec<String>) {
    if let Some(value) = capture_first(text, pattern) {
        let normalized = clean_text(&value);
        if !values
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(&normalized))
        {
            values.push(normalized);
        }
    }
}

fn contains_ci(text: &str, needle: &str) -> bool {
    text_helpers::contains_ci(text, needle)
}

fn clean_text(value: &str) -> String {
    text_helpers::clean_text(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_core_prospectus_signals() {
        let text = "PROSPECTUS 22,000,000 shares Class A common stock. We expect the initial public offering price to be between $31.00 and $34.00 per share. The Class A common stock has been approved for listing on the New York Stock Exchange under the symbol “ACME”. Morgan Stanley and Goldman Sachs are acting as underwriters. Use of Proceeds We intend to use the net proceeds for working capital. Risk Factors Investing in our common stock involves risks. Business We build software.";

        assert_eq!(proposed_ticker(text).as_deref(), Some("ACME"));
        assert_eq!(exchange(text).as_deref(), Some("New York Stock Exchange"));
        assert_eq!(price_range(text).as_deref(), Some("$31.00 and $34.00"));
        assert_eq!(shares_offered(text).as_deref(), Some("22,000,000 shares"));
        assert!(securities_offered(text).contains(&"Class A common stock".to_string()));
        assert_eq!(underwriters(text).len(), 2);
        assert!(excerpt(text, "Use of Proceeds", Some(60)).is_some());
    }

    #[test]
    fn matches_424b_family_selector() {
        assert!(matches_requested_form("424B4", Some("424B"), false));
        assert!(matches_requested_form("S-1/A", Some("S-1"), true));
        assert!(!matches_requested_form("S-1/A", Some("S-1"), false));
    }
}
