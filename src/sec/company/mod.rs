use anyhow::Result;

use crate::sec::{
    client::SecClient,
    documents::DocumentSet,
    edgar::accession_document_url,
    models::{
        CompanyReportQuery, CompanyReportRecord, CompanyTopicTableRecord, FilingQuery,
        HtmlTableRecord,
    },
    tables::extract_html_tables,
};

impl SecClient {
    pub async fn company_reports(
        &self,
        query: CompanyReportQuery,
    ) -> Result<Vec<CompanyReportRecord>> {
        let filings = self
            .filings(FilingQuery {
                cik: query.cik,
                form: query.form.clone().or_else(|| Some("10-K".to_string())),
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
            let tables = extract_html_tables(&filing, doc, None, query.limit_rows)?;
            let mut topics = classify_tables(&tables, query.topic.as_deref());
            if let Some(limit) = query.limit_tables {
                topics.truncate(limit);
            }

            records.push(CompanyReportRecord {
                accession: filing.accession.clone(),
                cik: filing.cik,
                company: filing.company.clone(),
                form: filing.form.clone(),
                filing_date: filing.filing_date.clone(),
                report_date: filing.report_date.clone(),
                matched_table_count: topics.len(),
                scanned_table_count: tables.len(),
                topics,
                document: doc.filename.clone(),
                document_url: doc.filename.as_deref().map(|filename| {
                    accession_document_url(filing.cik, &filing.accession, filename)
                }),
                source_url: filing.source_url.clone(),
            });
        }
        Ok(records)
    }
}

fn classify_tables(
    tables: &[HtmlTableRecord],
    topic_filter: Option<&str>,
) -> Vec<CompanyTopicTableRecord> {
    let filter = topic_filter.map(normalize_topic);
    let mut records = tables
        .iter()
        .filter_map(|table| classify_table(table))
        .filter(|record| {
            filter
                .as_deref()
                .is_none_or(|filter| record.topic == filter || record.topic.contains(filter))
        })
        .collect::<Vec<_>>();
    records.sort_by(|a, b| {
        b.confidence
            .total_cmp(&a.confidence)
            .then_with(|| a.table_index.cmp(&b.table_index))
    });
    records
}

fn classify_table(table: &HtmlTableRecord) -> Option<CompanyTopicTableRecord> {
    let corpus = table_corpus(table);
    let candidates = [
        topic_score(
            "segment_revenue",
            &corpus,
            &["segment", "reportable segment", "business segment"],
            &["revenue", "net sales", "sales"],
        ),
        topic_score(
            "geographic_revenue",
            &corpus,
            &[
                "geographic",
                "country",
                "americas",
                "europe",
                "china",
                "japan",
            ],
            &["revenue", "net sales", "sales"],
        ),
        topic_score(
            "revenue_disaggregation",
            &corpus,
            &["revenue by", "disaggregated revenue", "product", "service"],
            &["revenue", "net sales", "sales"],
        ),
        topic_score(
            "debt_maturity",
            &corpus,
            &["maturities", "long-term debt", "notes payable", "debt"],
            &["202", "thereafter", "principal", "interest"],
        ),
        topic_score(
            "contract_obligations",
            &corpus,
            &[
                "contractual obligations",
                "purchase obligations",
                "commitments",
            ],
            &["payments due", "thereafter", "operating lease"],
        ),
        topic_score(
            "lease_maturity",
            &corpus,
            &["lease", "leases", "right-of-use"],
            &["maturity", "thereafter", "future minimum"],
        ),
        topic_score(
            "share_repurchases",
            &corpus,
            &["repurchase", "share repurchase", "treasury stock"],
            &["shares", "amount", "average price"],
        ),
        topic_score(
            "tax_rate",
            &corpus,
            &["tax", "income tax", "effective tax"],
            &["rate", "benefit", "expense"],
        ),
    ];

    let (topic, score) =
        candidates
            .into_iter()
            .fold(None, |best: Option<(&'static str, f64)>, candidate| {
                Some(match best {
                    Some(best) if best.1 >= candidate.1 => best,
                    _ => candidate,
                })
            })?;
    if score < 3.0 {
        return None;
    }

    Some(CompanyTopicTableRecord {
        topic: topic.to_string(),
        confidence: (score / 8.0).min(1.0),
        table_index: table.table_index,
        title_hint: table.title_hint.clone(),
        row_count: table.row_count,
        column_count: table.column_count,
        returned_rows: table.returned_rows,
        truncated: table.truncated,
        headers: table.headers.clone(),
        rows: table.rows.clone(),
    })
}

fn topic_score(
    topic: &'static str,
    corpus: &str,
    primary: &[&str],
    secondary: &[&str],
) -> (&'static str, f64) {
    let primary_hits = primary
        .iter()
        .filter(|term| corpus.contains(**term))
        .count() as f64;
    let secondary_hits = secondary
        .iter()
        .filter(|term| corpus.contains(**term))
        .count() as f64;
    (topic, primary_hits * 2.0 + secondary_hits)
}

fn table_corpus(table: &HtmlTableRecord) -> String {
    let mut text = String::new();
    if let Some(title) = &table.title_hint {
        text.push_str(title);
        text.push(' ');
    }
    for header in &table.headers {
        text.push_str(header);
        text.push(' ');
    }
    for row in table.rows.iter().take(8) {
        for cell in row {
            text.push_str(cell);
            text.push(' ');
        }
    }
    text.to_ascii_lowercase()
}

fn normalize_topic(topic: &str) -> String {
    topic.trim().to_ascii_lowercase().replace('-', "_")
}

#[cfg(test)]
mod tests {
    use crate::sec::models::HtmlTableRecord;

    use super::classify_table;

    fn table(title: &str, headers: &[&str], rows: Vec<Vec<&str>>) -> HtmlTableRecord {
        HtmlTableRecord {
            accession: "000".to_string(),
            cik: 1,
            company: "Example".to_string(),
            form: "10-K".to_string(),
            filing_date: "2026-01-01".to_string(),
            report_date: None,
            table_index: 1,
            title_hint: Some(title.to_string()),
            row_count: rows.len() + 1,
            column_count: headers.len(),
            returned_rows: rows.len() + 1,
            truncated: false,
            headers: headers.iter().map(|value| value.to_string()).collect(),
            rows: rows
                .into_iter()
                .map(|row| row.into_iter().map(str::to_string).collect())
                .collect(),
            document: None,
            document_sequence: None,
            document_description: None,
            document_url: None,
            source_url: "https://example.test".to_string(),
        }
    }

    #[test]
    fn classifies_segment_and_debt_tables() {
        let segment = table(
            "Revenue by reportable segment",
            &["Segment", "Net sales"],
            vec![vec!["Services", "$10"]],
        );
        assert_eq!(classify_table(&segment).unwrap().topic, "segment_revenue");

        let debt = table(
            "Long-term debt maturities",
            &["2027", "2028", "Thereafter"],
            vec![vec!["Principal payments", "$10", "$20", "$30"]],
        );
        assert_eq!(classify_table(&debt).unwrap().topic, "debt_maturity");
    }
}
