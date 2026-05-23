use anyhow::Result;

use crate::sec::{
    documents::{DocumentSet, SubmissionDocument},
    models::{FilingRecord, Form4TransactionRecord},
};

use crate::sec::parsers::forms::{XmlEvent, parse_f64, path_ends_with, read_xml};

use super::{Issuer, Owner, apply_owner_text, transaction_type};

pub fn parse_form4_transaction_documents(
    filing: &FilingRecord,
    docs: &[SubmissionDocument],
) -> Result<Vec<Form4TransactionRecord>> {
    let mut records = Vec::new();
    let document_set = DocumentSet::new(docs);
    for doc in document_set.form4_ownership_xml() {
        records.extend(parse_form4_xml(filing, doc)?);
    }
    Ok(records)
}

fn parse_form4_xml(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
) -> Result<Vec<Form4TransactionRecord>> {
    let mut parser = Form4TransactionParser::new(filing, doc);
    read_xml(doc.xml_content(), |event| parser.handle(event))?;
    Ok(parser.finish())
}

#[derive(Default)]
struct Transaction {
    date: Option<String>,
    deemed_execution_date: Option<String>,
    form_type: Option<String>,
    code: Option<String>,
    equity_swap_involved: Option<bool>,
    acquired_disposed: Option<String>,
    security_title: Option<String>,
    shares: Option<f64>,
    price: Option<f64>,
    shares_owned_after: Option<f64>,
    direct_or_indirect: Option<String>,
    nature_of_ownership: Option<String>,
    derivative: bool,
    conversion_or_exercise_price: Option<f64>,
    exercise_date: Option<String>,
    expiration_date: Option<String>,
    underlying_security_title: Option<String>,
    underlying_shares: Option<f64>,
}

struct Form4TransactionParser<'a> {
    filing: &'a FilingRecord,
    doc: &'a SubmissionDocument,
    path: Vec<String>,
    issuer: Issuer,
    owners: Vec<Owner>,
    current_owner: Option<Owner>,
    current_tx: Option<Transaction>,
    records: Vec<Form4TransactionRecord>,
}

impl<'a> Form4TransactionParser<'a> {
    fn new(filing: &'a FilingRecord, doc: &'a SubmissionDocument) -> Self {
        Self {
            filing,
            doc,
            path: Vec::new(),
            issuer: Issuer::default(),
            owners: Vec::new(),
            current_owner: None,
            current_tx: None,
            records: Vec::new(),
        }
    }

    fn handle(&mut self, event: XmlEvent) -> Result<()> {
        match event {
            XmlEvent::Start(tag) => {
                if tag == "reportingOwner" {
                    self.current_owner = Some(Owner::default());
                } else if tag == "nonDerivativeTransaction" || tag == "derivativeTransaction" {
                    self.current_tx = Some(Transaction {
                        derivative: tag == "derivativeTransaction",
                        ..Transaction::default()
                    });
                }
                self.path.push(tag);
            }
            XmlEvent::End(tag) => {
                if tag == "reportingOwner" {
                    if let Some(owner) = self.current_owner.take() {
                        self.owners.push(owner);
                    }
                } else if (tag == "nonDerivativeTransaction" || tag == "derivativeTransaction")
                    && self.current_tx.is_some()
                {
                    let tx = self.current_tx.take().expect("checked above");
                    self.records.push(self.record_from(tx));
                }
                self.path.pop();
            }
            XmlEvent::Text(text) => self.apply_text(&text),
        }
        Ok(())
    }

    fn apply_text(&mut self, text: &str) {
        if path_ends_with(&self.path, &["issuer", "issuerName"]) {
            self.issuer.name = Some(text.to_string());
        } else if path_ends_with(&self.path, &["issuer", "issuerCik"]) {
            self.issuer.cik = Some(text.to_string());
        } else if path_ends_with(&self.path, &["issuer", "issuerTradingSymbol"]) {
            self.issuer.ticker = Some(text.to_string());
        } else if let Some(owner) = self.current_owner.as_mut() {
            apply_owner_text(owner, &self.path, text);
        }

        if let Some(tx) = self.current_tx.as_mut() {
            apply_transaction_text(tx, &self.path, text);
        }
    }

    fn finish(self) -> Vec<Form4TransactionRecord> {
        self.records
    }

