use std::sync::LazyLock;

use anyhow::{Context, Result, bail};
use regex::Regex;

use crate::sec::{
    client::SecClient,
    edgar::accession_document_url,
    models::{DocumentContentRecord, DocumentReadQuery, FilingQuery, FilingRecord},
    utils::truncate_utf8,
};

use super::{DocumentSet, SubmissionDocument};

impl SecClient {
    pub async fn document_content(
        &self,
        query: DocumentReadQuery,
    ) -> Result<DocumentContentRecord> {
        let filings = self
            .filings(FilingQuery {
                cik: query.cik,
                form: query.form.clone(),
                latest: query.latest,
                from: None,
                to: None,
                include_amends: query.include_amends,
            })
            .await?;

        let filing = select_filing(&filings, query.accession.as_deref())?;
        let docs = self.filing_documents(filing).await?;
        let doc = select_document(&docs, &query)?;

        Ok(content_record(filing, doc, query.limit_bytes))
    }
}

pub fn plain_text(content: &str) -> String {
    static SCRIPT_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new("(?is)<(script|style)[^>]*>.*?</(script|style)>").expect("valid regex")
    });
    static TAG_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("(?is)<[^>]+>").expect("valid regex"));

    let without_scripts = SCRIPT_RE.replace_all(content, " ");
    let without_tags = TAG_RE.replace_all(&without_scripts, " ");
    let decoded = without_tags
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");
    let decoded = decode_numeric_entities(&decoded);

    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn decode_numeric_entities(value: &str) -> String {
    static NUMERIC_ENTITY_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"&#(x[0-9a-fA-F]+|\d+);").expect("valid regex"));

    NUMERIC_ENTITY_RE
        .replace_all(value, |caps: &regex::Captures| {
            let raw = &caps[1];
            let parsed = if let Some(hex) = raw.strip_prefix('x') {
                u32::from_str_radix(hex, 16).ok()
            } else {
                raw.parse::<u32>().ok()
            };
            parsed
                .and_then(char::from_u32)
                .map(|ch| ch.to_string())
                .unwrap_or_else(|| caps[0].to_string())
        })
        .into_owned()
}

fn select_filing<'a>(
    filings: &'a [FilingRecord],
    accession: Option<&str>,
) -> Result<&'a FilingRecord> {
    if let Some(accession) = accession {
        return filings
            .iter()
            .find(|filing| filing.accession == accession)
            .with_context(|| format!("accession '{}' not found in selected filings", accession));
    }

    filings
        .first()
        .context("no filings matched the document query")
}

fn select_document<'a>(
    docs: &'a [SubmissionDocument],
    query: &DocumentReadQuery,
) -> Result<&'a SubmissionDocument> {
    if let Some(filename) = query.filename.as_deref() {
        return docs
            .iter()
            .find(|doc| doc.filename.as_deref() == Some(filename))
            .with_context(|| format!("document filename '{}' not found", filename));
    }

    if let Some(sequence) = query.sequence.as_deref() {
        return docs
            .iter()
            .find(|doc| doc.sequence.as_deref() == Some(sequence))
            .with_context(|| format!("document sequence '{}' not found", sequence));
    }

    if query.primary {
        return DocumentSet::new(docs)
            .primary_documents()
            .next()
            .context("primary document not found");
    }

    DocumentSet::new(docs)
        .primary_documents()
        .next()
        .or_else(|| docs.first())
        .context("submission has no documents")
}

fn content_record(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
    limit_bytes: Option<usize>,
) -> DocumentContentRecord {
    let byte_length = doc.content.len();
    let (content, truncated) = truncate_utf8(&doc.content, limit_bytes);
    let returned_bytes = content.len();

    DocumentContentRecord {
        accession: filing.accession.clone(),
        cik: filing.cik,
        company: filing.company.clone(),
        form: filing.form.clone(),
        filing_date: filing.filing_date.clone(),
        document_type: doc.document_type.clone(),
        sequence: doc.sequence.clone(),
        filename: doc.filename.clone(),
        description: doc.description.clone(),
        content_type: doc.content_type().to_string(),
        byte_length,
        returned_bytes,
        truncated,
        is_primary: doc.is_primary(),
        source_url: filing.source_url.clone(),
        document_url: doc
            .filename
            .as_deref()
            .map(|filename| accession_document_url(filing.cik, &filing.accession, filename)),
        content,
    }
}

pub fn content_for_terminal(
    record: &DocumentContentRecord,
    text_mode: bool,
    limit_bytes: Option<usize>,
) -> String {
    let content = if text_mode {
        plain_text(&record.content)
    } else {
        record.content.clone()
    };
    truncate_utf8(&content, limit_bytes).0
}

pub fn validate_doc_args(filename: &Option<String>, sequence: &Option<String>) -> Result<()> {
    if filename.is_some() && sequence.is_some() {
        bail!("use only one document selector: --filename or --sequence");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_on_utf8_boundary() {
        let (content, truncated) = truncate_utf8("AAPL 苹果", Some(6));

        assert!(truncated);
        assert_eq!(content, "AAPL ");
    }

    #[test]
    fn extracts_plain_text_from_html() {
        let text =
            plain_text("<html><script>x()</script><body>A&nbsp;&amp; B &#8217;</body></html>");

        assert_eq!(text, "A & B \u{2019}");
    }
}
