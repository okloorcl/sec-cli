mod cache;
pub mod company_search;
mod prompt;

use anyhow::{Context, Result, anyhow, bail};

use cache::{read_cached_resolve, write_cached_resolve};
use company_search::find_13f_manager_cik;
use prompt::{
    RawResolveCandidate, SYSTEM_PROMPT, cik_value, expanded_prompt, parse_or_repair, user_prompt,
    value_to_string,
};

use crate::sec::{
    client::SecClient,
    llm::{LlmClient, LlmConfig},
    models::{FilingQuery, ResolveCandidateRecord, ResolveValidationRecord},
    utils::is_legal_suffix,
};

#[derive(Debug, Clone)]
pub enum ResolveInput {
    Query(String),
    Cik(u64),
    Manager(String),
}

struct CheckedValidation {
    record: ResolveValidationRecord,
    cik: Option<u64>,
    company: Option<String>,
}

impl SecClient {
    pub async fn resolve_query(
        &self,
        query: &str,
        verify: bool,
        overrides: Option<LlmConfig>,
    ) -> Result<Vec<ResolveCandidateRecord>> {
        if verify
            && let Some(cached) = read_cached_resolve(self.cache_dir(), query)?
            && cached.validation_status == "verified_13f"
            && cached.latest_accession.is_some()
        {
            return Ok(vec![cached.to_record()]);
        }
        let llm = LlmClient::new(LlmConfig::load_with_overrides(overrides)?);
        let raw = llm
            .complete(SYSTEM_PROMPT, &user_prompt(query))
            .await
            .with_context(|| format!("failed to resolve '{}'", query))?;
        let mut candidates = parse_or_repair(&llm, query, &raw).await?;
        if candidates.is_empty() {
            let retry = llm
                .complete(SYSTEM_PROMPT, &expanded_prompt(query))
                .await
                .with_context(|| format!("failed to retry empty LLM response for '{}'", query))?;
            candidates = parse_or_repair(&llm, query, &retry).await?;
        }
        let mut records = Vec::new();
        for candidate in candidates.into_iter().take(5) {
            records.push(self.materialize_candidate(query, candidate, verify).await?);
        }
        if verify {
            for record in &records {
                if record.validation.status == "verified_13f" {
                    write_cached_resolve(self.cache_dir(), record)?;
                }
            }
        }
        Ok(records)
    }

    pub async fn resolve_input(&self, input: ResolveInput) -> Result<Vec<ResolveCandidateRecord>> {
        match input {
            ResolveInput::Query(query) => self.resolve_query(&query, true, None).await,
            ResolveInput::Cik(cik) => self.resolve_known_cik(cik, cik.to_string()).await,
            ResolveInput::Manager(manager) => self.resolve_manager(&manager).await,
        }
    }

    pub async fn resolve_manager(&self, manager: &str) -> Result<Vec<ResolveCandidateRecord>> {
        let Some(cik) = find_13f_manager_cik(self, manager).await? else {
            return Ok(vec![ResolveCandidateRecord {
                query: manager.to_string(),
                candidate_type: "filing_manager".to_string(),
                investor: None,
                manager: Some(manager.to_string()),
                cik: None,
                confidence: Some("rule".to_string()),
                relationship: Some("SEC company search did not find a 13F manager CIK".to_string()),
                evidence_queries: vec![manager.to_string()],
                notes: None,
                validation: validation("missing_cik", None),
                next_commands: Vec::new(),
            }]);
        };
        self.resolve_known_cik(cik, manager.to_string()).await
    }

    async fn resolve_known_cik(
        &self,
        cik: u64,
        query: String,
    ) -> Result<Vec<ResolveCandidateRecord>> {
        let checked = self.validate_13f_candidate(Some(cik)).await?;
        let validation = checked.record;
        let manager = checked.company.clone().unwrap_or_else(|| query.clone());
        let record = ResolveCandidateRecord {
            query,
            candidate_type: "filing_manager".to_string(),
            investor: None,
            manager: Some(manager),
            cik: Some(cik),
            confidence: Some("rule".to_string()),
            relationship: Some("Resolved from standard CIK/manager input".to_string()),
            evidence_queries: Vec::new(),
            notes: None,
            next_commands: next_commands(Some(cik)),
            validation,
        };
        if record.validation.status == "verified_13f" {
            write_cached_resolve(self.cache_dir(), &record)?;
        }
        Ok(vec![record])
    }

