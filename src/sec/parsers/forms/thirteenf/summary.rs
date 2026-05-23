use anyhow::Result;

use crate::sec::{
    documents::{DocumentSet, SubmissionDocument},
    models::{FilingRecord, ThirteenFOtherManagerRecord, ThirteenFReportRecord},
};

use crate::sec::parsers::forms::{XmlEvent, parse_u64, path_ends_with, read_xml};

use super::{normalize_mmddyyyy, scale_value_to_usd, value_scale};

pub fn parse_13f_report_documents(
    filing: &FilingRecord,
    docs: &[SubmissionDocument],
) -> Result<Vec<ThirteenFReportRecord>> {
    let mut records = Vec::new();
    let document_set = DocumentSet::new(docs);
    for doc in document_set.thirteenf_primary_documents() {
        records.push(parse_13f_report_xml(filing, doc)?);
    }
    Ok(records)
}

fn parse_13f_report_xml(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
) -> Result<ThirteenFReportRecord> {
    let mut parser = ThirteenFReportParser::new(filing, doc);
    read_xml(doc.xml_content(), |event| parser.handle(event))?;
    Ok(parser.finish())
}

#[derive(Default)]
struct ReportSummary {
    report_period: Option<String>,
    report_calendar_or_quarter: Option<String>,
    report_type: Option<String>,
    other_included_managers_count: Option<u64>,
    total_holdings_reported: Option<u64>,
    total_value_reported: Option<u64>,
    filing_manager_name: Option<String>,
    filing_manager_city: Option<String>,
    filing_manager_state_or_country: Option<String>,
    filing_manager_zipcode: Option<String>,
    signature_name: Option<String>,
    signature_title: Option<String>,
    signature_phone: Option<String>,
    signature_city: Option<String>,
    signature_state_or_country: Option<String>,
    signature_date: Option<String>,
    additional_information: Option<String>,
    other_managers: Vec<ThirteenFOtherManagerRecord>,
}

struct OtherManagerBuilder {
    sequence_number: Option<u64>,
    cik: Option<String>,
    name: Option<String>,
    form13f_file_number: Option<String>,
}

struct ThirteenFReportParser<'a> {
    filing: &'a FilingRecord,
    doc: &'a SubmissionDocument,
    path: Vec<String>,
    summary: ReportSummary,
    current_other_manager: Option<OtherManagerBuilder>,
}

impl<'a> ThirteenFReportParser<'a> {
    fn new(filing: &'a FilingRecord, doc: &'a SubmissionDocument) -> Self {
        Self {
            filing,
            doc,
            path: Vec::new(),
            summary: ReportSummary::default(),
            current_other_manager: None,
        }
    }

    fn handle(&mut self, event: XmlEvent) -> Result<()> {
        match event {
            XmlEvent::Start(tag) => {
                if tag == "otherManager2" {
                    self.current_other_manager = Some(OtherManagerBuilder {
                        sequence_number: None,
                        cik: None,
                        name: None,
                        form13f_file_number: None,
                    });
                }
                self.path.push(tag);
            }
            XmlEvent::End(tag) => {
                if tag == "otherManager2" {
                    if let Some(manager) = self.current_other_manager.take() {
                        self.summary
                            .other_managers
                            .push(ThirteenFOtherManagerRecord {
                                sequence_number: manager.sequence_number,
                                cik: manager.cik,
                                name: manager.name,
                                form13f_file_number: manager.form13f_file_number,
                            });
                    }
                }
                self.path.pop();
            }
            XmlEvent::Text(text) => self.apply_text(&text),
        }
        Ok(())
    }

    fn apply_text(&mut self, text: &str) {
        if path_ends_with(&self.path, &["filerInfo", "periodOfReport"]) {
            self.summary.report_period = Some(normalize_mmddyyyy(text));
        } else if path_ends_with(&self.path, &["coverPage", "reportCalendarOrQuarter"])
            || path_ends_with(&self.path, &["formData", "reportCalendarOrQuarter"])
        {
            self.summary.report_calendar_or_quarter = Some(normalize_mmddyyyy(text));
        } else if path_ends_with(&self.path, &["coverPage", "reportType"]) {
            self.summary.report_type = Some(text.to_string());
        } else if path_ends_with(&self.path, &["filingManager", "name"]) {
            self.summary.filing_manager_name = Some(text.to_string());
        } else if path_ends_with(&self.path, &["filingManager", "address", "city"]) {
            self.summary.filing_manager_city = Some(text.to_string());
        } else if path_ends_with(&self.path, &["filingManager", "address", "stateOrCountry"]) {
            self.summary.filing_manager_state_or_country = Some(text.to_string());
        } else if path_ends_with(&self.path, &["filingManager", "address", "zipCode"]) {
            self.summary.filing_manager_zipcode = Some(text.to_string());
        } else if path_ends_with(&self.path, &["summaryPage", "otherIncludedManagersCount"]) {
            self.summary.other_included_managers_count = parse_u64(text);
        } else if path_ends_with(&self.path, &["summaryPage", "tableEntryTotal"]) {
            self.summary.total_holdings_reported = parse_u64(text);
        } else if path_ends_with(&self.path, &["summaryPage", "tableValueTotal"]) {
            self.summary.total_value_reported = parse_u64(text);
        } else if path_ends_with(&self.path, &["signatureBlock", "name"]) {
            self.summary.signature_name = Some(text.to_string());
        } else if path_ends_with(&self.path, &["signatureBlock", "title"]) {
            self.summary.signature_title = Some(text.to_string());
        } else if path_ends_with(&self.path, &["signatureBlock", "phone"]) {
            self.summary.signature_phone = Some(text.to_string());
        } else if path_ends_with(&self.path, &["signatureBlock", "city"]) {
            self.summary.signature_city = Some(text.to_string());
        } else if path_ends_with(&self.path, &["signatureBlock", "stateOrCountry"]) {
            self.summary.signature_state_or_country = Some(text.to_string());
        } else if path_ends_with(&self.path, &["signatureBlock", "signatureDate"]) {
            self.summary.signature_date = Some(normalize_mmddyyyy(text));
        } else if path_ends_with(&self.path, &["coverPage", "additionalInformation"]) {
            self.summary.additional_information = Some(text.to_string());
        }

        if let Some(manager) = self.current_other_manager.as_mut() {
            if path_ends_with(&self.path, &["otherManager2", "sequenceNumber"]) {
                manager.sequence_number = parse_u64(text);
            } else if path_ends_with(&self.path, &["otherManager", "cik"]) {
                manager.cik = Some(text.to_string());
            } else if path_ends_with(&self.path, &["otherManager", "name"]) {
                manager.name = Some(text.to_string());
            } else if path_ends_with(&self.path, &["otherManager", "form13FFileNumber"]) {
                manager.form13f_file_number = Some(text.to_string());
            }
        }
    }

