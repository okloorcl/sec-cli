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
    let mut docs = Vec::new();
    let mut cursor = 0;
    const OPEN: &str = "<document>";
    const CLOSE: &str = "</document>";

    while let Some(open_start) = find_ascii_case_insensitive(text, OPEN, cursor) {
        let start = open_start + OPEN.len();
        let Some(end) = find_ascii_case_insensitive(text, CLOSE, start) else {
            break;
        };
        let raw = &text[start..end];
        let content = extract_tag(raw, "TEXT").unwrap_or(raw).trim().to_string();

        docs.push(SubmissionDocument {
            document_type: extract_metadata(raw, "TYPE"),
            sequence: extract_metadata(raw, "SEQUENCE"),
            filename: extract_metadata(raw, "FILENAME"),
            description: extract_metadata(raw, "DESCRIPTION"),
            content,
        });

        cursor = end + CLOSE.len();
    }

    docs
}

fn extract_metadata(raw: &str, tag: &str) -> Option<String> {
    let needle = format!("<{tag}>");
    let start = find_ascii_case_insensitive(raw, &needle, 0)? + needle.len();
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
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = find_ascii_case_insensitive(raw, &open, 0)? + open.len();
    let mut cursor = start;
    let mut depth = 1usize;

    loop {
        let next_open = find_ascii_case_insensitive(raw, &open, cursor);
        let next_close = find_ascii_case_insensitive(raw, &close, cursor)?;
        if next_open.is_some_and(|pos| pos < next_close) {
            depth += 1;
            cursor = next_open? + open.len();
            continue;
        }

        depth -= 1;
        if depth == 0 {
            return Some(&raw[start..next_close]);
        }
        cursor = next_close + close.len();
    }
}

fn find_ascii_case_insensitive(haystack: &str, needle: &str, from: usize) -> Option<usize> {
    let haystack = haystack.as_bytes();
    let needle = needle.as_bytes();
    if needle.is_empty() || from > haystack.len() || needle.len() > haystack.len() {
        return None;
    }
    haystack[from..]
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle))
        .map(|pos| from + pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_documents_case_insensitively() {
        let docs = parse_documents(
            "<DOCUMENT><TYPE>10-K\n<SEQUENCE>1\n<FILENAME>a.htm\n<TEXT>Hello</TEXT></DOCUMENT>",
        );

        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].document_type.as_deref(), Some("10-K"));
        assert_eq!(docs[0].content, "Hello");
    }

    #[test]
    fn extract_tag_handles_nested_same_name() {
        let value = extract_tag("<TEXT>outer <TEXT>inner</TEXT> tail</TEXT>", "TEXT").unwrap();

        assert_eq!(value, "outer <TEXT>inner</TEXT> tail");
    }
}