    fn record_from(&self, tx: Transaction) -> Form4TransactionRecord {
        let owner = self.owners.first().cloned().unwrap_or_default();
        let value = match (tx.shares, tx.price) {
            (Some(shares), Some(price)) => Some(shares * price),
            _ => None,
        };

        Form4TransactionRecord {
            accession: self.filing.accession.clone(),
            cik: self.filing.cik,
            company: self.filing.company.clone(),
            filing_date: self.filing.filing_date.clone(),
            issuer: self.issuer.name.clone(),
            issuer_cik: self.issuer.cik.clone(),
            issuer_ticker: self.issuer.ticker.clone(),
            reporting_owner: owner.name,
            owner_cik: owner.cik,
            is_director: owner.is_director,
            is_officer: owner.is_officer,
            is_ten_percent_owner: owner.is_ten_percent_owner,
            is_other: owner.is_other,
            officer_title: owner.officer_title,
            transaction_date: tx.date,
            deemed_execution_date: tx.deemed_execution_date,
            transaction_form_type: tx.form_type,
            transaction_code: tx.code.clone(),
            equity_swap_involved: tx.equity_swap_involved,
            acquired_disposed: tx.acquired_disposed.clone(),
            transaction_type: transaction_type(tx.code.as_deref(), tx.acquired_disposed.as_deref()),
            security_title: tx.security_title,
            shares: tx.shares,
            price: tx.price,
            value,
            shares_owned_after: tx.shares_owned_after,
            direct_or_indirect: tx.direct_or_indirect,
            nature_of_ownership: tx.nature_of_ownership,
            derivative: tx.derivative,
            conversion_or_exercise_price: tx.conversion_or_exercise_price,
            exercise_date: tx.exercise_date,
            expiration_date: tx.expiration_date,
            underlying_security_title: tx.underlying_security_title,
            underlying_shares: tx.underlying_shares,
            document: self.doc.filename.clone(),
            document_sequence: self.doc.sequence.clone(),
            document_description: self.doc.description.clone(),
            source_url: self.filing.source_url.clone(),
        }
    }
}

fn apply_transaction_text(tx: &mut Transaction, path: &[String], text: &str) {
    if path_ends_with(path, &["securityTitle", "value"]) {
        tx.security_title = Some(text.to_string());
    } else if path_ends_with(path, &["transactionDate", "value"]) {
        tx.date = Some(text.to_string());
    } else if path_ends_with(path, &["deemedExecutionDate", "value"]) {
        tx.deemed_execution_date = Some(text.to_string());
    } else if path_ends_with(path, &["transactionCoding", "transactionFormType"]) {
        tx.form_type = Some(text.to_string());
    } else if path_ends_with(path, &["transactionCoding", "transactionCode"]) {
        tx.code = Some(text.to_string());
    } else if path_ends_with(path, &["transactionCoding", "equitySwapInvolved"]) {
        tx.equity_swap_involved = super::parse_bool(text);
    } else if path_ends_with(path, &["transactionShares", "value"]) {
        tx.shares = parse_f64(text);
    } else if path_ends_with(path, &["transactionPricePerShare", "value"]) {
        tx.price = parse_f64(text);
    } else if path_ends_with(path, &["conversionOrExercisePrice", "value"]) {
        tx.conversion_or_exercise_price = parse_f64(text);
    } else if path_ends_with(path, &["transactionAcquiredDisposedCode", "value"]) {
        tx.acquired_disposed = Some(text.to_string());
    } else if path_ends_with(path, &["exerciseDate", "value"]) {
        tx.exercise_date = Some(text.to_string());
    } else if path_ends_with(path, &["expirationDate", "value"]) {
        tx.expiration_date = Some(text.to_string());
    } else if path_ends_with(path, &["underlyingSecurityTitle", "value"]) {
        tx.underlying_security_title = Some(text.to_string());
    } else if path_ends_with(path, &["underlyingSecurityShares", "value"]) {
        tx.underlying_shares = parse_f64(text);
    } else if path_ends_with(path, &["sharesOwnedFollowingTransaction", "value"]) {
        tx.shares_owned_after = parse_f64(text);
    } else if path_ends_with(path, &["directOrIndirectOwnership", "value"]) {
        tx.direct_or_indirect = Some(text.to_string());
    } else if path_ends_with(path, &["natureOfOwnership", "value"]) {
        tx.nature_of_ownership = Some(text.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sec::parsers::forms::form4::{sample_doc, sample_filing, sample_form4_xml};

    #[test]
    fn parses_non_derivative_form4_transaction() {
        let filing = sample_filing();
        let doc = sample_doc(sample_form4_xml());

        let records = parse_form4_transaction_documents(&filing, &[doc]).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].issuer.as_deref(), Some("ACME Inc."));
        assert_eq!(records[0].issuer_ticker.as_deref(), Some("ACME"));
        assert_eq!(records[0].reporting_owner.as_deref(), Some("Jane Owner"));
        assert_eq!(records[0].transaction_type.as_deref(), Some("sale"));
        assert_eq!(records[0].transaction_form_type.as_deref(), Some("4"));
        assert_eq!(records[0].equity_swap_involved, Some(false));
        assert_eq!(records[0].shares, Some(10.0));
        assert_eq!(records[0].value, Some(125.0));
        assert_eq!(records[0].nature_of_ownership.as_deref(), Some("By Trust"));
    }
}
