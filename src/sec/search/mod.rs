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
        snippet: normalize_ws(&strip_tags(&text[start..end])),
        source_url: filing.text_url.clone(),
    }
}

fn infer_section(prefix: &str) -> Option<String> {
    let tail_start = prefix.len().saturating_sub(20_000);
    let tail = &prefix[tail_start..];
    let re = Regex::new(r"(?i)item\s+([0-9]+[a-z]?)\.?\s+([^\n<]{3,120})").ok()?;
    re.captures_iter(tail)
        .last()
        .and_then(|caps| caps.get(0).map(|m| normalize_ws(m.as_str())))
}

fn strip_tags(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut in_tag = value
        .find('>')
        .is_some_and(|gt| value.find('<').map_or(true, |lt| gt < lt));

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

fn normalize_ws(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