    async fn materialize_candidate(
        &self,
        query: &str,
        candidate: RawResolveCandidate,
        verify: bool,
    ) -> Result<ResolveCandidateRecord> {
        let mut cik = candidate.cik.as_ref().and_then(cik_value);
        let mut notes = candidate.notes;
        let mut checked = if verify {
            self.validate_13f_candidate(cik).await?
        } else {
            CheckedValidation {
                record: ResolveValidationRecord {
                    status: "not_verified".to_string(),
                    latest_accession: None,
                    latest_report_date: None,
                    latest_filing_date: None,
                    source_url: None,
                },
                cik: None,
                company: None,
            }
        };
        if verify
            && (checked.record.status != "verified_13f"
                || !company_matches_candidate(
                    checked.company.as_deref(),
                    candidate.manager.as_deref(),
                    candidate.investor.as_deref(),
                ))
        {
            if let Some(corrected) = self
                .correct_cik_from_sec_search(
                    candidate.manager.as_deref(),
                    candidate.investor.as_deref(),
                )
                .await?
                && Some(corrected) != cik
            {
                let corrected_validation = self.validate_13f_candidate(Some(corrected)).await?;
                if corrected_validation.record.status == "verified_13f"
                    && company_matches_candidate(
                        corrected_validation.company.as_deref(),
                        candidate.manager.as_deref(),
                        candidate.investor.as_deref(),
                    )
                {
                    notes = append_note(
                        notes,
                        &format!(
                            "SEC company search corrected CIK from {} to {}.",
                            cik.map(|value| value.to_string())
                                .unwrap_or_else(|| "missing".to_string()),
                            corrected
                        ),
                    );
                    cik = Some(corrected);
                    checked = corrected_validation;
                }
            }
        }
        let mut validation = checked.record;
        if validation.status == "verified_13f"
            && !company_matches_candidate(
                checked.company.as_deref(),
                candidate.manager.as_deref(),
                candidate.investor.as_deref(),
            )
        {
            validation.status = "company_mismatch".to_string();
            cik = checked.cik;
        }
        Ok(ResolveCandidateRecord {
            query: query.to_string(),
            candidate_type: candidate
                .candidate_type
                .unwrap_or_else(|| "filing_manager".to_string()),
            investor: candidate.investor,
            manager: candidate.manager,
            cik,
            confidence: candidate.confidence.map(value_to_string),
            relationship: candidate.relationship,
            evidence_queries: candidate.evidence_queries.unwrap_or_default(),
            notes,
            next_commands: next_commands(cik),
            validation,
        })
    }

    async fn validate_13f_candidate(&self, cik: Option<u64>) -> Result<CheckedValidation> {
        let Some(cik) = cik else {
            return Ok(CheckedValidation {
                record: validation("missing_cik", None),
                cik: None,
                company: None,
            });
        };
        let filings = self
            .filings(FilingQuery {
                cik,
                form: Some("13F-HR".to_string()),
                latest: 1,
                from: None,
                to: None,
                include_amends: false,
            })
            .await?;
        let Some(filing) = filings.first() else {
            return Ok(CheckedValidation {
                record: validation("no_recent_13f", None),
                cik: Some(cik),
                company: None,
            });
        };
        Ok(CheckedValidation {
            record: ResolveValidationRecord {
                status: "verified_13f".to_string(),
                latest_accession: Some(filing.accession.clone()),
                latest_report_date: filing.report_date.clone(),
                latest_filing_date: Some(filing.filing_date.clone()),
                source_url: Some(filing.source_url.clone()),
            },
            cik: Some(cik),
            company: Some(filing.company.clone()),
        })
    }

    async fn correct_cik_from_sec_search(
        &self,
        manager: Option<&str>,
        investor: Option<&str>,
    ) -> Result<Option<u64>> {
        for name in [manager, investor].into_iter().flatten() {
            if let Some(cik) = find_13f_manager_cik(self, name).await? {
                return Ok(Some(cik));
            }
        }
        Ok(None)
    }
}

