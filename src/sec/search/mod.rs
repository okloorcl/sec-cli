use std::sync::LazyLock;

use regex::Regex;

use super::models::{FilingRecord, SearchMatch, SearchQuery};

pub fn find_matches(filing: &FilingRecord, text: &str, query: &SearchQuery) -> Vec<SearchMatch> {
    let needle = query.query.to_ascii_lowercase();
    let haystack = text.to_ascii_lowercase();
    let mut matches = Vec::new();
    let mut start_at = 0;

    while let Some(relative) = haystack[start_at..].find(&needle) {
        let offset = start_at + relative;
        matches.push(build_match(
            filing,
            text,
            query,
            offset,
            needle.len(),
            query.context,
        ));
        start_at = offset + needle.len().max(1);
    }

    if matches.is_empty() {
        matches.extend(find_token_window_matches(filing, text, query));
    }

    matches
}

fn find_token_window_matches(
    filing: &FilingRecord,
    text: &str,
    query: &SearchQuery,
) -> Vec<SearchMatch> {
    let tokens: Vec<String> = query
        .query
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|token| token.len() > 2)
        .map(|token| token.to_ascii_lowercase())
        .collect();
    if tokens.len() < 2 {
        return Vec::new();
    }

    let haystack = text.to_ascii_lowercase();
    let first = &tokens[0];
    let mut matches = Vec::new();
    let mut start_at = 0;
    let search_window = (query.context * 6).max(600);

    while let Some(relative) = haystack[start_at..].find(first) {
        let offset = start_at + relative;
        let window_start = offset.saturating_sub(search_window / 2);
        let window_end = (offset + search_window).min(text.len());
        let window_lc = &haystack[window_start..window_end];

        if tokens.iter().all(|token| window_lc.contains(token)) {
            matches.push(build_match(
                filing,
                text,
                query,
                offset,
                first.len(),
                query.context,
            ));
        }

        start_at = offset + first.len().max(1);
        if matches.len() >= 20 {
            break;
        }
    }

    matches
}

fn build_match(
    filing: &FilingRecord,
    text: &str,
    query: &SearchQuery,
    offset: usize,
    match_len: usize,
    context: usize,
) -> SearchMatch {
    let start = offset.saturating_sub(context);
    let end = (offset + match_len + context).min(text.len());

    SearchMatch {
        accession: filing.accession.clone(),
        cik: filing.cik,
        company: filing.company.clone(),
        form: filing.form.clone(),
        filing_date: filing.filing_date.clone(),
        query: query.query.clone(),
        document: filing
            .primary_document
            .clone()
            .unwrap_or_else(|| "complete-submission.txt".to_string()),
        section: infer_section(&text[..offset]),
        offset,
        snippet: normalize_ws(&strip_tags(
            &text[start..end],
            starts_inside_tag(text, start),
        )),
        source_url: filing.text_url.clone(),
    }
}

fn infer_section(prefix: &str) -> Option<String> {
    static SECTION_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)item\s+([0-9]+[a-z]?)\.?\s+([^\n<]{3,120})")
            .expect("valid search section regex")
    });

    let tail_start = prefix.len().saturating_sub(20_000);
    let tail = &prefix[tail_start..];
    SECTION_RE
        .captures_iter(tail)
        .last()
        .and_then(|caps| caps.get(0).map(|m| normalize_ws(m.as_str())))
}

fn strip_tags(value: &str, starts_inside_tag: bool) -> String {
    let mut out = String::with_capacity(value.len());
    let mut in_tag = starts_inside_tag;

    for ch in value.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn starts_inside_tag(full_text: &str, start: usize) -> bool {
    let prefix = &full_text[..start];
    let last_open = prefix.rfind('<');
    let last_close = prefix.rfind('>');
    last_open.is_some_and(|open| last_close.is_none_or(|close| open > close))
}

fn normalize_ws(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filing() -> FilingRecord {
        FilingRecord {
            accession: "000-test".to_string(),
            cik: 1,
            company: "ACME".to_string(),
            form: "10-K".to_string(),
            filing_date: "2026-01-01".to_string(),
            report_date: None,
            primary_document: Some("acme.htm".to_string()),
            primary_doc_description: None,
            is_xbrl: None,
            is_inline_xbrl: None,
            source_url: "https://www.sec.gov/index".to_string(),
            text_url: "https://www.sec.gov/text".to_string(),
        }
    }

    #[test]
    fn finds_exact_matches_with_section_context() {
        let text = "Item 1A. Risk Factors <b>supply chain risk</b> can be material.";
        let records = find_matches(
            &filing(),
            text,
            &SearchQuery {
                query: "supply chain risk".to_string(),
                context: 20,
            },
        );

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].section.as_deref(), Some("Item 1A. Risk Factors"));
        assert!(records[0].snippet.contains("supply chain risk"));
        assert!(!records[0].snippet.contains("<b>"));
    }

    #[test]
    fn falls_back_to_token_window_matching() {
        let text = "The company depends on supply partners across many markets and faces operational risk.";
        let records = find_matches(
            &filing(),
            text,
            &SearchQuery {
                query: "supply risk".to_string(),
                context: 30,
            },
        );

        assert_eq!(records.len(), 1);
    }

    #[test]
    fn strips_partial_tags_from_snippet_boundaries() {
        let text = "<span>alpha target beta</span>";
        let records = find_matches(
            &filing(),
            text,
            &SearchQuery {
                query: "target".to_string(),
                context: 8,
            },
        );

        assert_eq!(records[0].snippet, "alpha target beta");
    }
}
