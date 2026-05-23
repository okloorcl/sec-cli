use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::sec::{
    client::SecClient,
    documents::{SubmissionDocument, records::document_record},
    models::{
        ArchiveDocumentRecord, ArchiveFilingRecord, ArchiveManifestRecord, ArchiveQuery,
        FilingQuery, FilingRecord,
    },
    utils::truncate_utf8,
};

impl SecClient {
    pub async fn archive_filings(&self, query: ArchiveQuery) -> Result<ArchiveManifestRecord> {
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
        let archived = self.filing_documents_batch(filings).await?;

        std::fs::create_dir_all(&query.out_dir)
            .with_context(|| format!("failed to create archive dir {}", query.out_dir.display()))?;

        let mut filing_records = Vec::new();
        for (filing, docs) in archived {
            filing_records.push(write_filing_archive(&query, filing, docs)?);
        }

        let document_count = filing_records
            .iter()
            .map(|filing| filing.documents.len())
            .sum();
        let manifest_path = query.out_dir.join("manifest.json");
        let manifest = ArchiveManifestRecord {
            cik: query.cik,
            form: query.form,
            latest: query.latest,
            include_amends: query.include_amends,
            primary_only: query.primary_only,
            limit_bytes: query.limit_bytes,
            filing_count: filing_records.len(),
            document_count,
            out_dir: query.out_dir.display().to_string(),
            manifest_path: manifest_path.display().to_string(),
            filings: filing_records,
        };
        std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)
            .with_context(|| format!("failed to write {}", manifest_path.display()))?;
        Ok(manifest)
    }
}

fn write_filing_archive(
    query: &ArchiveQuery,
    filing: FilingRecord,
    docs: Vec<SubmissionDocument>,
) -> Result<ArchiveFilingRecord> {
    let filing_dir = query.out_dir.join(safe_path_part(&filing.accession));
    std::fs::create_dir_all(&filing_dir)
        .with_context(|| format!("failed to create {}", filing_dir.display()))?;

    let mut document_records = Vec::new();
    for (idx, doc) in docs
        .into_iter()
        .filter(|doc| !query.primary_only || doc.is_primary())
        .enumerate()
    {
        document_records.push(write_document(
            &filing,
            &filing_dir,
            idx,
            doc,
            query.limit_bytes,
        )?);
    }

    let record = ArchiveFilingRecord {
        accession: filing.accession.clone(),
        company: filing.company.clone(),
        form: filing.form.clone(),
        filing_date: filing.filing_date.clone(),
        source_url: filing.source_url.clone(),
        text_url: filing.text_url.clone(),
        directory: filing_dir.display().to_string(),
        documents: document_records,
    };
    std::fs::write(
        filing_dir.join("filing.json"),
        serde_json::to_vec_pretty(&record)?,
    )
    .with_context(|| format!("failed to write filing manifest for {}", filing.accession))?;
    Ok(record)
}

fn write_document(
    filing: &FilingRecord,
    filing_dir: &Path,
    idx: usize,
    doc: SubmissionDocument,
    limit_bytes: Option<usize>,
) -> Result<ArchiveDocumentRecord> {
    let record = document_record(filing, &doc);
    let path = filing_dir.join(document_filename(idx, &doc));
    let byte_length = doc.content.len();
    let (content, truncated) = truncate_utf8(&doc.content, limit_bytes);
    std::fs::write(&path, content.as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(ArchiveDocumentRecord {
        document_type: record.document_type,
        sequence: record.sequence,
        filename: record.filename,
        description: record.description,
        content_type: record.content_type,
        byte_length,
        written_bytes: content.len(),
        truncated,
        is_primary: record.is_primary,
        path: path.display().to_string(),
        document_url: record.document_url,
    })
}

fn document_filename(idx: usize, doc: &SubmissionDocument) -> PathBuf {
    let sequence = doc.sequence.as_deref().unwrap_or("doc");
    let name = doc.filename.as_deref().unwrap_or("document.txt");
    PathBuf::from(format!(
        "{:03}-{}-{}",
        idx + 1,
        safe_path_part(sequence),
        safe_path_part(name)
    ))
}

fn safe_path_part(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "unknown".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_archive_path_parts() {
        assert_eq!(safe_path_part("a/b c:1.htm"), "a_b_c_1.htm");
        assert_eq!(safe_path_part(""), "unknown");
    }

    #[test]
    fn builds_stable_document_names() {
        let doc = SubmissionDocument {
            document_type: Some("10-K".to_string()),
            sequence: Some("1".to_string()),
            filename: Some("a/b.htm".to_string()),
            description: None,
            content: String::new(),
        };

        assert_eq!(
            document_filename(0, &doc).to_string_lossy(),
            "001-1-a_b.htm"
        );
    }
}
