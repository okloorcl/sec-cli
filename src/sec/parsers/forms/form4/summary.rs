use anyhow::Result;

use crate::sec::{
    documents::{DocumentSet, SubmissionDocument},
    models::{
        FilingRecord, Form4FootnoteRecord, Form4OwnerRecord, Form4ReportRecord,
        Form4SignatureRecord,
    },
};

use crate::sec::parsers::forms::{
    XmlAttribute, XmlEventWithAttrs, parse_f64, path_ends_with, read_xml_with_attrs,
};

use super::{Issuer, Owner, apply_owner_text, parse_bool};

pub fn parse_form4_report_documents(
    filing: &FilingRecord,
    docs: &[SubmissionDocument],
) -> Result<Vec<Form4ReportRecord>> {
    let mut records = Vec::new();
    let document_set = DocumentSet::new(docs);
    for doc in document_set.form4_ownership_xml() {
        records.push(parse_form4_report_xml(filing, doc)?);
    }
    Ok(records)
}

fn parse_form4_report_xml(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
) -> Result<Form4ReportRecord> {
    let mut parser = Form4ReportParser::new(filing, doc);
    read_xml_with_attrs(doc.xml_content(), |event| parser.handle(event))?;
    Ok(parser.finish())
}

#[derive(Default)]
struct SignatureBuilder {
    name: Option<String>,
    date: Option<String>,
}

#[derive(Default)]
struct FootnoteBuilder {
    id: Option<String>,
    text: String,
}

#[derive(Default)]
struct TransactionBuilder {
    derivative: bool,
    shares: Option<f64>,
    price: Option<f64>,
    acquired_disposed: Option<String>,
}

#[derive(Default)]
struct TransactionTotals {
    transaction_count: usize,
    acquisition_count: usize,
    disposition_count: usize,
    derivative_transaction_count: usize,
    total_shares_acquired: f64,
    total_shares_disposed: f64,
    total_value: f64,
}

struct Form4ReportParser<'a> {
    filing: &'a FilingRecord,
    doc: &'a SubmissionDocument,
    path: Vec<String>,
    issuer: Issuer,
    owners: Vec<Owner>,
    signatures: Vec<Form4SignatureRecord>,
    footnotes: Vec<Form4FootnoteRecord>,
    period_of_report: Option<String>,
    not_subject_to_section16: Option<bool>,
    current_owner: Option<Owner>,
    current_signature: Option<SignatureBuilder>,
    current_footnote: Option<FootnoteBuilder>,
    current_tx: Option<TransactionBuilder>,
    totals: TransactionTotals,
}

impl<'a> Form4ReportParser<'a> {
    fn new(filing: &'a FilingRecord, doc: &'a SubmissionDocument) -> Self {
        Self {
            filing,
            doc,
            path: Vec::new(),
            issuer: Issuer::default(),
            owners: Vec::new(),
            signatures: Vec::new(),
            footnotes: Vec::new(),
            period_of_report: None,
            not_subject_to_section16: None,
            current_owner: None,
            current_signature: None,
            current_footnote: None,
            current_tx: None,
            totals: TransactionTotals::default(),
        }
    }

    fn handle(&mut self, event: XmlEventWithAttrs) -> Result<()> {
        match event {
            XmlEventWithAttrs::Start { name, attributes } => {
                self.handle_start(&name, &attributes);
                self.path.push(name);
            }
            XmlEventWithAttrs::End(tag) => {
                self.handle_end(&tag);
                self.path.pop();
            }
            XmlEventWithAttrs::Text(text) => self.apply_text(&text),
        }
        Ok(())
    }

    fn handle_start(&mut self, tag: &str, attributes: &[XmlAttribute]) {
        match tag {
            "reportingOwner" => self.current_owner = Some(Owner::default()),
            "ownerSignature" => self.current_signature = Some(SignatureBuilder::default()),
            "footnote" => {
                self.current_footnote = Some(FootnoteBuilder {
                    id: attribute_value(attributes, "id"),
                    text: String::new(),
                });
            }
            "nonDerivativeTransaction" | "derivativeTransaction" => {
                self.current_tx = Some(TransactionBuilder {
                    derivative: tag == "derivativeTransaction",
                    ..TransactionBuilder::default()
                });
            }
            _ => {}
        }
    }

    fn handle_end(&mut self, tag: &str) {
        match tag {
            "reportingOwner" => {
                if let Some(owner) = self.current_owner.take() {
                    self.owners.push(owner);
                }
            }
            "ownerSignature" => {
                if let Some(signature) = self.current_signature.take() {
                    self.signatures.push(Form4SignatureRecord {
                        signature_name: signature.name,
                        signature_date: signature.date,
                    });
                }
            }
            "footnote" => {
                if let Some(footnote) = self.current_footnote.take() {
                    self.footnotes.push(Form4FootnoteRecord {
                        id: footnote.id,
                        text: footnote.text.trim().to_string(),
                    });
                }
            }
            "nonDerivativeTransaction" | "derivativeTransaction" => {
                if let Some(tx) = self.current_tx.take() {
                    self.tally_transaction(tx);
                }
            }
            _ => {}
        }
    }

