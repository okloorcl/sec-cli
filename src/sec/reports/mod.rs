use anyhow::Result;

use crate::sec::{
    client::SecClient,
    models::{
        FinancialMetricRecord, Form4Query, MetricsQuery, ReportQuery, SectionQuery, ThirteenFQuery,
    },
    parsers::forms::thirteenf::{aggregate, diff, holdings, summary},
};

mod format;

use format::{
    bar, cell, compact_float, dollars, first_source_link, metric_display, opt, push_header,
    push_section_excerpt, signed_dollars, signed_number,
};

#[derive(Debug, Clone, Copy)]
pub enum ReportKind {
    Financial,
    Insider,
    Portfolio,
    Risk,
}

impl SecClient {
    pub async fn markdown_report(&self, kind: ReportKind, query: ReportQuery) -> Result<String> {
        match kind {
            ReportKind::Financial => self.financial_report(query).await,
            ReportKind::Insider => self.insider_report(query).await,
            ReportKind::Portfolio => self.portfolio_report(query).await,
            ReportKind::Risk => self.risk_report(query).await,
        }
    }

    async fn financial_report(&self, query: ReportQuery) -> Result<String> {
        let metrics = self
            .financial_metrics(MetricsQuery {
                cik: query.cik,
                form: Some("10-K".to_string()),
                unit: None,
                latest: query.latest.max(2),
            })
            .await?;

        let mut out = String::new();
        push_header(
            &mut out,
            "Financial Trend Report",
            &query.subject,
            query.cik,
        );
        out.push_str("## Latest metrics\n\n");
        out.push_str("| Metric | Category | Value | Period | Calculation | Source |\n");
        out.push_str("| --- | --- | ---: | --- | --- | --- |\n");
        for metric in latest_metrics(&metrics).into_iter().take(query.limit) {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                metric.metric,
                metric.category,
                metric_display(metric),
                opt(metric.period_end.as_deref()),
                cell(&metric.calculation),
                first_source_link(metric)
            ));
        }

        out.push_str("\n## Trend snapshot\n\n");
        out.push_str("| Period | Revenue growth | Net margin | FCF margin | ROA | ROE | Current ratio | Liabilities/assets |\n");
        out.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
        for period in metric_periods(&metrics).into_iter().take(query.latest) {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                period,
                metric_cell(&metrics, &period, "revenue_growth"),
                metric_cell(&metrics, &period, "net_margin"),
                metric_cell(&metrics, &period, "free_cash_flow_margin"),
                metric_cell(&metrics, &period, "return_on_assets"),
                metric_cell(&metrics, &period, "return_on_equity"),
                metric_cell(&metrics, &period, "current_ratio"),
                metric_cell(&metrics, &period, "liabilities_to_assets")
            ));
        }
        push_financial_signals(&mut out, &metrics);
        out.push_str("\nUse `sec metrics` for stable JSON with component facts and source URLs.\n");
        Ok(out)
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
        let portfolio = self.portfolio_report_data(&query).await?;
        let mut holdings = portfolio.current_holdings;
        let mut changes = portfolio.changes;
        changes.truncate(query.limit);
        holdings.truncate(query.limit);

        let mut out = String::new();
        push_header(&mut out, "13F Portfolio Report", &query.subject, query.cik);
        if let Some(summary) = portfolio.summary {
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

    async fn portfolio_report_data(&self, query: &ReportQuery) -> Result<PortfolioReportData> {
        let filings = self
            .thirteenf_filings(&ThirteenFQuery {
                cik: query.cik,
                latest: 2,
                include_amends: query.include_amends,
            })
            .await?;
        let mut parsed = self
            .filing_documents_batch(filings)
            .await?
            .into_iter()
            .map(|(filing, docs)| {
                let summary = summary::parse_13f_report_documents(&filing, &docs)?
                    .into_iter()
                    .next();
                let aggregate =
                    aggregate::aggregate_holdings(holdings::parse_13f_documents(&filing, &docs)?);
                Ok::<_, anyhow::Error>((summary, aggregate))
            })
            .collect::<Result<Vec<_>>>()?;

        if parsed.is_empty() {
            return Ok(PortfolioReportData::default());
        }
        let (summary, current_holdings) = parsed.remove(0);
        let previous_holdings = parsed
            .into_iter()
            .next()
            .map(|(_, holdings)| holdings)
            .unwrap_or_default();
        let changes = diff::diff_holdings(current_holdings.clone(), previous_holdings);
        Ok(PortfolioReportData {
            summary,
            current_holdings,
            changes,
        })
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

#[derive(Default)]
struct PortfolioReportData {
    summary: Option<crate::sec::models::ThirteenFReportRecord>,
    current_holdings: Vec<crate::sec::models::ThirteenFAggregateHoldingRecord>,
    changes: Vec<crate::sec::models::ThirteenFDiffRecord>,
}

fn latest_metrics(metrics: &[FinancialMetricRecord]) -> Vec<&FinancialMetricRecord> {
    let latest_period = metrics
        .iter()
        .filter_map(|metric| metric.period_end.as_deref())
        .max();
    metrics
        .iter()
        .filter(|metric| metric.period_end.as_deref() == latest_period)
        .collect()
}

fn metric_periods(metrics: &[FinancialMetricRecord]) -> Vec<String> {
    let mut periods = metrics
        .iter()
        .filter_map(|metric| metric.period_end.clone())
        .collect::<Vec<_>>();
    periods.sort();
    periods.dedup();
    periods.reverse();
    periods
}

fn metric_cell(metrics: &[FinancialMetricRecord], period: &str, metric_name: &str) -> String {
    metrics
        .iter()
        .find(|metric| metric.metric == metric_name && metric.period_end.as_deref() == Some(period))
        .map(metric_display)
        .unwrap_or_else(|| "-".to_string())
}

fn push_financial_signals(out: &mut String, metrics: &[FinancialMetricRecord]) {
    out.push_str("\n## Rule-based signals\n\n");
    let periods = metric_periods(metrics);
    let Some(latest_period) = periods.first() else {
        out.push_str("- No comparable metric periods were available.\n");
        return;
    };
    let previous_period = periods.get(1).map(String::as_str);
    let mut signals = Vec::new();

    push_change_signal(
        &mut signals,
        metrics,
        latest_period,
        previous_period,
        "revenue_growth",
        "Revenue growth",
        0.03,
    );
    push_change_signal(
        &mut signals,
        metrics,
        latest_period,
        previous_period,
        "net_margin",
        "Net margin",
        0.02,
    );
    push_change_signal(
        &mut signals,
        metrics,
        latest_period,
        previous_period,
        "free_cash_flow_margin",
        "Free cash flow margin",
        0.02,
    );

    if metric_value(metrics, latest_period, "current_ratio").is_some_and(|value| value < 1.0) {
        signals.push(format!(
            "Current ratio is below 1.0x at {}.",
            metric_cell(metrics, latest_period, "current_ratio")
        ));
    }
    if metric_value(metrics, latest_period, "liabilities_to_assets")
        .is_some_and(|value| value > 0.7)
    {
        signals.push(format!(
            "Liabilities/assets is elevated at {}.",
            metric_cell(metrics, latest_period, "liabilities_to_assets")
        ));
    }

    if signals.is_empty() {
        out.push_str("- No threshold-based signals fired for the latest period.\n");
    } else {
        for signal in signals {
            out.push_str(&format!("- {signal}\n"));
        }
    }
}

fn push_change_signal(
    signals: &mut Vec<String>,
    metrics: &[FinancialMetricRecord],
    latest_period: &str,
    previous_period: Option<&str>,
    metric_name: &str,
    label: &str,
    threshold: f64,
) {
    let Some(previous_period) = previous_period else {
        return;
    };
    let Some(current) = metric_value(metrics, latest_period, metric_name) else {
        return;
    };
    let Some(previous) = metric_value(metrics, previous_period, metric_name) else {
        return;
    };
    let delta = current - previous;
    if delta.abs() < threshold {
        return;
    }
    let direction = if delta > 0.0 { "improved" } else { "weakened" };
    signals.push(format!(
        "{label} {direction}: {} vs {} in the prior period.",
        metric_cell(metrics, latest_period, metric_name),
        metric_cell(metrics, previous_period, metric_name)
    ));
}

fn metric_value(metrics: &[FinancialMetricRecord], period: &str, metric_name: &str) -> Option<f64> {
    metrics
        .iter()
        .find(|metric| metric.metric == metric_name && metric.period_end.as_deref() == Some(period))
        .and_then(|metric| metric.value)
}

#[cfg(test)]
mod tests {
    #[test]
    fn portfolio_report_data_defaults_to_empty() {
        let data = super::PortfolioReportData::default();
        assert!(data.summary.is_none());
        assert!(data.current_holdings.is_empty());
        assert!(data.changes.is_empty());
    }
}
