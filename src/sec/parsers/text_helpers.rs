use regex::Regex;

use crate::sec::utils::truncate_utf8;

pub(crate) struct Excerpt {
    pub(crate) content: String,
    pub(crate) byte_length: usize,
    pub(crate) returned_bytes: usize,
    pub(crate) truncated: bool,
}

pub(crate) fn clean_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn clean_option(value: &str) -> Option<String> {
    let cleaned = clean_text(value);
    (!cleaned.is_empty()).then_some(cleaned)
}

pub(crate) fn capture_first(text: &str, pattern: &str) -> Option<String> {
    Regex::new(pattern)
        .ok()?
        .captures(text)
        .and_then(|capture| capture.get(1))
        .map(|m| clean_text(m.as_str()))
        .filter(|value| !value.is_empty())
}

pub(crate) fn capture_label(text: &str, label: &str) -> Option<String> {
    let pattern = format!(
        r"(?i)\b{}\b\s*[:\-]\s*([^\n]{{2,120}})",
        regex::escape(label)
    );
    capture_first(text, &pattern)
}

pub(crate) fn contains_ci(text: &str, needle: &str) -> bool {
    text.to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

pub(crate) fn section_start(text: &str, title: &str, min_offset: usize) -> Option<usize> {
    let pattern = format!(r"(?i)\b{}\b", regex::escape(title));
    Regex::new(&pattern)
        .ok()?
        .find_iter(text)
        .map(|m| m.start())
        .find(|pos| *pos >= min_offset)
}

pub(crate) fn excerpt_from_range(
    text: &str,
    title: &str,
    start: usize,
    end: Option<usize>,
    limit_bytes: Option<usize>,
) -> Option<Excerpt> {
    let end = end.unwrap_or(text.len());
    let content_full = text.get(start..end)?.trim();
    if content_full.len() < title.len() + 12 {
        return None;
    }
    let (content, truncated) = truncate_utf8(content_full, limit_bytes.or(Some(1200)));
    Some(Excerpt {
        byte_length: content_full.len(),
        returned_bytes: content.len(),
        truncated,
        content,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_and_captures_text() {
        assert_eq!(clean_text(" one\n two\tthree "), "one two three");
        assert_eq!(
            capture_label("Registrant: Example Trust", "Registrant").as_deref(),
            Some("Example Trust")
        );
    }

    #[test]
    fn extracts_excerpt_with_utf8_limit() {
        let text = "Risk Factors 内容内容内容内容内容内容内容内容 Business next";
        let excerpt = excerpt_from_range(text, "Risk Factors", 0, None, Some(24)).unwrap();

        assert!(excerpt.truncated);
        assert!(excerpt.returned_bytes <= 24);
    }
}