    fn apply_text(&mut self, text: &str) {
        if path_ends_with(&self.path, &["periodOfReport"]) {
            self.period_of_report = Some(text.to_string());
        } else if path_ends_with(&self.path, &["notSubjectToSection16"]) {
            self.not_subject_to_section16 = parse_bool(text);
        } else if path_ends_with(&self.path, &["issuer", "issuerName"]) {
            self.issuer.name = Some(text.to_string());
        } else if path_ends_with(&self.path, &["issuer", "issuerCik"]) {
            self.issuer.cik = Some(text.to_string());
        } else if path_ends_with(&self.path, &["issuer", "issuerTradingSymbol"]) {
            self.issuer.ticker = Some(text.to_string());
        }

        if let Some(owner) = self.current_owner.as_mut() {
            apply_owner_text(owner, &self.path, text);
        }
        if let Some(signature) = self.current_signature.as_mut() {
            apply_signature_text(signature, &self.path, text);
        }
        if let Some(footnote) = self.current_footnote.as_mut() {
            append_text(&mut footnote.text, text);
        }
        if let Some(tx) = self.current_tx.as_mut() {
            apply_transaction_text(tx, &self.path, text);
        }
    }

    fn tally_transaction(&mut self, tx: TransactionBuilder) {
        self.totals.transaction_count += 1;
        if tx.derivative {
            self.totals.derivative_transaction_count += 1;
        }

        let shares = tx.shares.unwrap_or(0.0);
        match tx.acquired_disposed.as_deref() {
            Some("A") => {
                self.totals.acquisition_count += 1;
                self.totals.total_shares_acquired += shares;
            }
            Some("D") => {
                self.totals.disposition_count += 1;
                self.totals.total_shares_disposed += shares;
            }
            _ => {}
        }

        if let (Some(shares), Some(price)) = (tx.shares, tx.price) {
            self.totals.total_value += shares * price;
        }
    }

    fn finish(self) -> Form4ReportRecord {
        let total_shares_acquired = round_float(self.totals.total_shares_acquired);
        let total_shares_disposed = round_float(self.totals.total_shares_disposed);

        Form4ReportRecord {
            accession: self.filing.accession.clone(),
            cik: self.filing.cik,
            company: self.filing.company.clone(),
            filing_date: self.filing.filing_date.clone(),
            period_of_report: self.period_of_report,
            not_subject_to_section16: self.not_subject_to_section16,
            issuer: self.issuer.name,
            issuer_cik: self.issuer.cik,
            issuer_ticker: self.issuer.ticker,
            owners: self.owners.into_iter().map(owner_record).collect(),
            signatures: self.signatures,
            footnotes: self.footnotes,
            transaction_count: self.totals.transaction_count,
            acquisition_count: self.totals.acquisition_count,
            disposition_count: self.totals.disposition_count,
            derivative_transaction_count: self.totals.derivative_transaction_count,
            total_shares_acquired,
            total_shares_disposed,
            net_shares: round_float(total_shares_acquired - total_shares_disposed),
            total_value: round_float(self.totals.total_value),
            document: self.doc.filename.clone(),
            document_sequence: self.doc.sequence.clone(),
            document_description: self.doc.description.clone(),
            source_url: self.filing.source_url.clone(),
        }
    }
}

fn apply_signature_text(signature: &mut SignatureBuilder, path: &[String], text: &str) {
    if path_ends_with(path, &["ownerSignature", "signatureName"]) {
        signature.name = Some(text.to_string());
    } else if path_ends_with(path, &["ownerSignature", "signatureDate"]) {
        signature.date = Some(text.to_string());
    }
}

fn apply_transaction_text(tx: &mut TransactionBuilder, path: &[String], text: &str) {
    if path_ends_with(path, &["transactionShares", "value"]) {
        tx.shares = parse_f64(text);
    } else if path_ends_with(path, &["transactionPricePerShare", "value"]) {
        tx.price = parse_f64(text);
    } else if path_ends_with(path, &["transactionAcquiredDisposedCode", "value"]) {
        tx.acquired_disposed = Some(text.to_string());
    }
}

fn attribute_value(attributes: &[XmlAttribute], name: &str) -> Option<String> {
    attributes
        .iter()
        .find(|attr| attr.name == name)
        .map(|attr| attr.value.clone())
}

fn append_text(target: &mut String, text: &str) {
    if !target.is_empty() {
        target.push(' ');
    }
    target.push_str(text);
}

fn owner_record(owner: Owner) -> Form4OwnerRecord {
    Form4OwnerRecord {
        owner_cik: owner.cik,
        owner_name: owner.name,
        is_director: owner.is_director,
        is_officer: owner.is_officer,
        is_ten_percent_owner: owner.is_ten_percent_owner,
        is_other: owner.is_other,
        officer_title: owner.officer_title,
    }
}

fn round_float(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sec::parsers::forms::form4::{sample_doc, sample_filing, sample_form4_xml};

    #[test]
    fn parses_form4_report_summary() {
        let filing = sample_filing();
        let doc = sample_doc(sample_form4_xml());

        let records = parse_form4_report_documents(&filing, &[doc]).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].issuer.as_deref(), Some("ACME Inc."));
        assert_eq!(records[0].period_of_report.as_deref(), Some("2026-01-01"));
        assert_eq!(records[0].owners.len(), 1);
        assert_eq!(records[0].signatures.len(), 1);
        assert_eq!(records[0].footnotes[0].id.as_deref(), Some("F1"));
        assert_eq!(records[0].transaction_count, 1);
        assert_eq!(records[0].disposition_count, 1);
        assert_eq!(records[0].total_shares_disposed, 10.0);
        assert_eq!(records[0].net_shares, -10.0);
        assert_eq!(records[0].total_value, 125.0);
    }
}
