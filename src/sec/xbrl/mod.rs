use std::collections::HashMap;

use anyhow::Result;

use crate::sec::{
    client::SecClient,
    documents::SubmissionDocument,
    edgar::{accession_document_url, filings::matches_form},
    models::{FilingQuery, FilingRecord, XbrlLinkbaseQuery, XbrlLinkbaseRecord},
    parsers::xml::{XmlAttribute, XmlEventWithAttrs, parse_f64, read_xml_with_attrs},
};

impl SecClient {
    pub async fn xbrl_linkbases(
        &self,
        query: XbrlLinkbaseQuery,
    ) -> Result<Vec<XbrlLinkbaseRecord>> {
        let filings = self
            .filings(FilingQuery {
                cik: query.cik,
                form: query.form.clone(),
                latest: query.latest,
                from: None,
                to: None,
                include_amends: query.include_amends,
            })
            .await?
            .into_iter()
            .filter(|filing| {
                query
                    .form
                    .as_deref()
                    .is_none_or(|form| matches_form(&filing.form, Some(form), query.include_amends))
            })
            .collect::<Vec<_>>();

        let mut records = Vec::new();
        for (filing, docs) in self.filing_documents_batch(filings).await? {
            for doc in docs.iter().filter(|doc| classify_linkbase(doc).is_some()) {
                let Some(kind) = classify_linkbase(doc) else {
                    continue;
                };
                if query
                    .linkbase
                    .as_deref()
                    .is_some_and(|filter| !kind.eq_ignore_ascii_case(filter))
                {
                    continue;
                }
                records.extend(parse_linkbase_document(&filing, doc, kind)?);
            }
        }

        records.retain(|record| {
            query.role.as_deref().is_none_or(|needle| {
                record.role.as_deref().is_some_and(|role| {
                    role.to_ascii_lowercase()
                        .contains(&needle.to_ascii_lowercase())
                })
            }) && query.concept.as_deref().is_none_or(|needle| {
                let needle = normalize_concept(needle);
                concept_matches(record.parent_concept.as_deref(), &needle)
                    || concept_matches(record.child_concept.as_deref(), &needle)
                    || concept_matches(record.concept.as_deref(), &needle)
            })
        });
        records.sort_by(|a, b| {
            a.accession
                .cmp(&b.accession)
                .then_with(|| a.linkbase.cmp(&b.linkbase))
                .then_with(|| a.role.cmp(&b.role))
                .then_with(|| {
                    a.order
                        .partial_cmp(&b.order)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        if let Some(limit) = query.limit {
            records.truncate(limit);
        }
        Ok(records)
    }
}

fn parse_linkbase_document(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
    linkbase: &'static str,
) -> Result<Vec<XbrlLinkbaseRecord>> {
    let mut parser = LinkbaseParser::new(filing, doc, linkbase);
    read_xml_with_attrs(doc.xml_content(), |event| parser.handle(event))?;
    Ok(parser.finish())
}

fn classify_linkbase(doc: &SubmissionDocument) -> Option<&'static str> {
    let haystack = format!(
        "{} {} {}",
        doc.document_type.as_deref().unwrap_or_default(),
        doc.filename.as_deref().unwrap_or_default(),
        doc.description.as_deref().unwrap_or_default()
    )
    .to_ascii_lowercase();

    if has_any(&haystack, &["ex-101.pre", "_pre.xml", "-pre.xml"]) {
        Some("presentation")
    } else if has_any(&haystack, &["ex-101.cal", "_cal.xml", "-cal.xml"]) {
        Some("calculation")
    } else if has_any(&haystack, &["ex-101.def", "_def.xml", "-def.xml"]) {
        Some("definition")
    } else if has_any(&haystack, &["ex-101.lab", "_lab.xml", "-lab.xml"]) {
        Some("label")
    } else if has_any(&haystack, &["ex-101.sch", ".xsd"]) {
        Some("schema")
    } else {
        None
    }
}

fn has_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

struct LinkbaseParser<'a> {
    filing: &'a FilingRecord,
    doc: &'a SubmissionDocument,
    linkbase: &'static str,
    role: Option<String>,
    locs: HashMap<String, String>,
    label_resources: HashMap<String, LabelResource>,
    label_arcs: Vec<LabelArc>,
    current_label: Option<LabelResource>,
    records: Vec<XbrlLinkbaseRecord>,
}

#[derive(Clone)]
struct LabelResource {
    id: String,
    role: Option<String>,
    text: String,
}

struct LabelArc {
    role: Option<String>,
    concept: Option<String>,
    to: String,
    order: Option<f64>,
    arcrole: Option<String>,
}

impl<'a> LinkbaseParser<'a> {
    fn new(filing: &'a FilingRecord, doc: &'a SubmissionDocument, linkbase: &'static str) -> Self {
        Self {
            filing,
            doc,
            linkbase,
            role: None,
            locs: HashMap::new(),
            label_resources: HashMap::new(),
            label_arcs: Vec::new(),
            current_label: None,
            records: Vec::new(),
        }
    }

    fn handle(&mut self, event: XmlEventWithAttrs) -> Result<()> {
        match event {
            XmlEventWithAttrs::Start { name, attributes } => {
                self.handle_start(&name, &attributes);
            }
            XmlEventWithAttrs::Text(text) => {
                if let Some(label) = &mut self.current_label {
                    if !label.text.is_empty() {
                        label.text.push(' ');
                    }
                    label.text.push_str(text.trim());
                }
            }
            XmlEventWithAttrs::End(name) => {
                if name.eq_ignore_ascii_case("label") {
                    if let Some(label) = self.current_label.take() {
                        self.label_resources.insert(label.id.clone(), label);
                    }
                } else if name.to_ascii_lowercase().ends_with("link") {
                    self.role = None;
                    self.locs.clear();
                }
            }
        }
        Ok(())
    }

    fn handle_start(&mut self, name: &str, attributes: &[XmlAttribute]) {
        let lower = name.to_ascii_lowercase();
        if lower.ends_with("link") {
            self.role = attr(attributes, "role").map(str::to_string);
            return;
        }

        match lower.as_str() {
            "loc" => self.capture_locator(attributes),
            "presentationarc" | "calculationarc" | "definitionarc" => {
                self.capture_relationship(&lower, attributes)
            }
            "labelarc" => self.capture_label_arc(attributes),
            "label" => self.start_label(attributes),
            "element" if self.linkbase == "schema" => self.capture_schema_element(attributes),
            _ => {}
        }
    }

    fn capture_locator(&mut self, attributes: &[XmlAttribute]) {
        let Some(label) = attr(attributes, "label") else {
            return;
        };
        let Some(href) = attr(attributes, "href") else {
            return;
        };
        self.locs
            .insert(label.to_string(), normalize_concept_href(href));
    }

    fn capture_relationship(&mut self, name: &str, attributes: &[XmlAttribute]) {
        let relationship = name.trim_end_matches("arc").to_string();
        let parent = attr(attributes, "from").and_then(|value| self.locs.get(value).cloned());
        let child = attr(attributes, "to").and_then(|value| self.locs.get(value).cloned());

        self.records.push(self.record(
            relationship,
            attr(attributes, "arcrole").map(str::to_string),
            parent,
            child,
            None,
            None,
            attr(attributes, "order").and_then(parse_f64),
            attr(attributes, "weight").and_then(parse_f64),
            attr(attributes, "preferredLabel").map(str::to_string),
        ));
    }

    fn capture_label_arc(&mut self, attributes: &[XmlAttribute]) {
        let Some(from) = attr(attributes, "from") else {
            return;
        };
        let Some(to) = attr(attributes, "to") else {
            return;
        };
        self.label_arcs.push(LabelArc {
            role: self.role.clone(),
            concept: self.locs.get(from).cloned(),
            to: to.to_string(),
            order: attr(attributes, "order").and_then(parse_f64),
            arcrole: attr(attributes, "arcrole").map(str::to_string),
        });
    }

    fn start_label(&mut self, attributes: &[XmlAttribute]) {
        let Some(id) = attr(attributes, "label") else {
            return;
        };
        self.current_label = Some(LabelResource {
            id: id.to_string(),
            role: attr(attributes, "role").map(str::to_string),
            text: String::new(),
        });
    }

    fn capture_schema_element(&mut self, attributes: &[XmlAttribute]) {
        let Some(name) = attr(attributes, "name") else {
            return;
        };
        self.records.push(self.record(
            "schema_element".to_string(),
            None,
            None,
            None,
            Some(name.to_string()),
            attr(attributes, "type").map(str::to_string),
            None,
            None,
            None,
        ));
    }

    fn finish(mut self) -> Vec<XbrlLinkbaseRecord> {
        for arc in std::mem::take(&mut self.label_arcs) {
            let Some(label) = self.label_resources.get(&arc.to) else {
                continue;
            };
            self.records.push(
                self.record(
                    "label".to_string(),
                    arc.arcrole,
                    None,
                    None,
                    arc.concept,
                    Some(label.text.clone()),
                    arc.order,
                    None,
                    None,
                )
                .with_role(arc.role)
                .with_label_role(label.role.clone()),
            );
        }
        self.records
    }

    #[allow(clippy::too_many_arguments)]
    fn record(
        &self,
        relationship: String,
        arcrole: Option<String>,
        parent_concept: Option<String>,
        child_concept: Option<String>,
        concept: Option<String>,
        label: Option<String>,
        order: Option<f64>,
        weight: Option<f64>,
        preferred_label: Option<String>,
    ) -> XbrlLinkbaseRecord {
        XbrlLinkbaseRecord {
            accession: self.filing.accession.clone(),
            cik: self.filing.cik,
            company: self.filing.company.clone(),
            form: self.filing.form.clone(),
            filing_date: self.filing.filing_date.clone(),
            report_date: self.filing.report_date.clone(),
            linkbase: self.linkbase.to_string(),
            relationship,
            role: self.role.clone(),
            arcrole,
            parent_concept,
            child_concept,
            concept,
            label,
            label_role: None,
            order,
            weight,
            preferred_label,
            document: self.doc.filename.clone(),
            document_sequence: self.doc.sequence.clone(),
            document_description: self.doc.description.clone(),
            document_url: self.doc.filename.as_deref().map(|filename| {
                accession_document_url(self.filing.cik, &self.filing.accession, filename)
            }),
            source_url: self.filing.source_url.clone(),
        }
    }
}

trait XbrlRecordExt {
    fn with_role(self, role: Option<String>) -> Self;
    fn with_label_role(self, label_role: Option<String>) -> Self;
}

impl XbrlRecordExt for XbrlLinkbaseRecord {
    fn with_role(mut self, role: Option<String>) -> Self {
        self.role = role;
        self
    }

    fn with_label_role(mut self, label_role: Option<String>) -> Self {
        self.label_role = label_role;
        self
    }
}

fn attr<'a>(attributes: &'a [XmlAttribute], name: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|attr| {
            attr.name.eq_ignore_ascii_case(name)
                || attr
                    .name
                    .rsplit_once(':')
                    .is_some_and(|(_, local)| local.eq_ignore_ascii_case(name))
        })
        .map(|attr| attr.value.as_str())
}

