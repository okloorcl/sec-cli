use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::Result;

use crate::sec::{
    client::SecClient,
    models::{XbrlLinkbaseQuery, XbrlPresentationTreeRecord, XbrlTreeQuery},
};

use super::normalize_concept;

impl SecClient {
    pub async fn xbrl_presentation_tree(
        &self,
        query: XbrlTreeQuery,
    ) -> Result<Vec<XbrlPresentationTreeRecord>> {
        let base = XbrlLinkbaseQuery {
            cik: query.cik,
            form: query.form.clone(),
            latest: query.latest,
            include_amends: query.include_amends,
            linkbase: None,
            role: query.role.clone(),
            concept: None,
            limit: None,
        };
        let mut presentation_query = base.clone();
        presentation_query.linkbase = Some("presentation".to_string());
        let presentation = self.xbrl_linkbases(presentation_query).await?;

        let mut label_query = base;
        label_query.linkbase = Some("label".to_string());
        let labels = self.xbrl_linkbases(label_query).await?;
        let label_map = labels
            .into_iter()
            .filter_map(|record| Some((record.concept?, record.label?)))
            .collect::<HashMap<_, _>>();

        let mut records = Vec::new();
        for (_, role_records) in group_by_role(presentation) {
            records.extend(render_role(role_records, &label_map));
        }

        if let Some(concept) = query.concept.as_deref() {
            let needle = normalize_concept(concept);
            records.retain(|record| normalize_concept(&record.concept).contains(&needle));
        }
        if let Some(limit) = query.limit {
            records.truncate(limit);
        }
        Ok(records)
    }
}

fn group_by_role(
    records: Vec<crate::sec::models::XbrlLinkbaseRecord>,
) -> BTreeMap<String, Vec<crate::sec::models::XbrlLinkbaseRecord>> {
    let mut grouped = BTreeMap::new();
    for record in records {
        let role = record.role.clone().unwrap_or_else(|| "unknown".to_string());
        grouped.entry(role).or_insert_with(Vec::new).push(record);
    }
    grouped
}

fn render_role(
    records: Vec<crate::sec::models::XbrlLinkbaseRecord>,
    label_map: &HashMap<String, String>,
) -> Vec<XbrlPresentationTreeRecord> {
    let mut children: BTreeMap<String, Vec<Edge>> = BTreeMap::new();
    let mut child_set = BTreeSet::new();
    let mut all_nodes = BTreeSet::new();

    for record in records {
        let (Some(parent), Some(child)) =
            (record.parent_concept.clone(), record.child_concept.clone())
        else {
            continue;
        };
        all_nodes.insert(parent.clone());
        all_nodes.insert(child.clone());
        child_set.insert(child.clone());
        children.entry(parent).or_default().push(Edge {
            child,
            order: record.order,
            preferred_label: record.preferred_label.clone(),
            template: record,
        });
    }
    for edges in children.values_mut() {
        edges.sort_by(|a, b| {
            a.order
                .partial_cmp(&b.order)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.child.cmp(&b.child))
        });
    }

    let roots = all_nodes
        .difference(&child_set)
        .cloned()
        .collect::<Vec<String>>();
    let mut output = Vec::new();
    let mut line_order = 1usize;
    for root in roots {
        walk(
            &root,
            None,
            0,
            &mut Vec::new(),
            &children,
            label_map,
            &mut line_order,
            &mut output,
        );
    }
    output
}

struct Edge {
    child: String,
    order: Option<f64>,
    preferred_label: Option<String>,
    template: crate::sec::models::XbrlLinkbaseRecord,
}

#[allow(clippy::too_many_arguments)]
fn walk(
    concept: &str,
    edge: Option<&Edge>,
    depth: usize,
    path: &mut Vec<String>,
    children: &BTreeMap<String, Vec<Edge>>,
    label_map: &HashMap<String, String>,
    line_order: &mut usize,
    output: &mut Vec<XbrlPresentationTreeRecord>,
) {
    path.push(concept.to_string());

    let template = edge.map(|edge| &edge.template).or_else(|| {
        children
            .get(concept)
            .and_then(|edges| edges.first())
            .map(|edge| &edge.template)
    });
    if let Some(template) = template {
        output.push(XbrlPresentationTreeRecord {
            accession: template.accession.clone(),
            cik: template.cik,
            company: template.company.clone(),
            form: template.form.clone(),
            filing_date: template.filing_date.clone(),
            report_date: template.report_date.clone(),
            role: template
                .role
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            depth,
            line_order: *line_order,
            concept: concept.to_string(),
            label: label_map.get(concept).cloned(),
            parent_concept: edge.and_then(|_| path.get(path.len().saturating_sub(2)).cloned()),
            order: edge.and_then(|edge| edge.order),
            preferred_label: edge.and_then(|edge| edge.preferred_label.clone()),
            path: path.join(" > "),
            document: template.document.clone(),
            document_url: template.document_url.clone(),
            source_url: template.source_url.clone(),
        });
        *line_order += 1;
    }

    if let Some(edges) = children.get(concept) {
        for child_edge in edges {
            walk(
                &child_edge.child,
                Some(child_edge),
                depth + 1,
                path,
                children,
                label_map,
                line_order,
                output,
            );
        }
    }

    path.pop();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sec::models::XbrlLinkbaseRecord;

    fn edge(parent: &str, child: &str, order: f64) -> XbrlLinkbaseRecord {
        XbrlLinkbaseRecord {
            accession: "a".to_string(),
            cik: 1,
            company: "Example".to_string(),
            form: "10-K".to_string(),
            filing_date: "2026-01-01".to_string(),
            report_date: None,
            linkbase: "presentation".to_string(),
            relationship: "presentation".to_string(),
            role: Some("role/income".to_string()),
            arcrole: None,
            parent_concept: Some(parent.to_string()),
            child_concept: Some(child.to_string()),
            concept: None,
            label: None,
            label_role: None,
            order: Some(order),
            weight: None,
            preferred_label: None,
            document: Some("pre.xml".to_string()),
            document_sequence: None,
            document_description: None,
            document_url: None,
            source_url: "source".to_string(),
        }
    }

    #[test]
    fn renders_role_in_preorder() {
        let labels = HashMap::from([
            ("root".to_string(), "Statement".to_string()),
            ("revenue".to_string(), "Revenue".to_string()),
        ]);
        let rows = render_role(
            vec![
                edge("root", "net_income", 2.0),
                edge("root", "revenue", 1.0),
            ],
            &labels,
        );

        assert_eq!(
            rows.iter()
                .map(|row| row.concept.as_str())
                .collect::<Vec<_>>(),
            vec!["root", "revenue", "net_income"]
        );
        assert_eq!(rows[1].depth, 1);
        assert_eq!(rows[1].label.as_deref(), Some("Revenue"));
    }
}
