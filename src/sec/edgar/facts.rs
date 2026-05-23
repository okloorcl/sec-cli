use anyhow::Result;
use std::collections::BTreeMap;

use crate::sec::{
    client::SecClient,
    edgar::filings::matches_form,
    models::{FactQuery, FactRecord},
};

use super::{
    types::{CompanyFactValue, CompanyFactsResponse},
    urls::{accession_index_url, company_facts_url},
};

impl SecClient {
    pub async fn facts(&self, query: FactQuery) -> Result<Vec<FactRecord>> {
        let response: CompanyFactsResponse = self.get_json(&company_facts_url(query.cik)).await?;

        let mut records = Vec::new();
        let concept_query_raw = query.concept.to_ascii_lowercase();
        let concept_query = concept_query_raw
            .rsplit(':')
            .next()
            .unwrap_or(&concept_query_raw)
            .to_string();

        for (taxonomy, concepts) in &response.facts {
            for (concept, concept_data) in concepts {
                let label = concept_data.label.clone();
                let description = concept_data.description.clone();

                if !concept_matches(
                    &concept_query,
                    concept,
                    label.as_deref(),
                    description.as_deref(),
                ) {
                    continue;
                }

                collect_unit_facts(
                    &query,
                    taxonomy,
                    concept,
                    label,
                    description,
                    &concept_data.units,
                    &mut records,
                );
            }
        }

        records.sort_by(|a, b| {
            b.filed
                .cmp(&a.filed)
                .then_with(|| b.end.cmp(&a.end))
                .then_with(|| b.accession.cmp(&a.accession))
        });
        records.truncate(query.latest);
        Ok(records)
    }
}

fn collect_unit_facts(
    query: &FactQuery,
    taxonomy: &str,
    concept: &str,
    label: Option<String>,
    description: Option<String>,
    units: &BTreeMap<String, Vec<CompanyFactValue>>,
    records: &mut Vec<FactRecord>,
) {
    for (unit, values) in units {
        if query
            .unit
            .as_deref()
            .is_some_and(|filter_unit| !unit.eq_ignore_ascii_case(filter_unit))
        {
            continue;
        }

        for item in values.iter().rev() {
            if query.form.as_deref().is_some_and(|form_filter| {
                let item_form = item.form.as_deref().unwrap_or("");
                !matches_form(item_form, Some(form_filter), true)
            }) {
                continue;
            }

            let accession = item.accn.clone();
            records.push(FactRecord {
                concept: format!("{taxonomy}:{concept}"),
                taxonomy: taxonomy.to_string(),
                label: label.clone(),
                description: description.clone(),
                value: item.val.clone(),
                unit: unit.to_string(),
                fy: item.fy,
                fp: item.fp.clone(),
                form: item.form.clone(),
                filed: item.filed.clone(),
                start: item.start.clone(),
                end: item.end.clone(),
                frame: item.frame.clone(),
                source_url: accession
                    .as_deref()
                    .map(|acc| accession_index_url(query.cik, acc)),
                fact_id: accession
                    .as_deref()
                    .map(|acc| format!("{taxonomy}:{concept}:{acc}:{unit}")),
                accession,
            });
        }
    }
}

fn concept_matches(
    query: &str,
    concept: &str,
    label: Option<&str>,
    _description: Option<&str>,
) -> bool {
    let concept_lc = concept.to_ascii_lowercase();
    if is_strict_alias(query) {
        return known_concept_alias(query, &concept_lc);
    }
    if concept_lc.contains(query) || known_concept_alias(query, &concept_lc) {
        return true;
    }
    label.is_some_and(|value| value.to_ascii_lowercase().contains(query))
}

fn is_strict_alias(query: &str) -> bool {
    matches!(
        query,
        "revenue" | "revenues" | "net income" | "netincome" | "assets" | "total assets" | "cash"
    )
}

fn known_concept_alias(query: &str, concept: &str) -> bool {
    match query {
        "revenue" | "revenues" => matches!(
            concept,
            "revenues"
                | "revenuefromcontractwithcustomerexcludingassessedtax"
                | "salesrevenuenet"
                | "salesrevenuegoodsnet"
        ),
        "net income" | "netincome" => concept == "netincomeloss",
        "assets" | "total assets" => concept == "assets",
        "cash" => concept == "cashandcashequivalentsatcarryingvalue",
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_known_strict_aliases() {
        assert!(concept_matches(
            "revenue",
            "RevenueFromContractWithCustomerExcludingAssessedTax",
            None,
            None
        ));
        assert!(concept_matches("netincome", "NetIncomeLoss", None, None));
        assert!(!concept_matches(
            "cash",
            "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
            None,
            None
        ));
    }

    #[test]
    fn matches_concept_or_label_for_non_strict_queries() {
        assert!(concept_matches("inventory", "InventoryNet", None, None));
        assert!(concept_matches(
            "research",
            "ResearchAndDevelopmentExpense",
            Some("Research and development"),
            None
        ));
    }
}
