use std::collections::HashMap;

use anyhow::Result;
use serde_json::Value;

use crate::sec::{
    client::SecClient,
    edgar::{types::CompanyFactsResponse, urls::company_facts_url},
    models::{
        XbrlCalculationQuery, XbrlPresentationTreeRecord, XbrlStatementQuery, XbrlStatementRecord,
        XbrlTreeQuery,
    },
};

use super::normalize_concept;

impl SecClient {
    pub async fn xbrl_statement(
        &self,
        query: XbrlStatementQuery,
    ) -> Result<Vec<XbrlStatementRecord>> {
        let tree = self
            .xbrl_presentation_tree(XbrlTreeQuery {
                cik: query.cik,
                form: query.form.clone(),
                latest: query.latest,
                include_amends: query.include_amends,
                role: query.role.clone(),
                concept: None,
                limit: None,
            })
            .await?;
        let facts: CompanyFactsResponse = self.get_json(&company_facts_url(query.cik)).await?;
        let unit = query.unit.as_deref().unwrap_or("USD");
        let fact_map = fact_values(&facts, unit);
        let label_map = fact_labels(&facts);
        let checks = self
            .xbrl_calculation_checks(XbrlCalculationQuery {
                cik: query.cik,
                form: query.form.clone(),
                latest: query.latest,
                include_amends: query.include_amends,
                role: query.role.clone(),
                concept: None,
                unit: Some(unit.to_string()),
                tolerance: query.tolerance,
                limit: None,
            })
            .await?;
        let check_map = checks
            .into_iter()
            .map(|check| {
                (
                    FactKey {
                        accession: check.accession.clone(),
                        concept: check.parent_concept.clone(),
                    },
                    CheckValue {
                        status: check.status,
                        difference: check.difference,
                        relative_difference: check.relative_difference,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        let mut rows = tree
            .into_iter()
            .filter(|row| {
                query.concept.as_deref().is_none_or(|needle| {
                    normalize_concept(&row.concept).contains(&normalize_concept(needle))
                })
            })
            .filter_map(|row| {
                let fact = fact_map.get(&FactKey {
                    accession: row.accession.clone(),
                    concept: row.concept.clone(),
                });
                if query.values_only && fact.is_none() {
                    return None;
                }
                Some(render_row(row, fact, &label_map, &check_map))
            })
            .collect::<Vec<_>>();

        if let Some(limit) = query.limit {
            rows.truncate(limit);
        }
        Ok(rows)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FactKey {
    accession: String,
    concept: String,
}

#[derive(Debug, Clone)]
struct FactValue {
    value: Value,
    numeric_value: Option<f64>,
    unit: String,
    fact_id: String,
}

#[derive(Debug, Clone)]
struct CheckValue {
    status: String,
    difference: Option<f64>,
    relative_difference: Option<f64>,
}

fn render_row(
    row: XbrlPresentationTreeRecord,
    fact: Option<&FactValue>,
    labels: &HashMap<String, String>,
    checks: &HashMap<FactKey, CheckValue>,
) -> XbrlStatementRecord {
    let check = checks.get(&FactKey {
        accession: row.accession.clone(),
        concept: row.concept.clone(),
    });
    XbrlStatementRecord {
        accession: row.accession,
        cik: row.cik,
        company: row.company,
        form: row.form,
        filing_date: row.filing_date,
        report_date: row.report_date,
        role: row.role,
        depth: row.depth,
        line_order: row.line_order,
        label: row
            .label
            .or_else(|| labels.get(&row.concept).cloned())
            .or_else(|| Some(row.concept.clone())),
        parent_concept: row.parent_concept,
        value: fact.map(|fact| fact.value.clone()),
        numeric_value: fact.and_then(|fact| fact.numeric_value),
        unit: fact.map(|fact| fact.unit.clone()),
        fact_id: fact.map(|fact| fact.fact_id.clone()),
        calculation_status: check.map(|check| check.status.clone()),
        calculation_difference: check.and_then(|check| check.difference),
        calculation_relative_difference: check.and_then(|check| check.relative_difference),
        path: row.path,
        document: row.document,
        document_url: row.document_url,
        source_url: row.source_url,
        concept: row.concept,
    }
}

fn fact_values(response: &CompanyFactsResponse, unit_filter: &str) -> HashMap<FactKey, FactValue> {
    let mut values = HashMap::new();
    for (taxonomy, concepts) in &response.facts {
        for (concept, data) in concepts {
            let concept = format!("{taxonomy}:{concept}");
            for (unit, facts) in &data.units {
                if !unit.eq_ignore_ascii_case(unit_filter) {
                    continue;
                }
                for fact in facts {
                    let Some(accession) = fact.accn.clone() else {
                        continue;
                    };
                    values.insert(
                        FactKey {
                            accession: accession.clone(),
                            concept: concept.clone(),
                        },
                        FactValue {
                            value: fact.val.clone(),
                            numeric_value: fact.val.as_f64(),
                            unit: unit.clone(),
                            fact_id: format!("{concept}:{accession}:{unit}"),
                        },
                    );
                }
            }
        }
    }
    values
}

fn fact_labels(response: &CompanyFactsResponse) -> HashMap<String, String> {
    let mut labels = HashMap::new();
    for (taxonomy, concepts) in &response.facts {
        for (concept, data) in concepts {
            if let Some(label) = data.label.clone() {
                labels.insert(format!("{taxonomy}:{concept}"), label);
            }
        }
    }
    labels
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(concept: &str) -> XbrlPresentationTreeRecord {
        XbrlPresentationTreeRecord {
            accession: "0000000000-26-000001".to_string(),
            cik: 1,
            company: "Example Inc.".to_string(),
            form: "10-K".to_string(),
            filing_date: "2026-02-01".to_string(),
            report_date: Some("2025-12-31".to_string()),
            role: "OPERATIONS".to_string(),
            depth: 1,
            line_order: 2,
            concept: concept.to_string(),
            label: None,
            parent_concept: Some("us-gaap:Statement".to_string()),
            order: Some(1.0),
            preferred_label: None,
            path: format!("us-gaap:Statement > {concept}"),
            document: Some("pre.xml".to_string()),
            document_url: Some("https://www.sec.gov/pre.xml".to_string()),
            source_url: "https://www.sec.gov/index.html".to_string(),
        }
    }

    #[test]
    fn render_row_adds_fact_label_and_check() {
        let fact = FactValue {
            value: serde_json::json!(100),
            numeric_value: Some(100.0),
            unit: "USD".to_string(),
            fact_id: "fact-id".to_string(),
        };
        let check = CheckValue {
            status: "ok".to_string(),
            difference: Some(0.0),
            relative_difference: Some(0.0),
        };
        let rows = render_row(
            row("us-gaap:Revenues"),
            Some(&fact),
            &HashMap::from([("us-gaap:Revenues".to_string(), "Revenue".to_string())]),
            &HashMap::from([(
                FactKey {
                    accession: "0000000000-26-000001".to_string(),
                    concept: "us-gaap:Revenues".to_string(),
                },
                check,
            )]),
        );

        assert_eq!(rows.label.as_deref(), Some("Revenue"));
        assert_eq!(rows.numeric_value, Some(100.0));
        assert_eq!(rows.calculation_status.as_deref(), Some("ok"));
    }
}