fn normalize_concept_href(value: &str) -> String {
    value
        .rsplit_once('#')
        .map(|(_, fragment)| fragment)
        .unwrap_or(value)
        .replace('_', ":")
}

fn normalize_concept(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("us-gaap:")
        .trim_start_matches("dei:")
        .trim_start_matches("srt:")
        .replace('_', ":")
        .to_ascii_lowercase()
}

fn concept_matches(value: Option<&str>, needle: &str) -> bool {
    value.is_some_and(|value| normalize_concept(value).contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filing() -> FilingRecord {
        FilingRecord {
            accession: "0000000000-26-000001".to_string(),
            cik: 1,
            company: "Example Inc.".to_string(),
            form: "10-K".to_string(),
            filing_date: "2026-02-01".to_string(),
            report_date: Some("2025-12-31".to_string()),
            primary_document: None,
            primary_doc_description: None,
            is_xbrl: Some(true),
            is_inline_xbrl: Some(true),
            source_url: "https://www.sec.gov/example-index.html".to_string(),
            text_url: "https://www.sec.gov/example.txt".to_string(),
        }
    }

    #[test]
    fn parses_presentation_and_label_linkbases() {
        let filing = filing();
        let pre = SubmissionDocument {
            document_type: Some("EX-101.PRE".to_string()),
            sequence: Some("10".to_string()),
            filename: Some("example-20251231_pre.xml".to_string()),
            description: None,
            content: r##"
                <linkbase xmlns:link="http://www.xbrl.org/2003/linkbase" xmlns:xlink="http://www.w3.org/1999/xlink">
                  <presentationLink xlink:role="http://example/role/IncomeStatement">
                    <loc xlink:label="loc_revenue" xlink:href="example.xsd#us-gaap_Revenues"/>
                    <loc xlink:label="loc_net" xlink:href="example.xsd#us-gaap_NetIncomeLoss"/>
                    <presentationArc xlink:from="loc_revenue" xlink:to="loc_net" order="2" preferredLabel="http://www.xbrl.org/2003/role/terseLabel"/>
                  </presentationLink>
                </linkbase>
            "##
            .to_string(),
        };
        let records = parse_linkbase_document(&filing, &pre, "presentation").unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].parent_concept.as_deref(),
            Some("us-gaap:Revenues")
        );
        assert_eq!(
            records[0].child_concept.as_deref(),
            Some("us-gaap:NetIncomeLoss")
        );

        let lab = SubmissionDocument {
            document_type: Some("EX-101.LAB".to_string()),
            sequence: Some("11".to_string()),
            filename: Some("example-20251231_lab.xml".to_string()),
            description: None,
            content: r##"
                <linkbase xmlns:link="http://www.xbrl.org/2003/linkbase" xmlns:xlink="http://www.w3.org/1999/xlink">
                  <labelLink xlink:role="http://www.xbrl.org/2003/role/link">
                    <loc xlink:label="loc_revenue" xlink:href="example.xsd#us-gaap_Revenues"/>
                    <label xlink:label="lab_revenue" xlink:role="http://www.xbrl.org/2003/role/label">Revenue</label>
                    <labelArc xlink:from="loc_revenue" xlink:to="lab_revenue" order="1"/>
                  </labelLink>
                </linkbase>
            "##
            .to_string(),
        };
        let labels = parse_linkbase_document(&filing, &lab, "label").unwrap();
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].concept.as_deref(), Some("us-gaap:Revenues"));
        assert_eq!(labels[0].label.as_deref(), Some("Revenue"));
    }
}