fn next_commands(cik: Option<u64>) -> Vec<String> {
    let Some(cik) = cik else {
        return Vec::new();
    };
    vec![
        format!("sec 13f-summary --cik {cik} --latest 2 --pretty"),
        format!("sec 13f-diff --cik {cik} --pretty"),
        format!("sec report --cik {cik} --kind portfolio --limit 15"),
    ]
}

fn validation(status: &str, source_url: Option<String>) -> ResolveValidationRecord {
    ResolveValidationRecord {
        status: status.to_string(),
        latest_accession: None,
        latest_report_date: None,
        latest_filing_date: None,
        source_url,
    }
}

fn append_note(existing: Option<String>, note: &str) -> Option<String> {
    Some(match existing {
        Some(existing) if !existing.trim().is_empty() => format!("{existing} {note}"),
        _ => note.to_string(),
    })
}

fn company_matches_candidate(
    sec_company: Option<&str>,
    manager: Option<&str>,
    investor: Option<&str>,
) -> bool {
    let Some(sec_company) = sec_company else {
        return false;
    };
    let left = comparable_name(sec_company);
    [manager, investor].into_iter().flatten().any(|candidate| {
        let right = comparable_name(candidate);
        comparable_names_match(&left, &right)
    })
}

fn comparable_names_match(left: &str, right: &str) -> bool {
    if left.is_empty() || right.is_empty() {
        return false;
    }
    if left == right {
        return true;
    }

    let left_words = left.split_whitespace().collect::<Vec<_>>();
    let right_words = right.split_whitespace().collect::<Vec<_>>();
    let (short, long) = if left_words.len() <= right_words.len() {
        (left_words, right_words)
    } else {
        (right_words, left_words)
    };
    if short.len() < 2 || short.join(" ").len() < 8 {
        return false;
    }
    long.windows(short.len()).any(|window| window == short)
}

fn comparable_name(name: &str) -> String {
    name.replace('&', " ")
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(|part| part.to_ascii_lowercase())
        .filter(|part| !part.is_empty() && !is_legal_suffix(part))
        .collect::<Vec<_>>()
        .join(" ")
}

pub async fn resolve_verified_13f_cik(client: &SecClient, query: &str) -> Result<(u64, String)> {
    if let Some(cached) = read_cached_resolve(client.cache_dir(), query)? {
        if cached.validation_status == "verified_13f" {
            return Ok((cached.cik, cached.subject));
        }
    }
    let records = client.resolve_query(query, true, None).await?;
    let best = records
        .iter()
        .find(|record| record.validation.status == "verified_13f" && record.cik.is_some())
        .or_else(|| records.iter().find(|record| record.cik.is_some()))
        .ok_or_else(|| anyhow!("LLM did not return a usable CIK for '{}'", query))?;
    let cik = best.cik.ok_or_else(|| anyhow!("candidate missing CIK"))?;
    if best.validation.status != "verified_13f" {
        bail!(
            "resolved '{}' to CIK {}, but SEC validation status is '{}'",
            query,
            cik,
            best.validation.status
        );
    }
    Ok((
        cik,
        best.manager
            .clone()
            .or_else(|| best.investor.clone())
            .unwrap_or_else(|| query.to_string()),
    ))
}

pub async fn resolve_verified_13f_manager(
    client: &SecClient,
    manager: &str,
) -> Result<(u64, String)> {
    let records = client.resolve_manager(manager).await?;
    let best = records
        .iter()
        .find(|record| record.validation.status == "verified_13f" && record.cik.is_some())
        .ok_or_else(|| anyhow!("SEC did not find a verified 13F manager for '{}'", manager))?;
    Ok((
        best.cik.ok_or_else(|| anyhow!("candidate missing CIK"))?,
        best.manager.clone().unwrap_or_else(|| manager.to_string()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_company_names_without_legal_suffixes() {
        assert!(company_matches_candidate(
            Some("H&H International Investment, LLC"),
            Some("H&H INTERNATIONAL INVESTMENT GROUP, LTD."),
            None
        ));
        assert!(!company_matches_candidate(
            Some("Scion Asset Management, LLC"),
            Some("H&H INTERNATIONAL INVESTMENT GROUP, LTD."),
            None
        ));
        assert!(!company_matches_candidate(
            Some("Very Large Industrial Group Inc."),
            Some("The Group"),
            None
        ));
    }
}
