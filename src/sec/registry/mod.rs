use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParserKind {
    Form4,
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
        kind: ParserKind::Form4,
        canonical_form: "4",
        forms: &["3", "3/A", "4", "4/A", "5", "5/A"],
        record_kind: "form4_transaction",
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
