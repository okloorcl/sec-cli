use anyhow::Result;

use crate::sec::{
    client::SecClient,
    documents::{DocumentSet, SubmissionDocument},
    edgar::accession_document_url,
    models::{FilingQuery, FilingRecord, InlineXbrlFactRecord, InlineXbrlQuery},
    parsers::xml::{XmlAttribute, XmlEventWithAttrs, read_xml_with_attrs},
};

impl SecClient {
    pub async fn inline_xbrl_facts(
        &self,
        query: InlineXbrlQuery,
    ) -> Result<Vec<InlineXbrlFactRecord>> {
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
        for filing in filings {
            let docs = self.filing_documents(&filing).await?;
            let Some(doc) = DocumentSet::new(&docs).primary_documents().next() else {
                continue;
            };
            records.extend(parse_inline_xbrl_facts(
                &filing,
                doc,
                query.concept.as_deref(),
            )?);
            if let Some(limit) = query.limit {
                if records.len() >= limit {
                    records.truncate(limit);
                    break;
                }
            }
        }
        Ok(records)
    }
}

pub fn parse_inline_xbrl_facts(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
    concept_filter: Option<&str>,
) -> Result<Vec<InlineXbrlFactRecord>> {
    let mut records = Vec::new();
    let mut current: Option<OpenFact> = None;

    read_xml_with_attrs(&doc.content, |event| {
        match event {
            XmlEventWithAttrs::Start { name, attributes } if is_ix_fact(&name) => {
                current = Some(OpenFact {
                    fact_type: name,
                    attributes,
                    text: String::new(),
                });
            }
            XmlEventWithAttrs::Text(text) => {
                if let Some(fact) = current.as_mut() {
                    fact.text.push_str(&text);
                }
            }
            XmlEventWithAttrs::End(name) if is_ix_fact(&name) => {
                if let Some(fact) = current.take() {
                    if let Some(record) = fact_record(filing, doc, fact, concept_filter) {
                        records.push(record);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    })?;

    Ok(records)
}

struct OpenFact {
    fact_type: String,
    attributes: Vec<XmlAttribute>,
    text: String,
}

fn fact_record(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
    fact: OpenFact,
    concept_filter: Option<&str>,
) -> Option<InlineXbrlFactRecord> {
    let name = attr(&fact.attributes, "name")?;
    let (namespace, local_name) = split_concept(&name);
    if !matches_concept(&name, &local_name, concept_filter) {
        return None;
    }
    let raw_value = clean_text(&fact.text);
    let scale = attr(&fact.attributes, "scale").and_then(|value| value.parse::<i32>().ok());
    let sign = attr(&fact.attributes, "sign");
    let numeric_value = (fact.fact_type == "nonFraction")
        .then(|| parse_numeric(&raw_value, scale, sign.as_deref()))
        .flatten();

    Some(InlineXbrlFactRecord {
        accession: filing.accession.clone(),
        cik: filing.cik,
        company: filing.company.clone(),
        form: filing.form.clone(),
        filing_date: filing.filing_date.clone(),
        report_date: filing.report_date.clone(),
        fact_type: fact.fact_type,
        name,
        namespace,
        local_name,
        context_ref: attr(&fact.attributes, "contextRef"),
        unit_ref: attr(&fact.attributes, "unitRef"),
        decimals: attr(&fact.attributes, "decimals"),
        scale,
        format: attr(&fact.attributes, "format"),
        sign,
        id: attr(&fact.attributes, "id"),
        value: raw_value.clone(),
        raw_value,
        numeric_value,
        document: doc.filename.clone(),
        document_sequence: doc.sequence.clone(),
        document_description: doc.description.clone(),
        document_url: doc
            .filename
            .as_deref()
            .map(|filename| accession_document_url(filing.cik, &filing.accession, filename)),
        source_url: filing.source_url.clone(),
    })
}

fn is_ix_fact(name: &str) -> bool {
    matches!(name, "nonFraction" | "nonNumeric")
}

fn attr(attributes: &[XmlAttribute], name: &str) -> Option<String> {
    attributes
        .iter()
        .find(|attr| attr.name == name)
        .map(|attr| attr.value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn split_concept(name: &str) -> (Option<String>, String) {
    if let Some((namespace, local_name)) = name.split_once(':') {
        (Some(namespace.to_string()), local_name.to_string())
    } else {
        (None, name.to_string())
    }
}

fn matches_concept(name: &str, local_name: &str, filter: Option<&str>) -> bool {
    let Some(filter) = filter else {
        return true;
    };
    if filter.contains(':') {
        name.eq_ignore_ascii_case(filter)
    } else {
        local_name.eq_ignore_ascii_case(filter)
    }
}

fn parse_numeric(value: &str, scale: Option<i32>, sign: Option<&str>) -> Option<f64> {
    let cleaned = value.replace([',', '$'], "").trim().to_string();
    if cleaned.is_empty() {
        return None;
    }
    let mut number = cleaned.parse::<f64>().ok()?;
    if let Some(scale) = scale {
        number *= 10_f64.powi(scale);
    }
    if sign == Some("-") {
        number = -number;
    }
    Some(number)
}

fn clean_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inline_xbrl_facts() {
        let filing = FilingRecord {
            accession: "0000000000-26-000001".to_string(),
            cik: 1,
            company: "Example Inc.".to_string(),
            form: "10-K".to_string(),
            filing_date: "2026-01-01".to_string(),
            report_date: Some("2025-12-31".to_string()),
            primary_document: Some("example.htm".to_string()),
            primary_doc_description: Some("10-K".to_string()),
            is_xbrl: Some(true),
            is_inline_xbrl: Some(true),
            source_url: "https://example.test/index.html".to_string(),
            text_url: "https://example.test/submission.txt".to_string(),
        };
        let doc = SubmissionDocument {
            document_type: Some("10-K".to_string()),
            sequence: Some("1".to_string()),
            filename: Some("example.htm".to_string()),
            description: Some("10-K".to_string()),
            content: r#"<html><body><ix:nonFraction name="us-gaap:Revenues" contextRef="c1" unitRef="USD" scale="6" decimals="-6" id="f1">123</ix:nonFraction><ix:nonNumeric name="dei:DocumentFiscalYearFocus" contextRef="c1">2025</ix:nonNumeric></body></html>"#.to_string(),
        };

        let records = parse_inline_xbrl_facts(&filing, &doc, None).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].local_name, "Revenues");
        assert_eq!(records[0].numeric_value, Some(123_000_000.0));
        assert_eq!(records[1].name, "dei:DocumentFiscalYearFocus");
    }
}
