use std::sync::LazyLock;

use anyhow::Result;
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
    let occurrences = item_occurrences(text);
    let mut starts = Vec::new();
    for (idx, occurrence) in occurrences.iter().enumerate() {
        if !same_item(&occurrence.item, target.item) {
            continue;
        }
        let title_end = occurrences
            .get(idx + 1)
            .map(|next| next.start)
            .unwrap_or(text.len());
        let title = text[occurrence.end..title_end].trim();
        if target
            .aliases
            .iter()
            .any(|alias| title_matches(title, alias))
        {
            starts.push(occurrence.start);
        }
    }

    if starts.is_empty() {
        starts.extend(
            occurrences
                .iter()
                .filter(|occurrence| same_item(&occurrence.item, target.item))
                .map(|occurrence| occurrence.start),
        );
    }

    starts.sort_unstable();
    starts.dedup();
    Ok(starts)
}

fn next_heading(text: &str, from: usize, target: &SectionTarget) -> Option<usize> {
    item_occurrences(&text[from..])
        .into_iter()
        .filter(|occurrence| {
            target
                .next_items
                .iter()
                .any(|target_item| same_item(&occurrence.item, target_item))
        })
        .map(|occurrence| from + occurrence.start)
        .min()
}

#[derive(Debug)]
struct ItemOccurrence {
    item: String,
    start: usize,
    end: usize,
}

fn item_occurrences(text: &str) -> Vec<ItemOccurrence> {
    static ITEM_NUMBER_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\bitem\s+([0-9]+[a-z]?)\s*[\.\-:]").expect("valid section next item regex")
    });

    ITEM_NUMBER_RE
        .captures_iter(text)
        .filter_map(|capture| {
            let item = capture.get(1)?.as_str().to_string();
            let full = capture.get(0)?;
            Some(ItemOccurrence {
                item,
                start: full.start(),
                end: full.end(),
            })
        })
        .collect()
}

fn title_matches(title: &str, alias: &str) -> bool {
    normalize_heading(title).contains(&normalize_heading(alias))
}

fn same_item(left: &str, right: &str) -> bool {
    normalize_heading(left) == normalize_heading(right)
}

fn normalize_heading(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
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
