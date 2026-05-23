use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;

use crate::sec::{
    client::SecClient,
    models::{
        FinancialStatementRecord, StatementQuery, StatementStitchQuery, StitchedStatementRecord,
    },
};

impl SecClient {
    pub async fn stitched_statements(
        &self,
        query: StatementStitchQuery,
    ) -> Result<Vec<StitchedStatementRecord>> {
        let rows = self
            .financial_statements(StatementQuery {
                cik: query.cik,
                statement: query.statement,
                form: None,
                unit: query.unit,
                latest: query.latest.saturating_mul(3).max(12),
            })
            .await?;
        Ok(stitch_rows(query.latest, rows))
    }
}

fn stitch_rows(latest: usize, rows: Vec<FinancialStatementRecord>) -> Vec<StitchedStatementRecord> {
    let mut grouped: BTreeMap<StitchKey, Vec<FinancialStatementRecord>> = BTreeMap::new();
    for row in rows {
        grouped.entry(StitchKey::from(&row)).or_default().push(row);
    }

    let mut stitched = grouped
        .into_values()
        .filter_map(stitch_group)
        .collect::<Vec<_>>();
    stitched.sort_by(|a, b| {
        statement_rank(&a.statement)
            .cmp(&statement_rank(&b.statement))
            .then_with(|| a.line_order.cmp(&b.line_order))
            .then_with(|| b.end.cmp(&a.end))
            .then_with(|| b.filed.cmp(&a.filed))
    });
    truncate_per_line(&mut stitched, latest);
    stitched
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StitchKey {
    statement: String,
    line_item: String,
    period_end: String,
}

impl StitchKey {
    fn from(row: &FinancialStatementRecord) -> Self {
        Self {
            statement: row.statement.clone(),
            line_item: row.line_item.clone(),
            period_end: row
                .end
                .clone()
                .or_else(|| row.filed.clone())
                .unwrap_or_else(|| "unknown".to_string()),
        }
    }
}

fn stitch_group(mut rows: Vec<FinancialStatementRecord>) -> Option<StitchedStatementRecord> {
    rows.sort_by(|a, b| {
        form_rank(a.form.as_deref(), a.fiscal_period.as_deref())
            .cmp(&form_rank(b.form.as_deref(), b.fiscal_period.as_deref()))
            .then_with(|| b.filed.cmp(&a.filed))
            .then_with(|| b.accession.cmp(&a.accession))
    });
    let selected = rows.first()?;
    let duplicate_forms = rows
        .iter()
        .filter_map(|row| row.form.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    Some(StitchedStatementRecord {
        cik: selected.cik,
        company: selected.company.clone(),
        statement: selected.statement.clone(),
        line_order: selected.line_order,
        line_item: selected.line_item.clone(),
        concept: selected.concept.clone(),
        value: selected.value.clone(),
        numeric_value: selected.numeric_value,
        unit: selected.unit.clone(),
        fiscal_year: selected.fiscal_year,
        fiscal_period: selected.fiscal_period.clone(),
        period_kind: period_kind(selected.fiscal_period.as_deref()),
        form: selected.form.clone(),
        filed: selected.filed.clone(),
        start: selected.start.clone(),
        end: selected.end.clone(),
        frame: selected.frame.clone(),
        accession: selected.accession.clone(),
        duplicate_forms,
        source_count: rows.len(),
        source_url: selected.source_url.clone(),
        fact_id: selected.fact_id.clone(),
    })
}

fn form_rank(form: Option<&str>, fiscal_period: Option<&str>) -> u8 {
    match (form.unwrap_or_default(), fiscal_period.unwrap_or_default()) {
        ("10-K", "FY") => 0,
        ("10-K/A", "FY") => 1,
        ("10-Q", _) => 2,
        ("10-Q/A", _) => 3,
        ("10-K", _) => 4,
        _ => 9,
    }
}

fn period_kind(fiscal_period: Option<&str>) -> String {
    match fiscal_period.unwrap_or_default() {
        "FY" => "annual".to_string(),
        "Q1" | "Q2" | "Q3" | "Q4" => "quarterly".to_string(),
        value if value.starts_with('Q') => "quarterly".to_string(),
        _ => "other".to_string(),
    }
}

fn truncate_per_line(rows: &mut Vec<StitchedStatementRecord>, latest: usize) {
    let mut counts = BTreeMap::<(String, String), usize>::new();
    rows.retain(|row| {
        let key = (row.statement.clone(), row.line_item.clone());
        let count = counts.entry(key).or_default();
        if *count >= latest {
            false
        } else {
            *count += 1;
            true
        }
    });
}

fn statement_rank(statement: &str) -> u8 {
    match statement {
        "income" => 0,
        "balance" => 1,
        "cashflow" => 2,
        _ => 9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn row(form: &str, fp: &str, filed: &str, accession: &str) -> FinancialStatementRecord {
        FinancialStatementRecord {
            cik: 1,
            company: Some("Example".to_string()),
            statement: "income".to_string(),
            line_order: 1,
            line_item: "revenue".to_string(),
            concept: "us-gaap:Revenues".to_string(),
            taxonomy: "us-gaap".to_string(),
            label: Some("Revenue".to_string()),
            value: json!(100),
            numeric_value: Some(100.0),
            unit: "USD".to_string(),
            fiscal_year: Some(2026),
            fiscal_period: Some(fp.to_string()),
            form: Some(form.to_string()),
            filed: Some(filed.to_string()),
            start: None,
            end: Some("2026-12-31".to_string()),
            frame: None,
            accession: Some(accession.to_string()),
            source_url: Some("source".to_string()),
            fact_id: Some("fact".to_string()),
        }
    }

    #[test]
    fn prefers_annual_10k_for_fy_periods() {
        let rows = stitch_rows(
            4,
            vec![
                row("10-Q", "Q4", "2027-01-10", "q4"),
                row("10-K", "FY", "2027-02-10", "k"),
            ],
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].form.as_deref(), Some("10-K"));
        assert_eq!(rows[0].duplicate_forms, vec!["10-K", "10-Q"]);
    }
}
