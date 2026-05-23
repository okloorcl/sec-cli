use anyhow::Result;

use crate::sec::{
    client::SecClient,
    documents::{DocumentSet, SubmissionDocument},
    models::{FilingQuery, FilingRecord, Form4Query, Form4TransactionRecord},
};

use super::{XmlEvent, parse_f64, path_ends_with, read_xml};

impl SecClient {
    pub async fn form4_transactions(
        &self,
        query: Form4Query,
    ) -> Result<Vec<Form4TransactionRecord>> {
        let filings = self
            .filings(FilingQuery {
                cik: query.cik,
                form: Some("4".to_string()),
                latest: query.latest,
                from: None,
                to: None,
                include_amends: query.include_amends,
            })
            .await?;

        let mut records = Vec::new();
        for filing in filings {
            let docs = self.filing_documents(&filing).await?;
            records.extend(parse_form4_documents(&filing, &docs)?);
        }
        Ok(records)
    }
}

pub fn parse_form4_documents(
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
    let mut parser = Form4Parser::new(filing, doc);
    read_xml(doc.xml_content(), |event| parser.handle(event))?;
    Ok(parser.finish())
}

#[derive(Default, Clone)]
struct Issuer {
    name: Option<String>,
    cik: Option<String>,
}

#[derive(Default, Clone)]
struct Owner {
    name: Option<String>,
    cik: Option<String>,
    is_director: Option<bool>,
    is_officer: Option<bool>,
    is_ten_percent_owner: Option<bool>,
    officer_title: Option<String>,
}

#[derive(Default)]
struct Transaction {
    date: Option<String>,
    code: Option<String>,
    acquired_disposed: Option<String>,
    security_title: Option<String>,
    shares: Option<f64>,
    price: Option<f64>,
    shares_owned_after: Option<f64>,
    direct_or_indirect: Option<String>,
    derivative: bool,
}

struct Form4Parser<'a> {
    filing: &'a FilingRecord,
    doc: &'a SubmissionDocument,
    path: Vec<String>,
    issuer: Issuer,
    owners: Vec<Owner>,
    current_owner: Option<Owner>,
    current_tx: Option<Transaction>,
    records: Vec<Form4TransactionRecord>,
}

impl<'a> Form4Parser<'a> {
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
            reporting_owner: owner.name,
            owner_cik: owner.cik,
            is_director: owner.is_director,
            is_officer: owner.is_officer,
            is_ten_percent_owner: owner.is_ten_percent_owner,
            officer_title: owner.officer_title,
            transaction_date: tx.date,
            transaction_code: tx.code.clone(),
            acquired_disposed: tx.acquired_disposed.clone(),
            transaction_type: transaction_type(tx.code.as_deref(), tx.acquired_disposed.as_deref()),
            security_title: tx.security_title,
            shares: tx.shares,
            price: tx.price,
            value,
            shares_owned_after: tx.shares_owned_after,
            direct_or_indirect: tx.direct_or_indirect,
            derivative: tx.derivative,
            document: self.doc.filename.clone(),
            document_sequence: self.doc.sequence.clone(),
            document_description: self.doc.description.clone(),
            source_url: self.filing.source_url.clone(),
        }
    }
}

fn apply_owner_text(owner: &mut Owner, path: &[String], text: &str) {
    if path_ends_with(path, &["reportingOwnerId", "rptOwnerName"]) {
        owner.name = Some(text.to_string());
    } else if path_ends_with(path, &["reportingOwnerId", "rptOwnerCik"]) {
        owner.cik = Some(text.to_string());
    } else if path_ends_with(path, &["reportingOwnerRelationship", "isDirector"]) {
        owner.is_director = parse_bool(text);
    } else if path_ends_with(path, &["reportingOwnerRelationship", "isOfficer"]) {
        owner.is_officer = parse_bool(text);
    } else if path_ends_with(path, &["reportingOwnerRelationship", "isTenPercentOwner"]) {
        owner.is_ten_percent_owner = parse_bool(text);
    } else if path_ends_with(path, &["reportingOwnerRelationship", "officerTitle"]) {
        owner.officer_title = Some(text.to_string());
    }
}

