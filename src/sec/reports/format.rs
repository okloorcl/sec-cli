use crate::sec::models::{FinancialMetricRecord, SectionRecord};

pub(super) fn push_header(out: &mut String, title: &str, subject: &str, cik: u64) {
    out.push_str(&format!("# {}\n\n", title));
    out.push_str(&format!("- Subject: {}\n- CIK: {}\n\n", subject, cik));
}

pub(super) fn push_section_excerpt(out: &mut String, title: &str, section: Option<&SectionRecord>) {
    out.push_str(&format!("## {}\n\n", title));
    if let Some(section) = section {
        out.push_str(&format!(
            "- Filing: {} {}\n- Source: [SEC]({})\n- Returned bytes: {} / {}\n\n",
            section.form,
            section.filing_date,
            section.source_url,
            section.returned_bytes,
            section.byte_length
        ));
        out.push_str("> ");
        out.push_str(&section.content.replace('\n', " "));
        out.push_str("\n\n");
    } else {
        out.push_str("No section was extracted for the selected filing.\n\n");
    }
}

pub(super) fn metric_display(metric: &FinancialMetricRecord) -> String {
    match (metric.unit.as_str(), metric.value) {
        ("USD", Some(value)) => dollars_f64(value),
        _ => metric.display_value.clone().unwrap_or_else(|| {
            metric
                .value
                .map_or_else(|| "-".to_string(), |value| format!("{value:.4}"))
        }),
    }
}

pub(super) fn first_source_link(metric: &FinancialMetricRecord) -> String {
    metric
        .source_urls
        .first()
        .map(|url| format!("[SEC]({url})"))
        .unwrap_or_else(|| "-".to_string())
}

pub(super) fn opt(value: Option<&str>) -> &str {
    value.filter(|v| !v.is_empty()).unwrap_or("-")
}

pub(super) fn cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

pub(super) fn dollars(value: u64) -> String {
    format!("${}", grouped(value))
}

fn dollars_f64(value: f64) -> String {
    if value < 0.0 {
        format!("-${}", grouped(value.abs().round() as u64))
    } else {
        dollars(value.round() as u64)
    }
}

pub(super) fn signed_dollars(value: i128) -> String {
    if value < 0 {
        format!("-${}", grouped(value.unsigned_abs() as u64))
    } else {
        format!("+${}", grouped(value as u64))
    }
}

fn grouped(value: u64) -> String {
    let raw = value.to_string();
    let first_group = raw.len() % 3;
    let mut out = String::with_capacity(raw.len() + raw.len() / 3);
    for (idx, ch) in raw.chars().enumerate() {
        if idx > 0 && (idx + 3 - first_group) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

pub(super) fn compact_float(value: f64) -> String {
    let sign = if value < 0.0 { "-" } else { "" };
    let abs = value.abs();
    if abs.fract().abs() < f64::EPSILON {
        format!("{}{}", sign, grouped(abs as u64))
    } else {
        format!("{value:.2}")
    }
}

pub(super) fn signed_number(value: f64) -> String {
    if value > 0.0 {
        format!("+{}", compact_float(value))
    } else {
        compact_float(value)
    }
}

pub(super) fn bar(value: u64, max_value: u64) -> String {
    let width = if max_value == 0 {
        0
    } else {
        ((value as f64 / max_value as f64) * 12.0).round() as usize
    };
    format!(
        "{}{}",
        "#".repeat(width),
        ".".repeat(12usize.saturating_sub(width))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_grouped_numbers_and_dollars() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(123), "123");
        assert_eq!(grouped(1234), "1,234");
        assert_eq!(dollars(1_234_567), "$1,234,567");
        assert_eq!(signed_dollars(-1_200), "-$1,200");
        assert_eq!(signed_dollars(1_200), "+$1,200");
    }

    #[test]
    fn escapes_markdown_cells_and_builds_bar() {
        assert_eq!(cell("A|B\nC"), "A\\|B C");
        assert_eq!(bar(50, 100), "######......");
        assert_eq!(bar(1, 0), "............");
    }
}