    fn finish(self) -> ThirteenFReportRecord {
        let report_date = self
            .summary
            .report_period
            .clone()
            .or_else(|| self.filing.report_date.clone());
        let value_scale = value_scale(report_date.as_deref());
        let total_value_usd = self
            .summary
            .total_value_reported
            .map(|value| scale_value_to_usd(value, value_scale));

        ThirteenFReportRecord {
            accession: self.filing.accession.clone(),
            cik: self.filing.cik,
            manager: self.filing.company.clone(),
            filing_date: self.filing.filing_date.clone(),
            report_date,
            report_calendar_or_quarter: self.summary.report_calendar_or_quarter,
            report_type: self.summary.report_type,
            other_included_managers_count: self.summary.other_included_managers_count,
            total_holdings_reported: self.summary.total_holdings_reported,
            total_value_reported: self.summary.total_value_reported,
            value_scale: value_scale.to_string(),
            total_value_usd,
            filing_manager_name: self.summary.filing_manager_name,
            filing_manager_city: self.summary.filing_manager_city,
            filing_manager_state_or_country: self.summary.filing_manager_state_or_country,
            filing_manager_zipcode: self.summary.filing_manager_zipcode,
            signature_name: self.summary.signature_name,
            signature_title: self.summary.signature_title,
            signature_phone: self.summary.signature_phone,
            signature_city: self.summary.signature_city,
            signature_state_or_country: self.summary.signature_state_or_country,
            signature_date: self.summary.signature_date,
            additional_information: self.summary.additional_information,
            other_managers: self.summary.other_managers,
            document: self.doc.filename.clone(),
            document_sequence: self.doc.sequence.clone(),
            document_description: self.doc.description.clone(),
            source_url: self.filing.source_url.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sec::parsers::forms::thirteenf::{sample_doc, sample_filing};

    #[test]
    fn parses_13f_primary_report_summary() {
        let filing = sample_filing();
        let doc = sample_doc(
            "13F-HR",
            "1",
            "primary.xml",
            "PRIMARY DOCUMENT",
            sample_primary_xml(),
        );

        let records = parse_13f_report_documents(&filing, &[doc]).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].report_type.as_deref(),
            Some("13F HOLDINGS REPORT")
        );
        assert_eq!(records[0].total_holdings_reported, Some(1));
        assert_eq!(records[0].total_value_usd, Some(25000));
        assert_eq!(records[0].other_managers.len(), 1);
    }

    fn sample_primary_xml() -> &'static str {
        r#"
        <edgarSubmission>
          <headerData><filerInfo><periodOfReport>12-31-2025</periodOfReport></filerInfo></headerData>
          <formData>
            <coverPage>
              <reportCalendarOrQuarter>12-31-2025</reportCalendarOrQuarter>
              <reportType>13F HOLDINGS REPORT</reportType>
              <filingManager>
                <name>ACME CAPITAL</name>
                <address><city>New York</city><stateOrCountry>NY</stateOrCountry><zipCode>10001</zipCode></address>
              </filingManager>
              <additionalInformation>none</additionalInformation>
            </coverPage>
            <summaryPage>
              <otherIncludedManagersCount>1</otherIncludedManagersCount>
              <tableEntryTotal>1</tableEntryTotal>
              <tableValueTotal>25000</tableValueTotal>
              <otherManagers2Info>
                <otherManager2>
                  <sequenceNumber>1</sequenceNumber>
                  <otherManager><cik>0001</cik><name>Sub Manager</name><form13FFileNumber>028-1</form13FFileNumber></otherManager>
                </otherManager2>
              </otherManagers2Info>
            </summaryPage>
            <signatureBlock>
              <name>Jane Signer</name><title>CEO</title><phone>555</phone><signature>Jane Signer</signature>
              <city>New York</city><stateOrCountry>NY</stateOrCountry><signatureDate>02-14-2026</signatureDate>
            </signatureBlock>
          </formData>
        </edgarSubmission>
        "#
    }
}
