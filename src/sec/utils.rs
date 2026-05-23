pub(crate) const LEGAL_SUFFIXES: &[&str] = &[
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

pub(crate) fn truncate_utf8(content: &str, limit_bytes: Option<usize>) -> (String, bool) {
    let Some(limit) = limit_bytes else {
        return (content.to_string(), false);
    };
    if content.len() <= limit {
        return (content.to_string(), false);
    }
    if limit == 0 {
        return (String::new(), true);
    }

    let mut end = limit.min(content.len());
    while !content.is_char_boundary(end) {
        end -= 1;
    }
    (content[..end].to_string(), true)
}

pub(crate) fn nonempty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

pub(crate) fn is_legal_suffix(value: &str) -> bool {
    LEGAL_SUFFIXES.contains(&value)
}
