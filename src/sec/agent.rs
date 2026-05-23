use std::collections::BTreeSet;

use anyhow::Result;

use crate::sec::{
    client::SecClient,
    models::{
        AgentPackQuery, AgentPackRecord, FilingQuery, HealthScoreQuery, MetricsQuery, SectionQuery,
    },
};

impl SecClient {
    pub async fn agent_pack(&self, query: AgentPackQuery) -> Result<AgentPackRecord> {
        let filings = self
            .filings(FilingQuery {
                cik: query.cik,
                form: Some(query.form.clone()),
                latest: query.latest,
                from: None,
                to: None,
                include_amends: false,
            })
            .await?;

        let mut sections = Vec::new();
        for item in &query.sections {
            sections.extend(
                self.sections(SectionQuery {
                    cik: query.cik,
                    form: Some(query.form.clone()),
                    latest: query.latest,
                    include_amends: false,
                    accession: None,
                    item: item.clone(),
                    limit_bytes: query.section_limit_bytes,
                })
                .await?,
            );
        }

        let metrics = self
            .financial_metrics(MetricsQuery {
                cik: query.cik,
                form: Some("10-K".to_string()),
                unit: None,
                latest: query.metrics_latest,
            })
            .await?;
        let scores = self
            .health_scores(HealthScoreQuery {
                cik: query.cik,
                form: Some("10-K".to_string()),
                unit: None,
                latest: 1,
            })
            .await?;

        let source_urls = source_urls(&filings, &sections, &metrics, &scores);
        let record = AgentPackRecord {
            cik: query.cik,
            form: query.form.clone(),
            latest: query.latest,
            sections_requested: query.sections.clone(),
            filing_count: filings.len(),
            section_count: sections.len(),
            metric_count: metrics.len(),
            score_count: scores.len(),
            filings,
            sections,
            metrics,
            scores,
            source_urls,
            next_commands: vec![
                format!(
                    "sec archive --cik {} --form {} --latest {} --primary-only --out-dir ./archives/{}",
                    query.cik, query.form, query.latest, query.cik
                ),
                format!(
                    "sec export --kind metrics --cik {} --period annual --latest {} --format parquet --out metrics.parquet",
                    query.cik, query.metrics_latest
                ),
                format!(
                    "sec report --cik {} --kind financial --latest {}",
                    query.cik, query.latest
                ),
            ],
        };
        Ok(record)
    }
}

fn source_urls(
    filings: &[crate::sec::models::FilingRecord],
    sections: &[crate::sec::models::SectionRecord],
    metrics: &[crate::sec::models::FinancialMetricRecord],
    scores: &[crate::sec::models::HealthScoreRecord],
) -> Vec<String> {
    let mut urls = BTreeSet::new();
    urls.extend(filings.iter().map(|record| record.source_url.clone()));
    urls.extend(sections.iter().map(|record| record.source_url.clone()));
    urls.extend(
        metrics
            .iter()
            .flat_map(|record| record.source_urls.iter().cloned()),
    );
    urls.extend(
        scores
            .iter()
            .flat_map(|record| record.source_urls.iter().cloned()),
    );
    urls.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_urls_are_unique_and_sorted() {
        let filing = crate::sec::models::FilingRecord {
            accession: "a".to_string(),
            cik: 1,
            company: "A".to_string(),
            form: "10-K".to_string(),
            filing_date: "2026-01-01".to_string(),
            report_date: None,
            primary_document: None,
            primary_doc_description: None,
            is_xbrl: None,
            is_inline_xbrl: None,
            source_url: "https://b.test".to_string(),
            text_url: "https://text.test".to_string(),
        };

        assert_eq!(
            source_urls(&[filing], &[], &[], &[]),
            vec!["https://b.test".to_string()]
        );
    }
}
