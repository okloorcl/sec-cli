use crate::sec::models::InvestorAliasRecord;

struct InvestorAlias {
    investor: &'static str,
    manager: &'static str,
    cik: u64,
    relationship: &'static str,
    aliases: &'static [&'static str],
    note: &'static str,
}

const INVESTORS: &[InvestorAlias] = &[
    InvestorAlias {
        investor: "Duan Yongping",
        manager: "H&H International Investment, LLC",
        cik: 1_759_760,
        relationship: "public investor name commonly associated with this 13F institutional manager",
        aliases: &[
            "段永平",
            "duan yongping",
            "yongping duan",
            "h&h",
            "h h international",
            "h&h international investment",
            "h international investment",
        ],
        note: "Use this CIK for SEC Form 13F-HR holdings. The SEC filing entity is the manager, not the individual's Chinese name.",
    },
    InvestorAlias {
        investor: "Warren Buffett",
        manager: "BERKSHIRE HATHAWAY INC",
        cik: 1_067_983,
        relationship: "public investor name commonly associated with Berkshire Hathaway 13F filings",
        aliases: &[
            "巴菲特",
            "warren buffett",
            "buffett",
            "berkshire",
            "berkshire hathaway",
            "brk",
        ],
        note: "Use this CIK for Berkshire Hathaway's SEC Form 13F-HR filings.",
    },
];

pub fn search_investors(query: &str) -> Vec<InvestorAliasRecord> {
    let normalized = normalize(query);
    INVESTORS
        .iter()
        .filter(|entry| matches_entry(entry, &normalized))
        .map(|entry| record_from(entry, query))
        .collect()
}

pub fn resolve_investor(query: &str) -> Option<InvestorAliasRecord> {
    search_investors(query).into_iter().next()
}

fn matches_entry(entry: &InvestorAlias, normalized: &str) -> bool {
    normalize(entry.investor).contains(normalized)
        || normalize(entry.manager).contains(normalized)
        || entry
            .aliases
            .iter()
            .any(|alias| normalize(alias).contains(normalized))
}

fn record_from(entry: &InvestorAlias, query: &str) -> InvestorAliasRecord {
    InvestorAliasRecord {
        query: query.to_string(),
        investor: entry.investor.to_string(),
        manager: entry.manager.to_string(),
        cik: entry.cik,
        relationship: entry.relationship.to_string(),
        aliases: entry
            .aliases
            .iter()
            .map(|alias| alias.to_string())
            .collect(),
        confidence: "curated_alias".to_string(),
        note: entry.note.to_string(),
    }
}

fn normalize(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-', '_', '.', ',', '&'], "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_chinese_alias() {
        let record = resolve_investor("段永平").unwrap();

        assert_eq!(record.cik, 1_759_760);
        assert_eq!(record.manager, "H&H International Investment, LLC");
    }
}