fn apply_transaction_text(tx: &mut Transaction, path: &[String], text: &str) {
    if path_ends_with(path, &["securityTitle", "value"]) {
        tx.security_title = Some(text.to_string());
    } else if path_ends_with(path, &["transactionDate", "value"]) {
        tx.date = Some(text.to_string());
    } else if path_ends_with(path, &["transactionCoding", "transactionCode"]) {
        tx.code = Some(text.to_string());
    } else if path_ends_with(path, &["transactionShares", "value"]) {
        tx.shares = parse_f64(text);
    } else if path_ends_with(path, &["transactionPricePerShare", "value"]) {
        tx.price = parse_f64(text);
    } else if path_ends_with(path, &["transactionAcquiredDisposedCode", "value"]) {
        tx.acquired_disposed = Some(text.to_string());
    } else if path_ends_with(path, &["sharesOwnedFollowingTransaction", "value"]) {
        tx.shares_owned_after = parse_f64(text);
    } else if path_ends_with(path, &["directOrIndirectOwnership", "value"]) {
        tx.direct_or_indirect = Some(text.to_string());
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" => Some(true),
        "0" | "false" => Some(false),
        _ => None,
    }
}

fn transaction_type(code: Option<&str>, acquired_disposed: Option<&str>) -> Option<String> {
    let label = match code {
        Some("P") => "purchase",
        Some("S") => "sale",
        Some("A") => "grant/award",
        Some("M") => "option exercise/conversion",
        Some("G") => "gift",
        Some("F") => "tax withholding/payment",
        Some("D") => "disposition",
        Some("V") => "voluntary report",
        _ => match acquired_disposed {
            Some("A") => "acquisition",
            Some("D") => "disposition",
            _ => return None,
        },
    };
    Some(label.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_non_derivative_form4_transaction() {
        let filing = FilingRecord {
            accession: "0000000000-00-000001".to_string(),
            cik: 1,
            company: "ACME Inc.".to_string(),
            form: "4".to_string(),
            filing_date: "2026-01-02".to_string(),
            report_date: None,
            primary_document: None,
            primary_doc_description: None,
            is_xbrl: None,
            is_inline_xbrl: None,
            source_url: "https://example.test/index.html".to_string(),
            text_url: "https://example.test/submission.txt".to_string(),
        };
        let doc = SubmissionDocument {
            document_type: Some("4".to_string()),
            sequence: Some("1".to_string()),
            filename: Some("form4.xml".to_string()),
            description: Some("FORM 4".to_string()),
            content: sample_form4_xml().to_string(),
        };

        let records = parse_form4_documents(&filing, &[doc]).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].issuer.as_deref(), Some("ACME Inc."));
        assert_eq!(records[0].reporting_owner.as_deref(), Some("Jane Owner"));
        assert_eq!(records[0].transaction_type.as_deref(), Some("sale"));
        assert_eq!(records[0].shares, Some(10.0));
        assert_eq!(records[0].value, Some(125.0));
    }

    fn sample_form4_xml() -> &'static str {
        r#"
        <ownershipDocument>
          <issuer><issuerCik>0000000001</issuerCik><issuerName>ACME Inc.</issuerName></issuer>
          <reportingOwner>
            <reportingOwnerId><rptOwnerCik>0000000002</rptOwnerCik><rptOwnerName>Jane Owner</rptOwnerName></reportingOwnerId>
            <reportingOwnerRelationship><isOfficer>1</isOfficer><officerTitle>CFO</officerTitle></reportingOwnerRelationship>
          </reportingOwner>
          <nonDerivativeTable>
            <nonDerivativeTransaction>
              <securityTitle><value>Common Stock</value></securityTitle>
              <transactionDate><value>2026-01-01</value></transactionDate>
              <transactionCoding><transactionCode>S</transactionCode></transactionCoding>
              <transactionAmounts>
                <transactionShares><value>10</value></transactionShares>
                <transactionPricePerShare><value>12.5</value></transactionPricePerShare>
                <transactionAcquiredDisposedCode><value>D</value></transactionAcquiredDisposedCode>
              </transactionAmounts>
              <postTransactionAmounts><sharesOwnedFollowingTransaction><value>90</value></sharesOwnedFollowingTransaction></postTransactionAmounts>
              <ownershipNature><directOrIndirectOwnership><value>D</value></directOrIndirectOwnership></ownershipNature>
            </nonDerivativeTransaction>
          </nonDerivativeTable>
        </ownershipDocument>
        "#
    }
}
