use std::collections::{BTreeMap, HashMap};

use anyhow::Result;

use crate::sec::{
    client::SecClient,
    edgar::{
        types::CompanyFactsResponse,
        urls::{accession_index_url, company_facts_url},
    },
    models::{XbrlCalculationCheckRecord, XbrlCalculationQuery, XbrlLinkbaseQuery},
};

use super::normalize_concept;

impl SecClient {
    pub async fn xbrl_calculation_checks(
        &self,
        query: XbrlCalculationQuery,
    ) -> Result<Vec<XbrlCalculationCheckRecord>> {
        let mut link_query = XbrlLinkbaseQuery {
            cik: query.cik,
            form: query.form.clone(),
            latest: query.latest,
            include_amends: query.include_amends,
            linkbase: Some("calculation".to_string()),
            role: query.role.clone(),
            concept: None,
            limit: None,
        };
        let links = self.xbrl_linkbases(link_query.clone()).await?;
        let facts: CompanyFactsResponse = self.get_json(&company_facts_url(query.cik)).await?;
        let values = fact_values(&facts, query.unit.as_deref().unwrap_or("USD"));

        let mut checks = Vec::new();
        for ((accession, role, parent), children) in group_calculations(links) {
            if query.concept.as_deref().is_some_and(|needle| {
                !normalize_concept(&parent).contains(&normalize_concept(needle))
            }) {
                continue;
            }
            let Some(first_child) = children.first() else {
                continue;
            };
            let template = &first_child.template;
            let fact_key = FactKey {
                accession: accession.clone(),
                concept: parent.clone(),
            };
            let parent_value = values.get(&fact_key).copied();
            let mut calculated = 0.0;
            let mut matched = 0usize;
            let mut missing = Vec::new();

            for child in &children {
                let child_key = FactKey {
                    accession: accession.clone(),
                    concept: child.child.child.clone(),
                };
                if let Some(value) = values.get(&child_key) {
                    calculated += value * child.child.weight.unwrap_or(1.0);
                    matched += 1;
                } else {
                    missing.push(child.child.child.clone());
                }
            }

            let calculated_value = (matched > 0).then_some(calculated);
            let difference = parent_value
                .zip(calculated_value)
                .map(|(parent, calc)| parent - calc);
            let relative_difference = difference.zip(parent_value).and_then(|(diff, parent)| {
                if parent.abs() > f64::EPSILON {
                    Some(diff / parent)
                } else {
                    None
                }
            });
            let status = check_status(parent_value, calculated_value, difference, query.tolerance);

            checks.push(XbrlCalculationCheckRecord {
                accession: accession.clone(),
                cik: template.cik,
                company: template.company.clone(),
                form: template.form.clone(),
                filing_date: template.filing_date.clone(),
                report_date: template.report_date.clone(),
                role,
                parent_concept: parent,
                parent_value,
                calculated_value,
                difference,
                relative_difference,
                status,
                children_count: children.len(),
                matched_children: matched,
                missing_children: missing,
                unit: query.unit.clone().unwrap_or_else(|| "USD".to_string()),
                document: template.document.clone(),
                document_url: template.document_url.clone(),
                source_url: accession_index_url(template.cik, &accession),
            });
        }

        checks.sort_by(|a, b| {
            a.accession
                .cmp(&b.accession)
                .then_with(|| a.role.cmp(&b.role))
                .then_with(|| a.parent_concept.cmp(&b.parent_concept))
        });
        if let Some(limit) = query.limit {
            checks.truncate(limit);
        }
        link_query.limit = query.limit;
        Ok(checks)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct FactKey {
    accession: String,
    concept: String,
}

fn fact_values(response: &CompanyFactsResponse, unit_filter: &str) -> HashMap<FactKey, f64> {
    let mut values = HashMap::new();
    for (taxonomy, concepts) in &response.facts {
        for (concept, data) in concepts {
            let concept = format!("{taxonomy}:{concept}");
            for (unit, facts) in &data.units {
                if !unit.eq_ignore_ascii_case(unit_filter) {
                    continue;
                }
                for fact in facts {
                    let (Some(accession), Some(value)) = (fact.accn.clone(), fact.val.as_f64())
                    else {
                        continue;
                    };
                    values.insert(
                        FactKey {
                            accession,
                            concept: concept.clone(),
                        },
                        value,
                    );
                }
            }
        }
    }
    values
}

#[derive(Clone)]
struct CalcChild {
    child: String,
    weight: Option<f64>,
}

type CalcGroupKey = (String, String, String);

fn group_calculations(
    links: Vec<crate::sec::models::XbrlLinkbaseRecord>,
) -> BTreeMap<CalcGroupKey, Vec<GroupedCalcChild>> {
    let mut grouped = BTreeMap::new();
    for link in links {
        let (Some(parent), Some(child)) = (link.parent_concept.clone(), link.child_concept.clone())
        else {
            continue;
        };
        let key = (
            link.accession.clone(),
            link.role.clone().unwrap_or_else(|| "unknown".to_string()),
            parent,
        );
        grouped
            .entry(key)
            .or_insert_with(Vec::new)
            .push(GroupedCalcChild {
                child: CalcChild {
                    child,
                    weight: link.weight,
                },
                template: link,
            });
    }
    grouped
}

struct GroupedCalcChild {
    child: CalcChild,
    template: crate::sec::models::XbrlLinkbaseRecord,
}

fn check_status(
    parent: Option<f64>,
    calculated: Option<f64>,
    difference: Option<f64>,
    tolerance: f64,
) -> String {
    match (parent, calculated, difference) {
        (None, _, _) => "missing_parent".to_string(),
        (_, None, _) => "missing_children".to_string(),
        (Some(_), Some(_), Some(diff)) if diff.abs() <= tolerance => "ok".to_string(),
        (Some(parent), Some(_), Some(diff))
            if parent.abs() > f64::EPSILON && (diff / parent).abs() <= tolerance =>
        {
            "ok".to_string()
        }
        _ => "mismatch".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_calculation_status() {
        assert_eq!(
            check_status(Some(100.0), Some(100.01), Some(-0.01), 0.1),
            "ok"
        );
        assert_eq!(check_status(None, Some(1.0), None, 0.1), "missing_parent");
        assert_eq!(check_status(Some(1.0), None, None, 0.1), "missing_children");
        assert_eq!(
            check_status(Some(100.0), Some(80.0), Some(20.0), 0.01),
            "mismatch"
        );
    }
}
