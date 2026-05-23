use anyhow::{Context, Result};

use crate::sec::{client::SecClient, models::FilingRecord};

#[derive(Debug, Clone)]
pub struct SubmissionDocument {
    pub document_type: Option<String>,
    pub sequence: Option<String>,
    pub filename: Option<String>,
    pub description: Option<String>,
    pub content: String,
}

impl SubmissionDocument {
    pub fn is_type(&self, document_type: &str) -> bool {
        self.document_type
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case(document_type))
    }

    pub fn xml_content(&self) -> &str {
        extract_tag(&self.content, "XML").unwrap_or(&self.content)
    }

    pub fn html_content(&self) -> Option<&str> {
        extract_tag(&self.content, "HTML")
    }
}

impl SecClient {
    pub async fn filing_documents(&self, filing: &FilingRecord) -> Result<Vec<SubmissionDocument>> {
        let text = self
            .get_text(&filing.text_url)
            .await
            .with_context(|| format!("failed to download {}", filing.text_url))?;
        Ok(parse_documents(&text))
    }
}

pub fn parse_documents(text: &str) -> Vec<SubmissionDocument> {
    let lower = text.to_ascii_lowercase();
    let mut docs = Vec::new();
    let mut cursor = 0;

    while let Some(start_rel) = lower[cursor..].find("<document>") {
        let start = cursor + start_rel + "<document>".len();
        let Some(end_rel) = lower[start..].find("</document>") else {
            break;
        };
        let end = start + end_rel;
        let raw = &text[start..end];
        let content = extract_tag(raw, "TEXT").unwrap_or(raw).trim().to_string();

        docs.push(SubmissionDocument {
            document_type: extract_metadata(raw, "TYPE"),
            sequence: extract_metadata(raw, "SEQUENCE"),
            filename: extract_metadata(raw, "FILENAME"),
            description: extract_metadata(raw, "DESCRIPTION"),
            content,
        });

        cursor = end + "</document>".len();
    }

    docs
}

fn extract_metadata(raw: &str, tag: &str) -> Option<String> {
    let lower = raw.to_ascii_lowercase();
    let needle = format!("<{}>", tag.to_ascii_lowercase());
    let start = lower.find(&needle)? + needle.len();
    let rest = &raw[start..];
    let value = rest
        .lines()
        .next()
        .unwrap_or("")
        .split('<')
        .next()
        .unwrap_or("")
        .trim();
    nonempty(value)
}

fn extract_tag<'a>(raw: &'a str, tag: &str) -> Option<&'a str> {
    let lower = raw.to_ascii_lowercase();
    let open = format!("<{}>", tag.to_ascii_lowercase());
    let close = format!("</{}>", tag.to_ascii_lowercase());
    let start = lower.find(&open)? + open.len();
    let end = lower[start..].find(&close)? + start;
    Some(&raw[start..end])
}

fn nonempty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
