mod company_search;

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use serde_json::Value;

use company_search::find_13f_manager_cik;

use crate::sec::{
    client::SecClient,
    llm::{LlmClient, LlmConfig},
    models::{FilingQuery, ResolveCandidateRecord, ResolveValidationRecord},
};

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
        let llm = LlmClient::new(LlmConfig::load_with_overrides(overrides)?);
        let raw = llm
            .complete(SYSTEM_PROMPT, &user_prompt(query))
            .await
            .with_context(|| format!("failed to resolve '{}'", query))?;
        let candidates = parse_candidates(query, &raw)?;
        let mut records = Vec::new();
        for candidate in candidates.into_iter().take(5) {
            records.push(self.materialize_candidate(query, candidate, verify).await?);
        }
        Ok(records)
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

#[derive(Debug, Deserialize)]
struct RawResolveCandidate {
    #[serde(default)]
    candidate_type: Option<String>,
    #[serde(default)]
    investor: Option<String>,
    #[serde(default)]
    manager: Option<String>,
    #[serde(default)]
    cik: Option<Value>,
    #[serde(default)]
    confidence: Option<Value>,
    #[serde(default)]
    relationship: Option<String>,
    #[serde(default)]
    evidence_queries: Option<Vec<String>>,
    #[serde(default)]
    notes: Option<String>,
}

const SYSTEM_PROMPT: &str = r#"You resolve public investor or fund names to SEC EDGAR Form 13F filing managers.
Return JSON only. Do not use markdown. Do not invent facts.
The answer must be either an array, or an object with a candidates array.
Each candidate must include:
candidate_type, investor, manager, cik, confidence, relationship, evidence_queries, notes.
Use cik only when you believe it is the SEC registrant/filing manager CIK.
If the query is ambiguous or unsupported, return [].
Prefer legal filing managers that submit Form 13F-HR."#;

fn user_prompt(query: &str) -> String {
    format!("Resolve this investor/fund/person name to SEC 13F filing manager candidates: {query}")
}

fn parse_candidates(query: &str, raw: &str) -> Result<Vec<RawResolveCandidate>> {
    let value =
        extract_json(raw).with_context(|| format!("LLM returned non-JSON for '{query}'"))?;
    let candidates = if value.is_array() {
        value
    } else {
        value
            .get("candidates")
            .cloned()
            .ok_or_else(|| anyhow!("LLM JSON missing candidates array"))?
    };
    serde_json::from_value(candidates).context("failed to parse resolve candidates")
}

fn extract_json(raw: &str) -> Result<Value> {
    let trimmed = raw.trim();
    if let Ok(value) = serde_json::from_str(trimmed) {
        return Ok(value);
    }
    let without_fence = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|text| text.strip_suffix("```"))
        .map(str::trim)
        .unwrap_or(trimmed);
    if let Ok(value) = serde_json::from_str(without_fence) {
        return Ok(value);
    }
    let start = without_fence
        .find(['[', '{'])
        .ok_or_else(|| anyhow!("no JSON object or array found"))?;
    let end = without_fence
        .rfind([']', '}'])
        .ok_or_else(|| anyhow!("no JSON object or array end found"))?;
    serde_json::from_str(&without_fence[start..=end]).context("failed to parse extracted JSON")
}

fn cik_value(value: &Value) -> Option<u64> {
    value.as_u64().or_else(|| {
        value
            .as_str()
            .map(|text| {
                text.chars()
                    .filter(|ch| ch.is_ascii_digit())
                    .collect::<String>()
            })
            .and_then(|digits| digits.parse::<u64>().ok())
    })
}

fn value_to_string(value: Value) -> String {
    value
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string())
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
    [manager, investor].into_iter().flatten().any(|candidate| {
        let left = comparable_name(sec_company);
        let right = comparable_name(candidate);
        !left.is_empty() && !right.is_empty() && (left.contains(&right) || right.contains(&left))
    })
}

fn comparable_name(name: &str) -> String {
    let stop_words = [
        "inc",
        "incorporated",
        "corp",
        "corporation",
        "co",
        "company",
        "llc",
        "ltd",
        "limited",
        "lp",
        "l.p",
        "group",
        "del",
    ];
    name.replace('&', " ")
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(|part| part.to_ascii_lowercase())
        .filter(|part| !part.is_empty() && !stop_words.contains(&part.as_str()))
        .collect::<Vec<_>>()
        .join(" ")
}

pub async fn resolve_verified_13f_cik(client: &SecClient, query: &str) -> Result<(u64, String)> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_candidate_array_from_fenced_json() {
        let raw = r#"```json
        [{"candidate_type":"person","investor":"A","manager":"B","cik":"0001234567","confidence":0.8}]
        ```"#;
        let records = parse_candidates("A", raw).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(cik_value(records[0].cik.as_ref().unwrap()), Some(1234567));
    }

    #[test]
    fn extracts_candidates_object() {
        let raw = r#"{"candidates":[{"manager":"B","cik":123}]}"#;
        let records = parse_candidates("B", raw).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(cik_value(records[0].cik.as_ref().unwrap()), Some(123));
    }

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
    }
}
