use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::sec::{
    client::SecClient,
    edgar::filings::matches_form,
    models::{FactQuery, FactRecord},
};

use super::urls::{accession_index_url, company_facts_url};

impl SecClient {
    pub async fn facts(&self, query: FactQuery) -> Result<Vec<FactRecord>> {
        let json: Value = self.get_json(&company_facts_url(query.cik)).await?;
        let facts_root = json
            .get("facts")
            .and_then(Value::as_object)
            .ok_or_else(|| anyhow!("companyfacts JSON missing facts"))?;

        let mut records = Vec::new();
        let concept_query_raw = query.concept.to_ascii_lowercase();
        let concept_query = concept_query_raw
            .rsplit(':')
            .next()
            .unwrap_or(&concept_query_raw)
            .to_string();

        for (taxonomy, concepts) in facts_root {
            let Some(concepts) = concepts.as_object() else {
                continue;
            };
            for (concept, concept_data) in concepts {
                let label = concept_data
                    .get("label")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let description = concept_data
                    .get("description")
                    .and_then(Value::as_str)
                    .map(str::to_string);

                if !concept_matches(
                    &concept_query,
                    concept,
                    label.as_deref(),
                    description.as_deref(),
                ) {
                    continue;
                }

                let Some(units) = concept_data.get("units").and_then(Value::as_object) else {
                    continue;
                };
                collect_unit_facts(
                    &query,
                    taxonomy,
                    concept,
                    label,
                    description,
                    units,
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
    units: &serde_json::Map<String, Value>,
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

        let Some(values) = values.as_array() else {
            continue;
        };
        for item in values.iter().rev() {
            if query.form.as_deref().is_some_and(|form_filter| {
                let item_form = item.get("form").and_then(Value::as_str).unwrap_or("");
                !matches_form(item_form, Some(form_filter), true)
            }) {
                continue;
            }

            let accession = item.get("accn").and_then(Value::as_str).map(str::to_string);
            records.push(FactRecord {
                concept: format!("{taxonomy}:{concept}"),
                taxonomy: taxonomy.to_string(),
                label: label.clone(),
                description: description.clone(),
                value: item.get("val").cloned().unwrap_or(Value::Null),
                unit: unit.to_string(),
                fy: item.get("fy").and_then(Value::as_i64),
                fp: item.get("fp").and_then(Value::as_str).map(str::to_string),
                form: item.get("form").and_then(Value::as_str).map(str::to_string),
                filed: item
                    .get("filed")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                start: item
                    .get("start")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                end: item.get("end").and_then(Value::as_str).map(str::to_string),
                frame: item
                    .get("frame")
                    .and_then(Value::as_str)
                    .map(str::to_string),
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
