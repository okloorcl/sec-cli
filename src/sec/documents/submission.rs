use anyhow::{Context, Result};

use crate::sec::{client::SecClient, models::FilingRecord, utils::nonempty};

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

    pub fn content_type(&self) -> &'static str {
        if let Some(filename) = self.filename.as_deref() {
            if let Some(kind) = content_type_from_filename(filename) {
                return kind;
            }
        }

        let content = self.content.trim_start();
        if extract_tag(content, "XML").is_some() || content.starts_with("<?xml") {
            "xml"
        } else if extract_tag(content, "HTML").is_some()
            || content
                .get(..128)
                .unwrap_or(content)
                .to_ascii_lowercase()
                .contains("<html")
        {
            "html"
        } else {
            "text"
        }
    }

    pub fn is_primary(&self) -> bool {
        self.sequence
            .as_deref()
            .map(|value| value.trim() == "1")
            .unwrap_or(false)
    }
}

fn content_type_from_filename(filename: &str) -> Option<&'static str> {
    let ext = filename.rsplit('.').next()?.to_ascii_lowercase();
    match ext.as_str() {
        "htm" | "html" => Some("html"),
        "xml" | "xsd" => Some("xml"),
        "json" => Some("json"),
        "css" => Some("css"),
        "js" => Some("javascript"),
        "txt" => Some("text"),
        "jpg" | "jpeg" | "png" | "gif" => Some("image"),
        "zip" => Some("zip"),
        _ => None,
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
