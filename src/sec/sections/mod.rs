use anyhow::{Context, Result};
use regex::Regex;

use crate::sec::{
    client::SecClient,
    documents::{DocumentSet, SubmissionDocument, read::plain_text},
    edgar::accession_document_url,
    models::{FilingQuery, FilingRecord, SectionQuery, SectionRecord},
    utils::truncate_utf8,
};

impl SecClient {
    pub async fn sections(&self, query: SectionQuery) -> Result<Vec<SectionRecord>> {
        let filings = self
            .filings(FilingQuery {
                cik: query.cik,
                form: query.form.or_else(|| Some("10-K".to_string())),
                latest: query.latest,
                from: None,
                to: None,
                include_amends: query.include_amends,
            })
            .await?;

        let target = SectionTarget::from_input(&query.item)?;
        let mut records = Vec::new();
        for filing in filings {
            if query
                .accession
                .as_deref()
                .is_some_and(|accession| accession != filing.accession)
            {
                continue;
            }
            let docs = self.filing_documents(&filing).await?;
            let Some(doc) = DocumentSet::new(&docs).primary_documents().next() else {
                continue;
            };
            if let Some(record) = extract_section(&filing, doc, &target, query.limit_bytes)? {
                records.push(record);
            }
        }

        Ok(records)
    }
}

struct SectionTarget {
    item: &'static str,
    title: &'static str,
    aliases: &'static [&'static str],
    next_items: &'static [&'static str],
}

impl SectionTarget {
    fn from_input(input: &str) -> Result<Self> {
        let normalized = input
            .trim()
            .to_ascii_lowercase()
            .replace(['_', '-', ' ', '&'], "");

        let target = match normalized.as_str() {
            "1" | "business" => Self::business(),
            "1a" | "risk" | "riskfactors" => Self::risk_factors(),
            "1b" | "unresolvedstaffcomments" => Self::staff_comments(),
            "1c" | "cybersecurity" => Self::cybersecurity(),
            "2" | "properties" => Self::properties(),
            "3" | "legalproceedings" => Self::legal_proceedings(),
            "7" | "mda" | "md&a" | "managementdiscussion" => Self::mda(),
            "7a" | "marketrisk" => Self::market_risk(),
            "8" | "financialstatements" => Self::financial_statements(),
            _ => anyhow::bail!("unsupported section item '{}'", input),
        };
        Ok(target)
    }

    fn business() -> Self {
        Self::new("1", "Business", &["business"], &["1A", "1B", "1C", "2"])
    }

    fn risk_factors() -> Self {
        Self::new("1A", "Risk Factors", &["risk factors"], &["1B", "1C", "2"])
    }

    fn staff_comments() -> Self {
        Self::new(
            "1B",
            "Unresolved Staff Comments",
            &["unresolved staff comments"],
            &["1C", "2", "3"],
        )
    }

    fn cybersecurity() -> Self {
        Self::new("1C", "Cybersecurity", &["cybersecurity"], &["2", "3"])
    }

    fn properties() -> Self {
        Self::new("2", "Properties", &["properties"], &["3", "4", "5"])
    }

    fn legal_proceedings() -> Self {
        Self::new(
            "3",
            "Legal Proceedings",
            &["legal proceedings"],
            &["4", "5"],
        )
    }

    fn mda() -> Self {
        Self::new(
            "7",
            "Management's Discussion and Analysis",
            &[
                "management's discussion and analysis",
                "managements discussion and analysis",
            ],
            &["7A", "8", "9"],
        )
    }

    fn market_risk() -> Self {
        Self::new(
            "7A",
            "Quantitative and Qualitative Disclosures About Market Risk",
            &["market risk"],
            &["8", "9"],
        )
    }

    fn financial_statements() -> Self {
        Self::new(
            "8",
            "Financial Statements and Supplementary Data",
            &["financial statements"],
            &["9", "9A", "9B", "9C", "10"],
        )
    }

