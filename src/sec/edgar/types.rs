use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub(crate) struct SubmissionsResponse {
    #[serde(default)]
    pub(crate) name: String,
    pub(crate) filings: SubmissionFilings,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SubmissionFilings {
    pub(crate) recent: RecentFilings,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RecentFilings {
    #[serde(rename = "accessionNumber")]
    pub(crate) accession_number: Vec<String>,
    pub(crate) form: Vec<String>,
    #[serde(rename = "filingDate")]
    pub(crate) filing_date: Vec<String>,
    #[serde(default, rename = "reportDate")]
    pub(crate) report_date: Vec<String>,
    #[serde(default, rename = "primaryDocument")]
    pub(crate) primary_document: Vec<String>,
    #[serde(default, rename = "primaryDocDescription")]
    pub(crate) primary_doc_description: Vec<String>,
    #[serde(default, rename = "isXBRL")]
    pub(crate) is_xbrl: Vec<Option<bool>>,
    #[serde(default, rename = "isInlineXBRL")]
    pub(crate) is_inline_xbrl: Vec<Option<bool>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CompanyFactsResponse {
    #[serde(default, rename = "entityName")]
    pub(crate) entity_name: Option<String>,
    pub(crate) facts: BTreeMap<String, BTreeMap<String, CompanyFactConcept>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CompanyFactConcept {
    #[serde(default)]
    pub(crate) label: Option<String>,
    #[serde(default)]
    pub(crate) description: Option<String>,
    #[serde(default)]
    pub(crate) units: BTreeMap<String, Vec<CompanyFactValue>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CompanyFactValue {
    #[serde(default)]
    pub(crate) accn: Option<String>,
    #[serde(default)]
    pub(crate) val: Value,
    #[serde(default)]
    pub(crate) fy: Option<i64>,
    #[serde(default)]
    pub(crate) fp: Option<String>,
    #[serde(default)]
    pub(crate) form: Option<String>,
    #[serde(default)]
    pub(crate) filed: Option<String>,
    #[serde(default)]
    pub(crate) start: Option<String>,
    #[serde(default)]
    pub(crate) end: Option<String>,
    #[serde(default)]
    pub(crate) frame: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_submissions_recent_filings() {
        let response: SubmissionsResponse = serde_json::from_value(serde_json::json!({
            "name": "Apple Inc.",
            "filings": {
                "recent": {
                    "accessionNumber": ["0000320193-26-000001"],
                    "form": ["10-K"],
                    "filingDate": ["2026-10-30"],
                    "reportDate": ["2026-09-26"],
                    "primaryDocument": ["aapl.htm"],
                    "primaryDocDescription": ["10-K"],
                    "isXBRL": [true],
                    "isInlineXBRL": [true]
                }
            }
        }))
        .unwrap();

        assert_eq!(response.name, "Apple Inc.");
        assert_eq!(response.filings.recent.form, vec!["10-K"]);
        assert_eq!(response.filings.recent.is_xbrl, vec![Some(true)]);
    }

    #[test]
    fn deserializes_dynamic_company_facts() {
        let response: CompanyFactsResponse = serde_json::from_value(serde_json::json!({
            "entityName": "Apple Inc.",
            "facts": {
                "us-gaap": {
                    "Revenues": {
                        "label": "Revenue",
                        "units": {
                            "USD": [{
                                "accn": "0000320193-26-000001",
                                "val": 100,
                                "fy": 2026,
                                "fp": "FY",
                                "form": "10-K",
                                "filed": "2026-10-30"
                            }]
                        }
                    }
                }
            }
        }))
        .unwrap();

        let concept = &response.facts["us-gaap"]["Revenues"];
        assert_eq!(concept.label.as_deref(), Some("Revenue"));
        assert_eq!(concept.units["USD"][0].val, serde_json::json!(100));
    }
}
