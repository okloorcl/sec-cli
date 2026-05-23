use std::collections::{BTreeMap, HashSet};

use anyhow::{Context, Result, anyhow};
use regex::Regex;

use crate::sec::{
    client::SecClient,
    documents::{DocumentSet, SubmissionDocument, read::plain_text},
    edgar::accession_document_url,
    models::{FilingQuery, FilingRecord, Schedule13Query, Schedule13Record},
    parsers::text_helpers,
};

impl SecClient {
    pub async fn schedule13_reports(
        &self,
        query: Schedule13Query,
    ) -> Result<Vec<Schedule13Record>> {
        let forms = schedule13_forms(query.form.as_deref())?;
        let mut filings = Vec::new();
        let mut seen = HashSet::new();

        for form in forms {
            for filing in self
                .filings(FilingQuery {
                    cik: query.cik,
                    form: Some(form.to_string()),
                    latest: query.latest,
                    from: None,
                    to: None,
                    include_amends: query.include_amends,
                })
                .await?
            {
                if seen.insert(filing.accession.clone()) {
                    filings.push(filing);
                }
            }
        }

        filings.sort_by(|a, b| {
            b.filing_date
                .cmp(&a.filing_date)
                .then_with(|| b.accession.cmp(&a.accession))
        });
        filings.truncate(query.latest);

        let mut records = Vec::new();
        for filing in filings {
            let docs = self.filing_documents(&filing).await?;
            let Some(doc) = DocumentSet::new(&docs).primary_documents().next() else {
                continue;
            };
            records.push(parse_schedule13_report(&filing, doc, query.limit_bytes)?);
        }
        Ok(records)
    }
}