    fn new(
        item: &'static str,
        title: &'static str,
        aliases: &'static [&'static str],
        next_items: &'static [&'static str],
    ) -> Self {
        Self {
            item,
            title,
            aliases,
            next_items,
        }
    }
}

fn extract_section(
    filing: &FilingRecord,
    doc: &SubmissionDocument,
    target: &SectionTarget,
    limit_bytes: Option<usize>,
) -> Result<Option<SectionRecord>> {
    let text = plain_text(&doc.content);
    let Some((start, end)) = locate_section(&text, target)? else {
        return Ok(None);
    };

    let full_content = text[start..end].trim().to_string();
    let byte_length = full_content.len();
    let (content, truncated) = truncate_utf8(&full_content, limit_bytes);

    Ok(Some(SectionRecord {
        accession: filing.accession.clone(),
        cik: filing.cik,
        company: filing.company.clone(),
        form: filing.form.clone(),
        filing_date: filing.filing_date.clone(),
        item: target.item.to_string(),
        title: target.title.to_string(),
        start_offset: start,
        end_offset: end,
        byte_length,
        returned_bytes: content.len(),
        truncated,
        document: doc.filename.clone(),
        document_sequence: doc.sequence.clone(),
        document_description: doc.description.clone(),
        document_url: doc
            .filename
            .as_deref()
            .map(|filename| accession_document_url(filing.cik, &filing.accession, filename)),
        source_url: filing.source_url.clone(),
        content,
    }))
}

fn locate_section(text: &str, target: &SectionTarget) -> Result<Option<(usize, usize)>> {
    let starts = start_candidates(text, target)?;
    let Some((start, end)) = starts
        .into_iter()
        .filter_map(|start| {
            let end = next_heading(text, start + 1, target).unwrap_or(text.len());
            (end > start).then_some((start, end, end - start))
        })
        .max_by_key(|(_, _, len)| *len)
        .map(|(start, end, _)| (start, end))
    else {
        return Ok(None);
    };
    Ok(Some((start, end)))
}

fn start_candidates(text: &str, target: &SectionTarget) -> Result<Vec<usize>> {
    let mut starts = Vec::new();
    for alias in target.aliases {
        let pattern = format!(
            r"(?i)\bitem\s+{}\s*[\.\-:]*\s*{}",
            regex::escape(target.item),
            flexible_words(alias)
        );
        let regex = Regex::new(&pattern).context("invalid section start regex")?;
        starts.extend(regex.find_iter(text).map(|m| m.start()));
    }

    if starts.is_empty() {
        let pattern = format!(r"(?i)\bitem\s+{}\s*[\.\-:]", regex::escape(target.item));
        let regex = Regex::new(&pattern).context("invalid item start regex")?;
        starts.extend(regex.find_iter(text).map(|m| m.start()));
    }

    starts.sort_unstable();
    starts.dedup();
    Ok(starts)
}

fn next_heading(text: &str, from: usize, target: &SectionTarget) -> Option<usize> {
    target
        .next_items
        .iter()
        .filter_map(|item| {
            let pattern = format!(r"(?i)\bitem\s+{}\s*[\.\-:]", regex::escape(item));
            Regex::new(&pattern)
                .ok()
                .and_then(|regex| regex.find(&text[from..]))
                .map(|m| from + m.start())
        })
        .min()
}

fn flexible_words(value: &str) -> String {
    value
        .split_whitespace()
        .map(regex::escape)
        .collect::<Vec<_>>()
        .join(r"\s+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_largest_matching_section() {
        let target = SectionTarget::from_input("risk-factors").unwrap();
        let text = "Item 1A. Risk Factors Item 1B. Unresolved Staff Comments \
                    Item 1A. Risk Factors Real risk content here. More detail. \
                    Item 1B. Unresolved Staff Comments";

        let (start, end) = locate_section(text, &target).unwrap().unwrap();

        assert!(text[start..end].contains("Real risk content"));
    }

    #[test]
    fn supports_mda_alias() {
        let target = SectionTarget::from_input("mda").unwrap();

        assert_eq!(target.item, "7");
    }
}
