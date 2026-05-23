use anyhow::Result;

use crate::sec::{
    client::SecClient,
    documents::{DocumentSet, SubmissionDocument},
    models::{FilingQuery, FilingRecord, ThirteenFHoldingRecord, ThirteenFQuery},
};

use super::{XmlEvent, parse_f64, parse_u64, path_ends_with, read_xml};

impl SecClient {
    pub async fn thirteenf_holdings(
        &self,
        query: ThirteenFQuery,
    ) -> Result<Vec<ThirteenFHoldingRecord>> {
        let filings = self
            .filings(FilingQuery {
                cik: query.cik,
                form: Some("13F-HR".to_string()),
                latest: query.latest,
                from: None,
                to: None,
                include_amends: query.include_amends,
            })
            .await?;

        let mut records = Vec::new();
        for filing in filings {
            let docs = self.filing_documents(&filing).await?;
            records.extend(parse_13f_documents(&filing, &docs)?);
        }
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_13f_information_table() {
        let filing = FilingRecord {
            accession: "0000000000-00-000013".to_string(),
            cik: 13,
            company: "ACME CAPITAL".to_string(),
            form: "13F-HR".to_string(),
            filing_date: "2026-02-14".to_string(),
            report_date: Some("2025-12-31".to_string()),
            primary_document: None,
            primary_doc_description: None,
            is_xbrl: None,
            is_inline_xbrl: None,
            source_url: "https://example.test/index.html".to_string(),
            text_url: "https://example.test/submission.txt".to_string(),
        };
        let doc = SubmissionDocument {
            document_type: Some("INFORMATION TABLE".to_string()),
            sequence: Some("2".to_string()),
            filename: Some("infotable.xml".to_string()),
            description: Some("INFORMATION TABLE FOR FORM 13F".to_string()),
            content: sample_13f_xml().to_string(),
        };

        let records = parse_13f_documents(&filing, &[doc]).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].manager, "ACME CAPITAL");
        assert_eq!(records[0].issuer.as_deref(), Some("APPLE INC"));
        assert_eq!(records[0].cusip.as_deref(), Some("037833100"));
        assert_eq!(records[0].shares, Some(100.0));
        assert_eq!(records[0].voting_sole, Some(100));
    }

    fn sample_13f_xml() -> &'static str {
        r#"
        <informationTable>
          <infoTable>
            <nameOfIssuer>APPLE INC</nameOfIssuer>
            <titleOfClass>COM</titleOfClass>
            <cusip>037833100</cusip>
            <value>25000</value>
            <shrsOrPrnAmt><sshPrnamt>100</sshPrnamt><sshPrnamtType>SH</sshPrnamtType></shrsOrPrnAmt>
            <investmentDiscretion>SOLE</investmentDiscretion>
            <votingAuthority><Sole>100</Sole><Shared>0</Shared><None>0</None></votingAuthority>
          </infoTable>
        </informationTable>
        "#
    }
}

pub fn parse_13f_documents(
    filing: &FilingRecord,
    docs: &[SubmissionDocument],
) -> Result<Vec<ThirteenFHoldingRecord>> {
    let mut records = Vec::new();
    let document_set = DocumentSet::new(docs);
    for doc in document_set.thirteenf_information_tables() {
        records.extend(parse_13f_xml(filing, doc)?);
    }
    Ok(records)
}

fn parse_13f_xml(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
) -> Result<Vec<ThirteenFHoldingRecord>> {
    let mut parser = ThirteenFParser::new(filing, doc);
    read_xml(doc.xml_content(), |event| parser.handle(event))?;
    Ok(parser.finish())
}

#[derive(Default)]
struct Holding {
    issuer: Option<String>,
    class: Option<String>,
    cusip: Option<String>,
    value_reported: Option<u64>,
    shares: Option<f64>,
    share_type: Option<String>,
    put_call: Option<String>,
    investment_discretion: Option<String>,
    other_manager: Option<String>,
    voting_sole: Option<u64>,
    voting_shared: Option<u64>,
    voting_none: Option<u64>,
}

