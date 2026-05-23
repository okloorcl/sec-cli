use anyhow::{Context, Result};
use regex::Regex;

use crate::sec::{
    client::SecClient,
    documents::{DocumentSet, SubmissionDocument, read::plain_text},
    edgar::accession_document_url,
    models::{FilingQuery, FilingRecord, HtmlTableQuery, HtmlTableRecord},
};

impl SecClient {
    pub async fn html_tables(&self, query: HtmlTableQuery) -> Result<Vec<HtmlTableRecord>> {
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

        let mut records = Vec::new();
        for (filing, docs) in self.filing_documents_batch(filings).await? {
            let Some(doc) = DocumentSet::new(&docs).primary_documents().next() else {
                continue;
            };
            records.extend(extract_html_tables(
                &filing,
                doc,
                query.limit_tables,
                query.limit_rows,
            )?);
        }
        if let Some(limit) = query.limit_tables {
            records.truncate(limit);
        }
        Ok(records)
    }
}

pub fn extract_html_tables(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
    limit_tables: Option<usize>,
    limit_rows: Option<usize>,
) -> Result<Vec<HtmlTableRecord>> {
    let table_regex =
        Regex::new(r"(?is)<table\b[^>]*>.*?</table>").context("invalid HTML table regex")?;
    let mut records = Vec::new();

    for (idx, table_match) in table_regex.find_iter(&doc.content).enumerate() {
        if limit_tables.is_some_and(|limit| records.len() >= limit) {
            break;
        }
        let table_html = table_match.as_str();
        let rows = table_rows(table_html)?;
        if rows.is_empty() {
            continue;
        }
        let row_count = rows.len();
        let column_count = rows.iter().map(Vec::len).max().unwrap_or(0);
        let headers = rows.first().cloned().unwrap_or_default();
        let mut returned_rows = rows;
        let truncated = limit_rows.is_some_and(|limit| returned_rows.len() > limit);
        if let Some(limit) = limit_rows {
            returned_rows.truncate(limit);
        }

        records.push(HtmlTableRecord {
            accession: filing.accession.clone(),
            cik: filing.cik,
            company: filing.company.clone(),
            form: filing.form.clone(),
            filing_date: filing.filing_date.clone(),
            report_date: filing.report_date.clone(),
            table_index: idx + 1,
            title_hint: title_hint(&doc.content, table_match.start()),
            row_count,
            column_count,
            returned_rows: returned_rows.len(),
            truncated,
            headers,
            rows: returned_rows,
            document: doc.filename.clone(),
            document_sequence: doc.sequence.clone(),
            document_description: doc.description.clone(),
            document_url: doc
                .filename
                .as_deref()
                .map(|filename| accession_document_url(filing.cik, &filing.accession, filename)),
            source_url: filing.source_url.clone(),
        });
    }
    Ok(records)
}

fn table_rows(table_html: &str) -> Result<Vec<Vec<String>>> {
    let row_regex = Regex::new(r"(?is)<tr\b[^>]*>(.*?)</tr>").context("invalid row regex")?;
    let cell_regex =
        Regex::new(r"(?is)<t[dh]\b[^>]*>(.*?)</t[dh]>").context("invalid cell regex")?;
    let mut rows = Vec::new();
    for row_capture in row_regex.captures_iter(table_html) {
        let Some(row_html) = row_capture.get(1).map(|m| m.as_str()) else {
            continue;
        };
        let row: Vec<String> = cell_regex
            .captures_iter(row_html)
            .filter_map(|capture| capture.get(1).map(|m| clean_cell(m.as_str())))
            .filter(|cell| !cell.is_empty())
            .collect();
        if !row.is_empty() {
            rows.push(row);
        }
    }
    Ok(rows)
}

fn title_hint(content: &str, table_start: usize) -> Option<String> {
    let start = table_start.saturating_sub(1200);
    let mut fragment = &content[start..table_start];
    if let Some(first_gt) = fragment.find('>') {
        let first_lt = fragment.find('<').unwrap_or(fragment.len());
        if first_gt < first_lt {
            fragment = &fragment[first_gt + 1..];
        }
    }
    let prefix = plain_text(fragment);
    let hint = prefix
        .split(['.', '\n'])
        .next_back()
        .unwrap_or(prefix.as_str())
        .trim();
    (!hint.is_empty()).then(|| hint.chars().take(180).collect())
}

fn clean_cell(cell_html: &str) -> String {
    plain_text(cell_html)
        .trim()
        .trim_matches(['\u{200b}', '\u{feff}'])
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_simple_html_table() {
        let filing = FilingRecord {
            accession: "0000000000-26-000001".to_string(),
            cik: 1,
            company: "Example Inc.".to_string(),
            form: "10-K".to_string(),
            filing_date: "2026-01-01".to_string(),
            report_date: None,
            primary_document: Some("example.htm".to_string()),
            primary_doc_description: Some("10-K".to_string()),
            is_xbrl: None,
            is_inline_xbrl: None,
            source_url: "https://example.test/index.html".to_string(),
            text_url: "https://example.test/submission.txt".to_string(),
        };
        let doc = SubmissionDocument {
            document_type: Some("10-K".to_string()),
            sequence: Some("1".to_string()),
            filename: Some("example.htm".to_string()),
            description: Some("10-K".to_string()),
            content: "<h2>Revenue by segment</h2><table><tr><th>Segment</th><th>Revenue</th></tr><tr><td>Services</td><td>$10</td></tr></table>".to_string(),
        };

        let tables = extract_html_tables(&filing, &doc, Some(1), Some(10)).unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].headers, vec!["Segment", "Revenue"]);
        assert_eq!(tables[0].rows[1], vec!["Services", "$10"]);
    }
}
