use anyhow::Result;

use crate::sec::{
    client::SecClient,
    edgar::accession_document_url,
    models::{DocumentQuery, DocumentRecord, FilingQuery, FilingRecord},
};

use super::SubmissionDocument;

impl SecClient {
    pub async fn document_records(&self, query: DocumentQuery) -> Result<Vec<DocumentRecord>> {
        let filings = self
            .filings(FilingQuery {
                cik: query.cik,
                form: query.form,
                latest: query.latest,
                from: None,
                to: None,
                include_amends: query.include_amends,
            })
            .await?;

        let mut records = Vec::new();
        for (filing, docs) in self.filing_documents_batch(filings).await? {
            records.extend(docs.iter().map(|doc| document_record(&filing, doc)));
        }
        if let Some(limit) = query.limit {
            records.truncate(limit);
        }
        Ok(records)
    }
}

pub(crate) fn document_record(filing: &FilingRecord, doc: &SubmissionDocument) -> DocumentRecord {
    DocumentRecord {
        accession: filing.accession.clone(),
        cik: filing.cik,
        company: filing.company.clone(),
        form: filing.form.clone(),
        filing_date: filing.filing_date.clone(),
        document_type: doc.document_type.clone(),
        sequence: doc.sequence.clone(),
        filename: doc.filename.clone(),
        description: doc.description.clone(),
        content_type: doc.content_type().to_string(),
        byte_length: doc.content.len(),
        is_primary: doc.is_primary(),
        source_url: filing.source_url.clone(),
        document_url: doc
            .filename
            .as_deref()
            .map(|filename| accession_document_url(filing.cik, &filing.accession, filename)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_document_record_with_archive_url() {
        let filing = FilingRecord {
            accession: "0000320193-26-000001".to_string(),
            cik: 320193,
            company: "Apple Inc.".to_string(),
            form: "10-K".to_string(),
            filing_date: "2026-01-01".to_string(),
            report_date: None,
            primary_document: None,
            primary_doc_description: None,
            is_xbrl: None,
            is_inline_xbrl: None,
            source_url: "https://example.test/index.html".to_string(),
            text_url: "https://example.test/submission.txt".to_string(),
        };
        let doc = SubmissionDocument {
            document_type: Some("10-K".to_string()),
            sequence: Some("1".to_string()),
            filename: Some("aapl-20260101.htm".to_string()),
            description: Some("FORM 10-K".to_string()),
            content: "<HTML></HTML>".to_string(),
        };

        let record = document_record(&filing, &doc);

        assert!(record.is_primary);
        assert_eq!(record.content_type, "html");
        assert_eq!(
            record.document_url.as_deref(),
            Some(
                "https://www.sec.gov/Archives/edgar/data/320193/000032019326000001/aapl-20260101.htm"
            )
        );
    }
}