struct ThirteenFParser<'a> {
    filing: &'a FilingRecord,
    doc: &'a SubmissionDocument,
    path: Vec<String>,
    current: Option<Holding>,
    records: Vec<ThirteenFHoldingRecord>,
}

impl<'a> ThirteenFParser<'a> {
    fn new(filing: &'a FilingRecord, doc: &'a SubmissionDocument) -> Self {
        Self {
            filing,
            doc,
            path: Vec::new(),
            current: None,
            records: Vec::new(),
        }
    }

    fn handle(&mut self, event: XmlEvent) -> Result<()> {
        match event {
            XmlEvent::Start(tag) => {
                if tag == "infoTable" {
                    self.current = Some(Holding::default());
                }
                self.path.push(tag);
            }
            XmlEvent::End(tag) => {
                if tag == "infoTable" && self.current.is_some() {
                    let holding = self.current.take().expect("checked above");
                    self.records.push(self.record_from(holding));
                }
                self.path.pop();
            }
            XmlEvent::Text(text) => self.apply_text(&text),
        }
        Ok(())
    }

    fn apply_text(&mut self, text: &str) {
        let Some(holding) = self.current.as_mut() else {
            return;
        };

        if path_ends_with(&self.path, &["nameOfIssuer"]) {
            holding.issuer = Some(text.to_string());
        } else if path_ends_with(&self.path, &["titleOfClass"]) {
            holding.class = Some(text.to_string());
        } else if path_ends_with(&self.path, &["cusip"]) {
            holding.cusip = Some(text.to_string());
        } else if path_ends_with(&self.path, &["value"]) {
            holding.value_reported = parse_u64(text);
        } else if path_ends_with(&self.path, &["sshPrnamt"]) {
            holding.shares = parse_f64(text);
        } else if path_ends_with(&self.path, &["sshPrnamtType"]) {
            holding.share_type = Some(text.to_string());
        } else if path_ends_with(&self.path, &["putCall"]) {
            holding.put_call = Some(text.to_string());
        } else if path_ends_with(&self.path, &["investmentDiscretion"]) {
            holding.investment_discretion = Some(text.to_string());
        } else if path_ends_with(&self.path, &["otherManager"]) {
            holding.other_manager = Some(text.to_string());
        } else if path_ends_with(&self.path, &["votingAuthority", "Sole"]) {
            holding.voting_sole = parse_u64(text);
        } else if path_ends_with(&self.path, &["votingAuthority", "Shared"]) {
            holding.voting_shared = parse_u64(text);
        } else if path_ends_with(&self.path, &["votingAuthority", "None"]) {
            holding.voting_none = parse_u64(text);
        }
    }

    fn finish(self) -> Vec<ThirteenFHoldingRecord> {
        self.records
    }

    fn record_from(&self, holding: Holding) -> ThirteenFHoldingRecord {
        ThirteenFHoldingRecord {
            accession: self.filing.accession.clone(),
            cik: self.filing.cik,
            manager: self.filing.company.clone(),
            filing_date: self.filing.filing_date.clone(),
            report_date: self.filing.report_date.clone(),
            issuer: holding.issuer,
            class: holding.class,
            cusip: holding.cusip,
            value_reported: holding.value_reported,
            shares: holding.shares,
            share_type: holding.share_type,
            put_call: holding.put_call,
            investment_discretion: holding.investment_discretion,
            other_manager: holding.other_manager,
            voting_sole: holding.voting_sole,
            voting_shared: holding.voting_shared,
            voting_none: holding.voting_none,
            document: self.doc.filename.clone(),
            document_sequence: self.doc.sequence.clone(),
            document_description: self.doc.description.clone(),
            source_url: self.filing.source_url.clone(),
        }
    }
}
