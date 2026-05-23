use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParserKind {
    CompanyReport,
    Form4,
    EightK,
    Schedule13,
    Proxy,
    Prospectus,
    ForeignIssuer,
    FundDisclosure,
    ThirteenF,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ParserSpec {
    pub kind: ParserKind,
    pub canonical_form: &'static str,
    pub forms: &'static [&'static str],
    pub record_kind: &'static str,
}

const PARSERS: &[ParserSpec] = &[
    ParserSpec {
        kind: ParserKind::CompanyReport,
        canonical_form: "10-K",
        forms: &["10-K", "10-K/A", "10-Q", "10-Q/A"],
        record_kind: "company_report",
    },
    ParserSpec {
        kind: ParserKind::Form4,
        canonical_form: "4",
        forms: &["3", "3/A", "4", "4/A", "5", "5/A"],
        record_kind: "form4_transaction",
    },
    ParserSpec {
        kind: ParserKind::EightK,
        canonical_form: "8-K",
        forms: &["8-K", "8-K/A"],
        record_kind: "eightk_event",
    },
    ParserSpec {
        kind: ParserKind::Schedule13,
        canonical_form: "SC 13D",
        forms: &["SC 13D", "SC 13D/A", "SC 13G", "SC 13G/A"],
        record_kind: "schedule13",
    },
    ParserSpec {
        kind: ParserKind::Proxy,
        canonical_form: "DEF 14A",
        forms: &["DEF 14A", "DEF 14A/A"],
        record_kind: "proxy_statement",
    },
    ParserSpec {
        kind: ParserKind::Prospectus,
        canonical_form: "S-1",
        forms: &[
            "S-1", "S-1/A", "F-1", "F-1/A", "424B", "424B1", "424B2", "424B3", "424B4", "424B5",
            "424B7",
        ],
        record_kind: "prospectus",
    },
    ParserSpec {
        kind: ParserKind::ForeignIssuer,
        canonical_form: "20-F",
        forms: &["20-F", "20-F/A", "6-K", "6-K/A", "40-F", "40-F/A"],
        record_kind: "foreign_issuer",
    },
    ParserSpec {
        kind: ParserKind::FundDisclosure,
        canonical_form: "NPORT-P",
        forms: &[
            "NPORT-P",
            "NPORT-P/A",
            "N-PORT",
            "N-PORT/A",
            "N-CSR",
            "N-CSR/A",
            "N-CSRS",
            "N-CSRS/A",
            "N-CEN",
            "N-CEN/A",
            "N-PX",
            "N-PX/A",
            "497K",
            "497K/A",
            "24F-2NT",
            "24F-2NT/A",
        ],
        record_kind: "fund_disclosure",
    },
    ParserSpec {
        kind: ParserKind::ThirteenF,
        canonical_form: "13F-HR",
        forms: &["13F-HR", "13F-HR/A", "13F-NT", "13F-NT/A"],
        record_kind: "thirteenf_holding",
    },
];

pub fn parser_for_form(form: &str) -> Option<&'static ParserSpec> {
    let normalized = normalize_form(form);
    PARSERS
        .iter()
        .find(|spec| spec.forms.iter().any(|value| *value == normalized))
}

pub fn supported_parsers() -> &'static [ParserSpec] {
    PARSERS
}

pub fn normalize_form(form: &str) -> String {
    form.trim().to_ascii_uppercase()
}
