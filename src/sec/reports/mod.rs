use anyhow::Result;

use crate::sec::{
    client::SecClient,
    models::{Form4Query, ReportQuery, SectionQuery, ThirteenFQuery},
};

#[derive(Debug, Clone, Copy)]
pub enum ReportKind {
    Insider,
    Portfolio,
    Risk,
}

impl SecClient {
    pub async fn markdown_report(&self, kind: ReportKind, query: ReportQuery) -> Result<String> {
        match kind {
            ReportKind::Insider => self.insider_report(query).await,
            ReportKind::Portfolio => self.portfolio_report(query).await,
            ReportKind::Risk => self.risk_report(query).await,
        }
    }

    async fn insider_report(&self, query: ReportQuery) -> Result<String> {
        let mut reports = self
            .form4_reports(Form4Query {
                cik: query.cik,
                latest: query.latest,
                include_amends: query.include_amends,
            })
            .await?;
        reports.truncate(query.limit);

        let mut out = String::new();
        push_header(
            &mut out,
            "Insider Activity Report",
            &query.subject,
            query.cik,
        );
        out.push_str("## Form 4 report summaries\n\n");
        out.push_str("| Filing date | Period | Owner | Role | Net shares | Value | Source |\n");
        out.push_str("| --- | --- | --- | --- | ---: | ---: | --- |\n");
        for report in reports {
            let owner = report
                .owners
                .first()
                .and_then(|owner| owner.owner_name.clone())
                .unwrap_or_else(|| "-".to_string());
            let role = report
                .owners
                .first()
                .and_then(|owner| owner.officer_title.clone())
                .unwrap_or_else(|| "-".to_string());
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | [SEC]({}) |\n",
                report.filing_date,
                opt(report.period_of_report.as_deref()),
                cell(&owner),
                cell(&role),
                signed_number(report.net_shares),
                dollars(report.total_value as u64),
                report.source_url
            ));
        }
        out.push_str("\nUse `sec form4` for row-level transactions and `sec form4-summary` for stable JSON.\n");
        Ok(out)
    }

    async fn portfolio_report(&self, query: ReportQuery) -> Result<String> {
        let mut summaries = self
            .thirteenf_reports(ThirteenFQuery {
                cik: query.cik,
                latest: 1,
                include_amends: query.include_amends,
            })
            .await?;
        let mut holdings = self
            .thirteenf_aggregate_holdings(ThirteenFQuery {
                cik: query.cik,
                latest: 1,
                include_amends: query.include_amends,
            })
            .await?;
        let mut changes = self
            .thirteenf_diff_holdings(ThirteenFQuery {
                cik: query.cik,
                latest: 2,
                include_amends: query.include_amends,
            })
            .await?;
        holdings.truncate(query.limit);
        changes.truncate(query.limit);

        let mut out = String::new();
        push_header(&mut out, "13F Portfolio Report", &query.subject, query.cik);
        if let Some(summary) = summaries.pop() {
            out.push_str("## Latest 13F summary\n\n");
            out.push_str(&format!(
                "- Report date: {}\n- Report type: {}\n- Total holdings: {}\n- Total value: {}\n- Source: [SEC]({})\n\n",
                opt(summary.report_date.as_deref()),
                opt(summary.report_type.as_deref()),
                summary
                    .total_holdings_reported
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                summary
                    .total_value_usd
                    .map(dollars)
                    .unwrap_or_else(|| "-".to_string()),
                summary.source_url
            ));
        }

        out.push_str("## Top holdings\n\n");
        out.push_str("| Holding | CUSIP | Value | Shares | Visual |\n");
        out.push_str("| --- | --- | ---: | ---: | --- |\n");
        let max_value = holdings.first().map(|h| h.value_usd).unwrap_or(1);
        for holding in &holdings {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                cell(holding.issuer.as_deref().unwrap_or("-")),
                opt(holding.cusip.as_deref()),
                dollars(holding.value_usd),
                compact_float(holding.shares),
                bar(holding.value_usd, max_value)
            ));
        }

        out.push_str("\n## Largest position changes\n\n");
        out.push_str("| Change | Holding | CUSIP | Shares delta | Value delta | Source |\n");
        out.push_str("| --- | --- | --- | ---: | ---: | --- |\n");
        for change in changes {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | [SEC]({}) |\n",
                change.change_type,
                cell(change.issuer.as_deref().unwrap_or("-")),
                opt(change.cusip.as_deref()),
                signed_number(change.change_shares),
                signed_dollars(change.change_value_usd),
                change.current_source_url
            ));
        }
        Ok(out)
    }

    async fn risk_report(&self, query: ReportQuery) -> Result<String> {
        let risk = self
            .sections(SectionQuery {
                cik: query.cik,
                form: Some("10-K".to_string()),
                latest: 1,
                include_amends: query.include_amends,
                accession: None,
                item: "risk-factors".to_string(),
                limit_bytes: Some(query.limit_bytes),
            })
            .await?;
        let mda = self
            .sections(SectionQuery {
                cik: query.cik,
                form: Some("10-K".to_string()),
                latest: 1,
                include_amends: query.include_amends,
                accession: None,
                item: "mda".to_string(),
                limit_bytes: Some(query.limit_bytes / 2),
            })
            .await?;

        let mut out = String::new();
        push_header(
            &mut out,
            "10-K Risk and MD&A Report",
            &query.subject,
            query.cik,
        );
        push_section_excerpt(&mut out, "Risk Factors", risk.first());
        push_section_excerpt(&mut out, "MD&A", mda.first());
        out.push_str("Use `sec section --item risk-factors` or `sec section --item mda` for stable JSON excerpts.\n");
        Ok(out)
    }
}

fn push_header(out: &mut String, title: &str, subject: &str, cik: u64) {
    out.push_str(&format!("# {}\n\n", title));
    out.push_str(&format!("- Subject: {}\n- CIK: {}\n\n", subject, cik));
}

fn push_section_excerpt(
    out: &mut String,
    title: &str,
    section: Option<&crate::sec::models::SectionRecord>,
) {
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

fn opt(value: Option<&str>) -> &str {
    value.filter(|v| !v.is_empty()).unwrap_or("-")
}

fn cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn dollars(value: u64) -> String {
    format!("${}", grouped(value))
}

fn signed_dollars(value: i128) -> String {
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

fn compact_float(value: f64) -> String {
    let sign = if value < 0.0 { "-" } else { "" };
    let abs = value.abs();
    if abs.fract().abs() < f64::EPSILON {
        format!("{}{}", sign, grouped(abs as u64))
    } else {
        format!("{value:.2}")
    }
}

fn signed_number(value: f64) -> String {
    if value > 0.0 {
        format!("+{}", compact_float(value))
    } else {
        compact_float(value)
    }
}

fn bar(value: u64, max_value: u64) -> String {
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
