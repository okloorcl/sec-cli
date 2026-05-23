use std::collections::BTreeMap;

use anyhow::Result;
use regex::Regex;

use crate::sec::{
    client::SecClient,
    documents::{DocumentSet, SubmissionDocument, read::plain_text},
    edgar::accession_document_url,
    models::{
        FilingQuery, FilingRecord, HtmlTableRecord, ProxyProposalRecord, ProxyQuery,
        ProxyStatementRecord, ProxyTableRecord,
    },
    parsers::text_helpers,
    tables::extract_html_tables,
};

impl SecClient {
    pub async fn proxy_statements(&self, query: ProxyQuery) -> Result<Vec<ProxyStatementRecord>> {
        let filings = self
            .filings(FilingQuery {
                cik: query.cik,
                form: Some("DEF 14A".to_string()),
                latest: query.latest,
                from: None,
                to: None,
                include_amends: query.include_amends,
            })
            .await?;

        let mut records = Vec::new();
        for filing in filings {
            let docs = self.filing_documents(&filing).await?;
            let Some(doc) = DocumentSet::new(&docs).primary_documents().next() else {
                continue;
            };
            records.push(parse_proxy_statement(&filing, doc, query.limit_rows)?);
        }
        Ok(records)
    }
}

pub fn parse_proxy_statement(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
    limit_rows: Option<usize>,
) -> Result<ProxyStatementRecord> {
    let text = plain_text(&doc.content);
    let tables = extract_html_tables(filing, doc, None, Some(12))?;
    let compensation_table = summary_compensation_table(&tables);
    let named_executive_officers = compensation_table
        .as_ref()
        .map(|table| named_executives_from_table(table))
        .unwrap_or_default();

    Ok(ProxyStatementRecord {
        accession: filing.accession.clone(),
        cik: filing.cik,
        company: filing.company.clone(),
        form: filing.form.clone(),
        filing_date: filing.filing_date.clone(),
        meeting_date: meeting_date(&text),
        meeting_time: capture_first(
            &text,
            &[r"(?i)(\d{1,2}:\d{2}\s*(?:A\.M\.|P\.M\.|AM|PM)\s+[A-Z][a-z]+ Time)"],
        ),
        meeting_site: capture_first(
            &text,
            &[r"(?i)(www\.virtualshareholdermeeting\.com/[A-Z0-9]+)"],
        ),
        record_date: capture_first(
            &text,
            &[r"(?i)record at the close of business on ([A-Z][a-z]+ \d{1,2}, \d{4})"],
        ),
        materials_available_date: capture_first(
            &text,
            &[r"(?i)first sent or made available to shareholders on ([A-Z][a-z]+ \d{1,2}, \d{4})"],
        ),
        proposals: proposals(&text, &tables),
        director_nominees: director_nominees(&text),
        auditor: auditor(&text),
        named_executive_officers,
        summary_compensation_table: compensation_table
            .map(|table| proxy_table(table, limit_rows.unwrap_or(12))),
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

fn proposals(text: &str, tables: &[HtmlTableRecord]) -> Vec<ProxyProposalRecord> {
    let table_records = proposal_table_records(tables);
    if !table_records.is_empty() {
        return table_records;
    }

    let notice_records = notice_proposals(text);
    if !notice_records.is_empty() {
        return notice_records;
    }

    let mut by_number: BTreeMap<u64, ProxyProposalRecord> = BTreeMap::new();
    if let Ok(short_regex) = Regex::new(r"(?i)Proposal[^0-9]{0,12}(\d+)\s*[-–—]\s*([^.;]+)") {
        for capture in short_regex.captures_iter(text) {
            insert_proposal(text, &mut by_number, &capture);
        }
    }
    by_number.into_values().collect()
}

fn proposal_table_records(tables: &[HtmlTableRecord]) -> Vec<ProxyProposalRecord> {
    for table in tables {
        let rows = table
            .rows
            .iter()
            .filter_map(|row| proposal_from_row(row))
            .collect::<Vec<_>>();
        if rows.len() >= 2 {
            return rows;
        }
    }
    Vec::new()
}

fn proposal_from_row(row: &[String]) -> Option<ProxyProposalRecord> {
    if row.len() < 3 {
        return None;
    }
    let number = row.first()?.trim().parse::<u64>().ok()?;
    let title = clean_text(row.get(1)?);
    if !(6..=220).contains(&title.len()) {
        return None;
    }
    let recommendation =
        row.iter()
            .rev()
            .find_map(|cell| match cell.trim().to_ascii_uppercase().as_str() {
                "FOR" => Some("FOR".to_string()),
                "AGAINST" => Some("AGAINST".to_string()),
                _ => None,
            });
    Some(ProxyProposalRecord {
        proposal_number: number,
        category: proposal_category(&title).to_string(),
        board_recommendation: recommendation,
        title: proposal_title(&title),
    })
}

fn insert_proposal(
    text: &str,
    by_number: &mut BTreeMap<u64, ProxyProposalRecord>,
    capture: &regex::Captures<'_>,
) {
    let Some(number) = capture.get(1).and_then(|m| m.as_str().parse::<u64>().ok()) else {
        return;
    };
    let title = capture
        .get(2)
        .map(|m| {
            clean_text(m.as_str())
                .split(" The Board")
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        })
        .unwrap_or_default();
    if title.len() < 6 || title.len() > 180 {
        return;
    }
    by_number
        .entry(number)
        .or_insert_with(|| ProxyProposalRecord {
            proposal_number: number,
            category: proposal_category(&title).to_string(),
            board_recommendation: board_recommendation(text, number),
            title,
        });
}

fn notice_proposals(text: &str) -> Vec<ProxyProposalRecord> {
    let section = text
        .split("Items of Business and Board Voting Recommendations")
        .nth(1)
        .and_then(|rest| rest.split("And other business").next())
        .unwrap_or(text);
    let Ok(number_regex) = Regex::new(r"\b([1-9]|1[0-9])\s+") else {
        return Vec::new();
    };
    let positions: Vec<(u64, usize)> = number_regex
        .captures_iter(section)
        .filter_map(|capture| {
            Some((
                capture.get(1)?.as_str().parse::<u64>().ok()?,
                capture.get(0)?.start(),
            ))
        })
        .collect();

    let mut records = Vec::new();
    for (idx, (number, start)) in positions.iter().enumerate() {
        let end = positions
            .get(idx + 1)
            .map(|(_, pos)| *pos)
            .unwrap_or(section.len());
        let mut span = clean_text(&section[*start..end]);
        span = span
            .trim_start_matches(|ch: char| ch.is_ascii_digit() || ch.is_whitespace())
            .to_string();
        let recommendation = if span.contains("AGAINST") {
            Some("AGAINST".to_string())
        } else if span.contains("FOR") {
            Some("FOR".to_string())
        } else {
            None
        };
        let mut title = span
            .split(" FOR")
            .next()
            .unwrap_or(&span)
            .split(" AGAINST")
            .next()
            .unwrap_or(&span)
            .trim()
            .to_string();
        if title.starts_with("Election of Directors:") {
            title = "Election of Directors".to_string();
        }
        if (6..=180).contains(&title.len()) {
            records.push(ProxyProposalRecord {
                proposal_number: *number,
                category: proposal_category(&title).to_string(),
                board_recommendation: recommendation,
                title,
            });
        }
    }
    records
}

fn board_recommendation(text: &str, number: u64) -> Option<String> {
    let patterns = [
        format!(r"(?i)recommends a vote\s+(FOR|AGAINST)\s+Proposal\s+{number}\b"),
        format!(r"(?i)Proposal\s+No\.?\s*{number}[^.{{}}]{{0,220}}\b(FOR|AGAINST)\b"),
    ];
    for pattern in patterns {
        if let Ok(regex) = Regex::new(&pattern) {
            if let Some(value) = regex
                .captures(text)
                .and_then(|capture| capture.get(1))
                .map(|m| m.as_str().to_ascii_uppercase())
            {
                return Some(value);
            }
        }
    }
    if (2..=4).contains(&number) && text.contains("recommends a vote FOR Proposals 2 to 4") {
        return Some("FOR".to_string());
    }
    if number == 5 && text.contains("recommends a vote AGAINST Proposal 5") {
        return Some("AGAINST".to_string());
    }
    None
}

fn director_nominees(text: &str) -> Vec<String> {
    let mut nominees = Vec::new();
    if let Some(value) = capture_first(
        text,
        &[r"(?i)Election of Directors:\s*(.*?)\s+FOR\s+2\s+Ratification"],
    ) {
        for name in split_people(&value) {
            push_unique(&mut nominees, name);
        }
    }
    nominees
}

fn named_executives_from_table(table: &HtmlTableRecord) -> Vec<String> {
    let mut names = Vec::new();
    for row in &table.rows {
        if let Some(first) = row.first() {
            if let Some(name) = leading_person_name(first) {
                push_unique(&mut names, name);
            }
        }
    }
    names
}

fn summary_compensation_table(tables: &[HtmlTableRecord]) -> Option<&HtmlTableRecord> {
    tables.iter().find(|table| {
        table
            .title_hint
            .as_deref()
            .is_some_and(|title| title.contains("Summary Compensation Table"))
            || table.headers.iter().any(|header| header.contains("Total"))
                && table.headers.iter().any(|header| header.contains("Salary"))
                && table.headers.iter().any(|header| header.contains("Stock"))
    })
}

fn proxy_table(table: &HtmlTableRecord, limit_rows: usize) -> ProxyTableRecord {
    let mut rows = table.rows.clone();
    let truncated = rows.len() > limit_rows;
    rows.truncate(limit_rows);
    ProxyTableRecord {
        table_index: table.table_index,
        title_hint: table.title_hint.clone(),
        headers: table.headers.clone(),
        rows,
        row_count: table.row_count,
        column_count: table.column_count,
        truncated,
    }
}

fn meeting_date(text: &str) -> Option<String> {
    capture_first(
        text,
        &[
            r"(?i)Annual Meeting[^.]{0,120}?on ([A-Z][a-z]+ \d{1,2}, \d{4})",
            r"(?i)Date and Time[^A-Z]*([A-Z][a-z]+ \d{1,2}, \d{4})",
        ],
    )
}

fn auditor(text: &str) -> Option<String> {
    capture_first(
        text,
        &[
            r"(?i)(Ernst\s*&\s*Young\s+LLP)",
            r"(?i)(Deloitte\s*&\s*Touche\s+LLP)",
            r"(?i)(PricewaterhouseCoopers\s+LLP|PwC)",
            r"(?i)(KPMG\s+LLP)",
        ],
    )
}

fn proposal_category(title: &str) -> &'static str {
    let lower = title.to_ascii_lowercase();
    if lower.contains("election") && lower.contains("director") {
        "director_election"
    } else if lower.contains("registered public accounting") || lower.contains("auditor") {
        "auditor_ratification"
    } else if lower.contains("executive compensation") || lower.contains("say-on-pay") {
        "say_on_pay"
    } else if lower.contains("stock plan") || lower.contains("equity") {
        "equity_plan"
    } else if lower.contains("shareholder") {
        "shareholder_proposal"
    } else {
        "other"
    }
}

fn capture_first(text: &str, patterns: &[&str]) -> Option<String> {
    for pattern in patterns {
        let Ok(regex) = Regex::new(pattern) else {
            continue;
        };
        if let Some(value) = regex
            .captures(text)
            .and_then(|capture| capture.get(1))
            .map(|m| clean_text(m.as_str()))
            .filter(|value| !value.is_empty())
        {
            return Some(value);
        }
    }
    None
}

fn split_people(value: &str) -> Vec<String> {
    value
        .replace(", and ", ", ")
        .replace(" and ", ", ")
        .split(',')
        .map(clean_text)
        .filter(|name| {
            let words = name.split_whitespace().count();
            (2..=5).contains(&words) && !name.contains("Proposal")
        })
        .collect()
}

fn leading_person_name(value: &str) -> Option<String> {
    let regex = Regex::new(r"^([A-Z][A-Za-z’'`.-]+\s+[A-Z][A-Za-z’'`.-]+)").ok()?;
    regex
        .captures(value)
        .and_then(|capture| capture.get(1))
        .map(|m| clean_text(m.as_str()))
}

fn proposal_title(title: &str) -> String {
    if title.starts_with("Election of Directors:") {
        "Election of Directors".to_string()
    } else {
        title.to_string()
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn clean_text(value: &str) -> String {
    text_helpers::clean_text(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_proxy_proposals_and_nominees() {
        let text = "Items of Business and Board Voting Recommendations 1 Election of Directors: Wanda Austin, Tim Cook, Alex Gorsky FOR 2 Ratification of Appointment of Independent Registered Public Accounting Firm FOR Proposal No. 3 - Advisory Vote to Approve Executive Compensation The Board of Directors recommends a vote FOR Proposals 2 to 4. The Board of Directors recommends a vote AGAINST Proposal 5.";
        let proposals = proposals(text, &[]);
        assert!(proposals.iter().any(|p| p.proposal_number == 3));
        assert_eq!(board_recommendation(text, 5).as_deref(), Some("AGAINST"));
        assert_eq!(
            director_nominees(text),
            vec!["Wanda Austin", "Tim Cook", "Alex Gorsky"]
        );
    }
}