pub fn parse_schedule13_report(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
    limit_bytes: Option<usize>,
) -> Result<Schedule13Record> {
    let text = plain_text(&doc.content);
    let filing_type = filing_type(&filing.form);
    let items = item_spans(&text, &filing_type)?;
    let item1 = item_content(&items, 1);
    let item2 = item_content(&items, 2);
    let item4 = item_content(&items, 4);
    let item5 = item_content(&items, 5);
    let ownership_text = item4.unwrap_or(&text);

    let purpose = (filing_type == "13d")
        .then(|| item4.and_then(|value| truncated(value, limit_bytes).0))
        .flatten();
    let ownership = item4
        .or(item5)
        .and_then(|value| truncated(value, limit_bytes).0);

    Ok(Schedule13Record {
        accession: filing.accession.clone(),
        cik: filing.cik,
        company: filing.company.clone(),
        form: filing.form.clone(),
        filing_date: filing.filing_date.clone(),
        report_date: filing.report_date.clone(),
        filing_type,
        is_amendment: filing.form.to_ascii_uppercase().ends_with("/A"),
        activist_intent: filing.form.to_ascii_uppercase().contains("13D"),
        issuer_name: capture_first(
            item1.unwrap_or(&text),
            &[r"(?i)Name of Issuer:\s*(.*?)\s*(?:\([bB]\)|Address of Issuer|Item\s+2[\.\(])"],
        )
        .or_else(|| capture_before_marker(&text, r"(?i)\(Name of Issuer\)")),
        issuer_address: capture_first(
            item1.unwrap_or(&text),
            &[r"(?i)Address of Issuer[^:]*:\s*(.*?)\s*(?:Item\s+2[\.\(]|$)"],
        ),
        security_title: capture_first(
            item2.or(item1).unwrap_or(&text),
            &[r"(?i)Title of Class of Securities:\s*(.*?)\s*(?:\([eE]\)|CUSIP Number|Item\s+3\.)"],
        ),
        cusip: capture_first(
            item2.or(item1).unwrap_or(&text),
            &[r"(?i)CUSIP Number:\s*([A-Z0-9 \-]+)"],
        ),
        event_date: capture_first(
            &text,
            &[r"(?i)([A-Z][a-z]+ \d{1,2}, \d{4})\s*\(Date of Event Which Requires Filing"],
        ),
        reporting_persons: reporting_persons(&text, item2),
        filing_rule: filing_rule(&text),
        citizenship_or_organization: capture_first(
            item2.unwrap_or(&text),
            &[
                r"(?i)Citizenship or Place of Organization\s*(.*?)\s*(?:Number of Shares|5\.|Item\s+3\.)",
                r"(?i)Citizenship:\s*(.*?)\s*(?:\([dD]\)|Title of Class|Item\s+3\.)",
            ],
        ),
        beneficially_owned_shares: number_after(
            ownership_text,
            &[
                r"(?i)Aggregate Amount Beneficially Owned by Each Reporting Person\s*",
                r"(?i)Amount beneficially owned:\s*",
            ],
        )
        .or_else(|| {
            number_after(
                &text,
                &[r"(?i)Aggregate Amount Beneficially Owned by Each Reporting Person\s*"],
            )
        }),
        percent_of_class: percent_after(
            ownership_text,
            &[
                r"(?i)Percent of Class Represented by Amount in Row \(9\)\s*",
                r"(?i)Percent of class:\s*",
            ],
        )
        .or_else(|| {
            percent_after(
                &text,
                &[r"(?i)Percent of Class Represented by Amount in Row \(9\)\s*"],
            )
        }),
        sole_voting_power: number_after(
            ownership_text,
            &[
                r"(?i)Sole Voting Power\s*",
                r"(?i)Sole power to vote or to direct the vote:\s*",
            ],
        )
        .or_else(|| number_after(&text, &[r"(?i)Sole Voting Power\s*"])),
        shared_voting_power: number_after(
            ownership_text,
            &[
                r"(?i)Shared Voting Power\s*",
                r"(?i)Shared power to vote or to direct the vote:\s*",
            ],
        )
        .or_else(|| number_after(&text, &[r"(?i)Shared Voting Power\s*"])),
        sole_dispositive_power: number_after(
            ownership_text,
            &[
                r"(?i)Sole Dispositive Power\s*",
                r"(?i)Sole power to dispose or to direct the disposition of:\s*",
            ],
        )
        .or_else(|| number_after(&text, &[r"(?i)Sole Dispositive Power\s*"])),
        shared_dispositive_power: number_after(
            ownership_text,
            &[
                r"(?i)Shared Dispositive Power\s*",
                r"(?i)Shared power to dispose or to direct the disposition of:\s*",
            ],
        )
        .or_else(|| number_after(&text, &[r"(?i)Shared Dispositive Power\s*"])),
        purpose_of_transaction: purpose,
        ownership_summary: ownership,
        item_count: items.len(),
        signatures: signatures(&text),
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

fn schedule13_forms(form: Option<&str>) -> Result<Vec<&'static str>> {
    let Some(form) = form else {
        return Ok(vec!["SC 13D", "SC 13G"]);
    };
    match form.trim().to_ascii_uppercase().replace('-', " ").as_str() {
        "" | "ALL" => Ok(vec!["SC 13D", "SC 13G"]),
        "13D" | "SC 13D" | "SC13D" => Ok(vec!["SC 13D"]),
        "13G" | "SC 13G" | "SC13G" => Ok(vec!["SC 13G"]),
        "SC 13D/A" | "SC13D/A" | "13D/A" => Ok(vec!["SC 13D/A"]),
        "SC 13G/A" | "SC13G/A" | "13G/A" => Ok(vec!["SC 13G/A"]),
        other => Err(anyhow!("unsupported Schedule 13 form '{}'", other)),
    }
}

fn item_spans(text: &str, filing_type: &str) -> Result<Vec<ItemSpan>> {
    let max_item = if filing_type == "13d" { 7 } else { 10 };
    let regex = Regex::new(r"(?i)\bItem\s+(10|[1-9])(?:\s*\.|\s*\([a-z]\)\s*[-–—:])\s*")
        .context("invalid Schedule 13 item regex")?;
    let mut spans_by_item: BTreeMap<u8, ItemSpan> = BTreeMap::new();
    let mut headings = Vec::new();

    for capture in regex.captures_iter(text) {
        let Some(full) = capture.get(0) else {
            continue;
        };
        let item = capture
            .get(1)
            .and_then(|m| m.as_str().parse::<u8>().ok())
            .unwrap_or(0);
        if item == 0 || item > max_item {
            continue;
        }
        headings.push((item, full.start()));
    }

    for (idx, (item, start)) in headings.iter().enumerate() {
        let end = headings
            .get(idx + 1)
            .map(|(_, next)| *next)
            .unwrap_or(text.len());
        if end <= *start {
            continue;
        }
        let content = text[*start..end].trim().to_string();
        if content.len() < 16 {
            continue;
        }
        let span = ItemSpan {
            item: *item,
            content,
        };
        spans_by_item.entry(*item).or_insert(span);
    }
    Ok(spans_by_item.into_values().collect())
}

#[derive(Debug, Clone)]
struct ItemSpan {
    item: u8,
    content: String,
}

fn item_content(items: &[ItemSpan], item: u8) -> Option<&str> {
    items
        .iter()
        .find(|span| span.item == item)
        .map(|span| span.content.as_str())
}

fn filing_type(form: &str) -> String {
    if form.to_ascii_uppercase().contains("13D") {
        "13d".to_string()
    } else {
        "13g".to_string()
    }
}

fn capture_first(text: &str, patterns: &[&str]) -> Option<String> {
    for pattern in patterns {
        let Ok(regex) = Regex::new(pattern) else {
            continue;
        };
        if let Some(value) = regex
            .captures(text)
            .and_then(|captures| captures.get(1))
            .map(|m| clean_value(m.as_str()))
            .filter(|value| !value.is_empty())
        {
            return Some(value);
        }
    }
    None
}

fn capture_before_marker(text: &str, marker: &str) -> Option<String> {
    let Ok(regex) = Regex::new(&format!(r"([^()]+?)\s*{marker}")) else {
        return None;
    };
    regex
        .captures_iter(text)
        .filter_map(|captures| captures.get(1).map(|m| clean_value(m.as_str())))
        .filter(|value| !value.is_empty())
        .last()
}

fn reporting_persons(text: &str, item2: Option<&str>) -> Vec<String> {
    let mut people = Vec::new();
    if let Ok(regex) = Regex::new(r"(?i)\b1\.\s*Names? of Reporting Persons?\.?\s*(.*?)\s+2\.") {
        for capture in regex.captures_iter(text) {
            if let Some(value) = capture.get(1).map(|m| clean_value(m.as_str())) {
                push_unique(&mut people, value);
            }
        }
    }
    if people.is_empty() {
        if let Some(value) = item2.and_then(|text| {
            capture_first(
                text,
                &[r"(?i)Name of Person Filing:\s*(.*?)\s*(?:\([bB]\)|Address)"],
            )
        }) {
            push_unique(&mut people, value);
        }
    }
    people
}

fn filing_rule(text: &str) -> Option<String> {
    capture_first(
        text,
        &[
            r"(?i)[☒xX]\s*Rule\s*(13d-[^\s;]+)",
            r"(?i)Rule\s*(13d-[^\s;]+)\s*[☒xX]",
        ],
    )
    .map(|value| {
        let normalized = value.replace("13d-l", "13d-1");
        if normalized.contains('(') && !normalized.ends_with(')') {
            format!("{normalized})")
        } else {
            normalized
        }
    })
}

fn number_after(text: &str, labels: &[&str]) -> Option<f64> {
    for label in labels {
        let pattern = format!("{label}[^0-9-]*([0-9][0-9,]*(?:\\.\\d+)?)");
        if let Some(value) = capture_first(text, &[&pattern]).and_then(|value| parse_number(&value))
        {
            return Some(value);
        }
    }
    None
}

fn percent_after(text: &str, labels: &[&str]) -> Option<f64> {
    for label in labels {
        let pattern = format!("{label}[^0-9-]*([0-9]+(?:\\.[0-9]+)?)\\s*%");
        if let Some(value) = capture_first(text, &[&pattern]).and_then(|value| parse_number(&value))
        {
            return Some(value);
        }
    }
    None
}

fn signatures(text: &str) -> Vec<String> {
    let Ok(regex) = Regex::new(r"/s/\s*([A-Za-z0-9 .,'&\-]+)") else {
        return Vec::new();
    };
    let mut values = Vec::new();
    for capture in regex.captures_iter(text) {
        if let Some(value) = capture.get(1).map(|m| clean_signature(m.as_str())) {
            push_unique(&mut values, value);
        }
    }
    values
}

fn truncated(value: &str, limit: Option<usize>) -> (Option<String>, bool) {
    let text = clean_value(value);
    if text.is_empty() {
        return (None, false);
    }
    let Some(limit) = limit else {
        return (Some(text), false);
    };
    if text.len() <= limit {
        return (Some(text), false);
    }
    let mut end = limit.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    (Some(text[..end].to_string()), true)
}

fn parse_number(value: &str) -> Option<f64> {
    value.replace(',', "").parse::<f64>().ok()
}

fn clean_value(value: &str) -> String {
    let trimmed = value
        .split('|')
        .next()
        .unwrap_or(value)
        .trim()
        .trim_matches([':', '.', ';', ',', '(', ')']);
    text_helpers::clean_text(trimmed)
}

fn clean_signature(value: &str) -> String {
    let value = value
        .split(" Page ")
        .next()
        .unwrap_or(value)
        .split(" Name:")
        .next()
        .unwrap_or(value);
    let cleaned = clean_value(value);
    let parts: Vec<&str> = cleaned.split_whitespace().collect();
    if parts.len() % 2 == 0 {
        let half = parts.len() / 2;
        if parts[..half] == parts[half..] {
            return parts[..half].join(" ");
        }
    }
    cleaned
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !value.is_empty() && !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_schedule_13g_core_fields() {
        let filing = FilingRecord {
            accession: "0001193125-24-036110".to_string(),
            cik: 1,
            company: "Tesla, Inc.".to_string(),
            form: "SC 13G/A".to_string(),
            filing_date: "2024-02-14".to_string(),
            report_date: None,
            primary_document: Some("doc.htm".to_string()),
            primary_doc_description: Some("SC 13G/A".to_string()),
            is_xbrl: None,
            is_inline_xbrl: None,
            source_url: "https://example.test/index.html".to_string(),
            text_url: "https://example.test/submission.txt".to_string(),
        };
        let doc = SubmissionDocument {
            sequence: Some("1".to_string()),
            filename: Some("doc.htm".to_string()),
            description: Some("SC 13G/A".to_string()),
            document_type: Some("SC 13G/A".to_string()),
            content: SAMPLE_13G.to_string(),
        };

        let record = parse_schedule13_report(&filing, &doc, Some(220)).unwrap();
        assert_eq!(record.filing_type, "13g");
        assert!(!record.activist_intent);
        assert_eq!(record.issuer_name.as_deref(), Some("Tesla, Inc"));
        assert_eq!(record.reporting_persons, vec!["Elon R. Musk"]);
        assert_eq!(record.beneficially_owned_shares, Some(715_022_706.0));
        assert_eq!(record.percent_of_class, Some(20.5));
        assert_eq!(record.sole_voting_power, Some(715_022_706.0));
        assert_eq!(record.item_count, 10);
    }

    const SAMPLE_13G: &str = "SCHEDULE 13G/A Tesla, Inc. (Name of Issuer) Common Stock (Title of Class of Securities) 88160R 101 (CUSIP Number) December 31, 2023 (Date of Event Which Requires Filing of this Statement) ☒ Rule 13d-l(d) 1. Names of Reporting Persons. Elon R. Musk 2. Check the Appropriate Box 4. Citizenship or Place of Organization United States 5. Sole Voting Power 715,022,706 6. Shared Voting Power 715,022,706 7. Sole Dispositive Power 715,022,706 8. Shared Dispositive Power 715,022,706 9. Aggregate Amount Beneficially Owned by Each Reporting Person 715,022,706 11. Percent of Class Represented by Amount in Row (9) 20.5% Item 1. (a) Name of Issuer: Tesla, Inc. (b) Address of Issuer Principal Executive Offices: 1 Tesla Road Item 2. (a) Name of Person Filing: Elon R. Musk (b) Address (c) Citizenship: United States (d) Title of Class of Securities: Common Stock (e) CUSIP Number: 88160R 101 Item 3. If this statement is filed pursuant to Rules Item 4. Ownership. (a) Amount beneficially owned:715,022,706 shares (b) Percent of class: 20.5% (c) Number of shares as to which the person has: (i) Sole power to vote or to direct the vote: 715,022,706 (ii) Shared power to vote or to direct the vote: 715,022,706 (iii) Sole power to dispose or to direct the disposition of: 715,022,706 (iv) Shared power to dispose or to direct the disposition of: 715,022,706 Item 5. Ownership of Five Percent or Less of a Class. Not applicable. Item 6. Ownership of More than Five Percent on Behalf of Another Person. Not applicable. Item 7. Identification and Classification. Not applicable. Item 8. Identification and Classification of Members of the Group. Not applicable. Item 9. Notice of Dissolution of Group. Not applicable. Item 10. Certifications. Not applicable. SIGNATURE /s/ Elon R. Musk Elon R. Musk";
}
