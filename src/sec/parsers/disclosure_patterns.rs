use super::text_helpers::{capture_first, contains_ci};

const AUDITOR_PATTERNS: &[&str] = &[
    r"(?i)(Ernst\s*&\s*Young\s+LLP|EY\s+LLP)",
    r"(?i)(Deloitte\s*&\s*Touche\s+LLP|Deloitte\s+LLP)",
    r"(?i)(PricewaterhouseCoopers\s+LLP|PwC\s+LLP|PwC)",
    r"(?i)(KPMG\s+LLP)",
    r"(?i)(BDO\s+USA,\s+P\.?C\.?|BDO\s+LLP)",
    r"(?i)(Grant\s+Thornton\s+LLP)",
    r"(?i)(RSM\s+US\s+LLP)",
    r"(?i)(Mazars\s+USA\s+LLP)",
    r"(?i)(Marcum\s+LLP)",
    r"(?i)(Crowe\s+LLP)",
    r"(?i)(Baker\s+Tilly\s+US,\s+LLP)",
];

const UNDERWRITERS: &[&str] = &[
    "Morgan Stanley",
    "Goldman Sachs",
    "J.P. Morgan",
    "JP Morgan",
    "JPMorgan",
    "BofA Securities",
    "Merrill Lynch",
    "Barclays",
    "Citigroup",
    "Citi",
    "Deutsche Bank",
    "Evercore",
    "Jefferies",
    "RBC Capital Markets",
    "UBS",
    "Wells Fargo",
    "Credit Suisse",
    "BNP Paribas",
    "HSBC",
    "Nomura",
    "Mizuho",
    "Cowen",
    "Piper Sandler",
    "William Blair",
    "Stifel",
    "Raymond James",
    "Canaccord Genuity",
    "Needham",
    "Oppenheimer",
    "Baird",
];

pub(crate) fn known_auditor(text: &str) -> Option<String> {
    AUDITOR_PATTERNS
        .iter()
        .find_map(|pattern| capture_first(text, pattern))
}

pub(crate) fn known_underwriters(text: &str) -> Vec<String> {
    let mut values = Vec::new();
    for name in UNDERWRITERS {
        if contains_ci(text, name) && !values.iter().any(|value| value == name) {
            values.push((*name).to_string());
        }
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_expanded_auditor_names() {
        assert_eq!(
            known_auditor("The auditor is Grant Thornton LLP.").as_deref(),
            Some("Grant Thornton LLP")
        );
        assert_eq!(
            known_auditor("Independent registered public accounting firm: EY LLP").as_deref(),
            Some("EY LLP")
        );
    }

    #[test]
    fn detects_broader_underwriter_set() {
        let values = known_underwriters("HSBC, Nomura and Piper Sandler are underwriters.");
        assert!(values.contains(&"HSBC".to_string()));
        assert!(values.contains(&"Nomura".to_string()));
        assert!(values.contains(&"Piper Sandler".to_string()));
    }
}
