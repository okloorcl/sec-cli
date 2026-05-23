use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::sec::models::{ResolveCandidateRecord, ResolveValidationRecord};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct CachedResolve {
    pub query: String,
    pub subject: String,
    pub cik: u64,
    pub manager: Option<String>,
    pub investor: Option<String>,
    pub validation_status: String,
    #[serde(default)]
    pub latest_accession: Option<String>,
    #[serde(default)]
    pub latest_report_date: Option<String>,
    #[serde(default)]
    pub latest_filing_date: Option<String>,
    pub source_url: Option<String>,
}

pub(super) fn read_cached_resolve(cache_dir: &Path, query: &str) -> Result<Option<CachedResolve>> {
    let cache = read_cache(cache_dir)?;
    Ok(cache.get(&cache_key(query)).cloned())
}

pub(super) fn write_cached_resolve(
    cache_dir: &Path,
    record: &ResolveCandidateRecord,
) -> Result<()> {
    let Some(cik) = record.cik else {
        return Ok(());
    };
    let mut cache = read_cache(cache_dir)?;
    let subject = record
        .manager
        .clone()
        .or_else(|| record.investor.clone())
        .unwrap_or_else(|| record.query.clone());
    cache.insert(
        cache_key(&record.query),
        CachedResolve {
            query: record.query.clone(),
            subject,
            cik,
            manager: record.manager.clone(),
            investor: record.investor.clone(),
            validation_status: record.validation.status.clone(),
            latest_accession: record.validation.latest_accession.clone(),
            latest_report_date: record.validation.latest_report_date.clone(),
            latest_filing_date: record.validation.latest_filing_date.clone(),
            source_url: record.validation.source_url.clone(),
        },
    );
    write_cache(cache_dir, &cache)
}

fn read_cache(cache_dir: &Path) -> Result<BTreeMap<String, CachedResolve>> {
    let path = cache_path(cache_dir);
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let bytes = fs::read(&path)
        .with_context(|| format!("failed to read resolve cache {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse resolve cache {}", path.display()))
}

fn write_cache(cache_dir: &Path, cache: &BTreeMap<String, CachedResolve>) -> Result<()> {
    fs::create_dir_all(cache_dir)?;
    let path = cache_path(cache_dir);
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(cache)?)?;
    fs::rename(tmp, path)?;
    Ok(())
}

fn cache_path(cache_dir: &Path) -> std::path::PathBuf {
    cache_dir.join("resolve-cache.json")
}

fn cache_key(query: &str) -> String {
    query.trim().to_ascii_lowercase()
}

impl CachedResolve {
    pub(super) fn to_record(&self) -> ResolveCandidateRecord {
        ResolveCandidateRecord {
            query: self.query.clone(),
            candidate_type: "filing_manager".to_string(),
            investor: self.investor.clone(),
            manager: self.manager.clone().or_else(|| Some(self.subject.clone())),
            cik: Some(self.cik),
            confidence: Some("cache".to_string()),
            relationship: Some("Previously SEC-verified local cache entry".to_string()),
            evidence_queries: Vec::new(),
            notes: Some("Loaded from local verified resolve cache.".to_string()),
            validation: ResolveValidationRecord {
                status: self.validation_status.clone(),
                latest_accession: self.latest_accession.clone(),
                latest_report_date: self.latest_report_date.clone(),
                latest_filing_date: self.latest_filing_date.clone(),
                source_url: self.source_url.clone(),
            },
            next_commands: vec![
                format!("sec 13f-summary --cik {} --latest 2 --pretty", self.cik),
                format!("sec 13f-diff --cik {} --pretty", self.cik),
                format!("sec report --cik {} --kind portfolio --limit 15", self.cik),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_cache_keys() {
        assert_eq!(cache_key("  Duan Yongping "), "duan yongping");
    }
}
