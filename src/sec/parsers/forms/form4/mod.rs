pub mod summary;
pub mod transactions;

use anyhow::Result;

use crate::sec::{
    client::SecClient,
    models::{FilingQuery, FilingRecord, Form4Query, Form4ReportRecord, Form4TransactionRecord},
};

impl SecClient {
    pub async fn form4_transactions(
        &self,
        query: Form4Query,
    ) -> Result<Vec<Form4TransactionRecord>> {
        let filings = self.form4_filings(&query).await?;
        let mut records = Vec::new();
        for (filing, docs) in self.filing_documents_batch(filings).await? {
            records.extend(transactions::parse_form4_transaction_documents(
                &filing, &docs,
            )?);
        }
        Ok(records)
    }

    pub async fn form4_reports(&self, query: Form4Query) -> Result<Vec<Form4ReportRecord>> {
        let filings = self.form4_filings(&query).await?;
        let mut records = Vec::new();
        for (filing, docs) in self.filing_documents_batch(filings).await? {
            records.extend(summary::parse_form4_report_documents(&filing, &docs)?);
        }
        Ok(records)
    }

    async fn form4_filings(&self, query: &Form4Query) -> Result<Vec<FilingRecord>> {
        self.filings(FilingQuery {
            cik: query.cik,
            form: Some("4".to_string()),
            latest: query.latest,
            from: None,
            to: None,
            include_amends: query.include_amends,
        })
        .await
    }
}

#[derive(Default, Clone)]
pub(crate) struct Issuer {
    pub(crate) name: Option<String>,
    pub(crate) cik: Option<String>,
    pub(crate) ticker: Option<String>,
}

#[derive(Default, Clone)]
pub(crate) struct Owner {
    pub(crate) name: Option<String>,
    pub(crate) cik: Option<String>,
    pub(crate) is_director: Option<bool>,
    pub(crate) is_officer: Option<bool>,
    pub(crate) is_ten_percent_owner: Option<bool>,
    pub(crate) is_other: Option<bool>,
    pub(crate) officer_title: Option<String>,
}

pub(crate) fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" => Some(true),
        "0" | "false" => Some(false),
        _ => None,
    }
}

pub(crate) fn transaction_type(
    code: Option<&str>,
    acquired_disposed: Option<&str>,
) -> Option<String> {
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

pub(crate) fn apply_owner_text(owner: &mut Owner, path: &[String], text: &str) {
    use crate::sec::parsers::forms::path_ends_with;

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
    } else if path_ends_with(path, &["reportingOwnerRelationship", "isOther"]) {
        owner.is_other = parse_bool(text);
    } else if path_ends_with(path, &["reportingOwnerRelationship", "officerTitle"]) {
        owner.officer_title = Some(text.to_string());
    }
}

#[cfg(test)]
pub(crate) fn sample_filing() -> FilingRecord {
    FilingRecord {
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
    }
}

#[cfg(test)]
pub(crate) fn sample_doc(content: &str) -> crate::sec::documents::SubmissionDocument {
    use crate::sec::documents::SubmissionDocument;

    SubmissionDocument {
        document_type: Some("4".to_string()),
        sequence: Some("1".to_string()),
        filename: Some("form4.xml".to_string()),
        description: Some("FORM 4".to_string()),
        content: content.to_string(),
    }
}

#[cfg(test)]
pub(crate) fn sample_form4_xml() -> &'static str {
    r#"
    <ownershipDocument>
      <periodOfReport>2026-01-01</periodOfReport>
      <notSubjectToSection16>0</notSubjectToSection16>
      <issuer><issuerCik>0000000001</issuerCik><issuerName>ACME Inc.</issuerName><issuerTradingSymbol>ACME</issuerTradingSymbol></issuer>
      <reportingOwner>
        <reportingOwnerId><rptOwnerCik>0000000002</rptOwnerCik><rptOwnerName>Jane Owner</rptOwnerName></reportingOwnerId>
        <reportingOwnerRelationship><isDirector>0</isDirector><isOfficer>1</isOfficer><isTenPercentOwner>0</isTenPercentOwner><isOther>0</isOther><officerTitle>CFO</officerTitle></reportingOwnerRelationship>
      </reportingOwner>
      <nonDerivativeTable>
        <nonDerivativeTransaction>
          <securityTitle><value>Common Stock</value></securityTitle>
          <transactionDate><value>2026-01-01</value></transactionDate>
          <transactionCoding><transactionFormType>4</transactionFormType><transactionCode>S</transactionCode><equitySwapInvolved>0</equitySwapInvolved></transactionCoding>
          <transactionAmounts>
            <transactionShares><value>10</value></transactionShares>
            <transactionPricePerShare><value>12.5</value></transactionPricePerShare>
            <transactionAcquiredDisposedCode><value>D</value></transactionAcquiredDisposedCode>
          </transactionAmounts>
          <postTransactionAmounts><sharesOwnedFollowingTransaction><value>90</value></sharesOwnedFollowingTransaction></postTransactionAmounts>
          <ownershipNature><directOrIndirectOwnership><value>D</value></directOrIndirectOwnership><natureOfOwnership><value>By Trust</value></natureOfOwnership></ownershipNature>
        </nonDerivativeTransaction>
      </nonDerivativeTable>
      <ownerSignature><signatureName>Jane Owner</signatureName><signatureDate>2026-01-02</signatureDate></ownerSignature>
      <footnotes><footnote id="F1">Rule 10b5-1 plan.</footnote></footnotes>
    </ownershipDocument>
    "#
}
