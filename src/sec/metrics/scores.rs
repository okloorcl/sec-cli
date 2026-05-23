use anyhow::Result;

use crate::sec::{
    client::SecClient,
    models::{
        FinancialMetricRecord, FinancialStatementRecord, HealthScoreQuery, HealthScoreRecord,
        HealthScoreSignalRecord, MetricsQuery, StatementQuery,
    },
};

use super::score_support::*;

impl SecClient {
    pub async fn health_scores(&self, query: HealthScoreQuery) -> Result<Vec<HealthScoreRecord>> {
        let latest = query.latest.saturating_add(1).max(2);
        let statements = self
            .financial_statements(StatementQuery {
                cik: query.cik,
                statement: "all".to_string(),
                form: query.form.clone(),
                unit: query.unit.clone(),
                latest,
            })
            .await?;
        let metrics = self
            .financial_metrics(MetricsQuery {
                cik: query.cik,
                form: query.form,
                unit: query.unit,
                latest,
            })
            .await?;

        Ok(build_health_scores(
            query.cik,
            query.latest,
            &metrics,
            &statements,
        ))
    }
}

fn build_health_scores(
    cik: u64,
    latest: usize,
    metrics: &[FinancialMetricRecord],
    statements: &[FinancialStatementRecord],
) -> Vec<HealthScoreRecord> {
    let periods = metric_periods(metrics);
    let statement_maps = statement_period_maps(statements);
    let mut records = Vec::new();

    for period in periods.iter().take(latest) {
        let current = PeriodView {
            period,
            metrics,
            statements: statement_maps.get(period),
        };
        let previous = previous_period(period, &periods).map(|previous| PeriodView {
            period: previous,
            metrics,
            statements: statement_maps.get(previous),
        });
        records.push(piotroski_score(cik, current, previous));
        records.push(altman_score(cik, current));
        records.push(beneish_score(cik, current, previous));
    }

    records
}

fn piotroski_score(
    cik: u64,
    current: PeriodView<'_>,
    previous: Option<PeriodView<'_>>,
) -> HealthScoreRecord {
    let signals = vec![
        pass_gt(
            current.statement("income:net_income"),
            0.0,
            "positive_net_income",
        ),
        pass_gt(
            current.statement("cashflow:operating_cash_flow"),
            0.0,
            "positive_operating_cash_flow",
        ),
        pass_improved_metric(current, previous, "return_on_assets", "improving_roa"),
        pass_gt_pair(
            current.statement("cashflow:operating_cash_flow"),
            current.statement("income:net_income"),
            "accrual_quality",
        ),
        pass_decreased_metric(current, previous, "liabilities_to_assets", "lower_leverage"),
        pass_improved_metric(current, previous, "current_ratio", "higher_current_ratio"),
        pass_not_increased_statement(
            current,
            previous,
            "income:shares_diluted",
            "no_share_dilution",
        ),
        pass_improved_metric(current, previous, "gross_margin", "higher_gross_margin"),
        pass_improved_metric(current, previous, "asset_turnover", "higher_asset_turnover"),
    ];
    scored_record(
        cik,
        current,
        "piotroski_f_score",
        "Piotroski F-Score: nine binary profitability, leverage/liquidity, and operating-efficiency signals",
        signals,
        Some(9.0),
        piotroski_rating,
    )
}

fn altman_score(cik: u64, current: PeriodView<'_>) -> HealthScoreRecord {
    let wc_ta = current.metric("working_capital_to_assets");
    let re_ta = ratio(
        current.statement("balance:retained_earnings"),
        current.statement("balance:total_assets"),
    );
    let ebit_ta = ratio(
        current.statement("income:operating_income"),
        current.statement("balance:total_assets"),
    );
    let equity_liabilities = ratio(
        current.statement("balance:stockholders_equity"),
        current.statement("balance:total_liabilities"),
    );
    let score = zip4(wc_ta, re_ta, ebit_ta, equity_liabilities)
        .map(|(a, b, c, d)| 6.56 * a + 3.26 * b + 6.72 * c + 1.05 * d);
    let signals = vec![
        value_signal(
            "working_capital_to_assets",
            wc_ta,
            "working_capital / total_assets",
        ),
        value_signal(
            "retained_earnings_to_assets",
            re_ta,
            "retained_earnings / total_assets",
        ),
        value_signal("ebit_to_assets", ebit_ta, "operating_income / total_assets"),
        value_signal(
            "equity_to_liabilities",
            equity_liabilities,
            "stockholders_equity / total_liabilities",
        ),
    ];
    value_record(
        cik,
        current,
        "altman_z_score_private",
        score,
        "Altman Z''-Score approximation: 6.56*WC/TA + 3.26*RE/TA + 6.72*EBIT/TA + 1.05*Equity/Liabilities",
        signals,
        altman_rating,
    )
}

fn beneish_score(
    cik: u64,
    current: PeriodView<'_>,
    previous: Option<PeriodView<'_>>,
) -> HealthScoreRecord {
    let previous = previous;
    let dsri = current
        .metric("receivables_to_revenue")
        .zip(previous.and_then(|p| p.metric("receivables_to_revenue")))
        .and_then(|(current, previous)| safe_div(current, previous));
    let gmi = previous
        .and_then(|p| p.metric("gross_margin"))
        .zip(current.metric("gross_margin"))
        .and_then(|(previous, current)| safe_div(previous, current));
    let aqi = asset_quality(current)
        .zip(previous.and_then(asset_quality))
        .and_then(|(current, previous)| safe_div(current, previous));
    let sgi = current
        .statement("income:revenue")
        .zip(previous.and_then(|p| p.statement("income:revenue")))
        .and_then(|(current, previous)| safe_div(current, previous));
    let depi = depreciation_rate(previous)
        .zip(depreciation_rate(Some(current)))
        .and_then(|(previous, current)| safe_div(previous, current));
    let sgai = current
        .metric("sga_to_revenue")
        .zip(previous.and_then(|p| p.metric("sga_to_revenue")))
        .and_then(|(current, previous)| safe_div(current, previous));
    let lvgi = current
        .metric("liabilities_to_assets")
        .zip(previous.and_then(|p| p.metric("liabilities_to_assets")))
        .and_then(|(current, previous)| safe_div(current, previous));
    let tata = current
        .statement("income:net_income")
        .zip(current.statement("cashflow:operating_cash_flow"))
        .zip(current.statement("balance:total_assets"))
        .and_then(|((net_income, ocf), assets)| safe_div(net_income - ocf, assets));
    let score = zip8(dsri, gmi, aqi, sgi, depi, sgai, lvgi, tata).map(
        |(dsri, gmi, aqi, sgi, depi, sgai, lvgi, tata)| {
            -4.84 + 0.92 * dsri + 0.528 * gmi + 0.404 * aqi + 0.892 * sgi + 0.115 * depi
                - 0.172 * sgai
                + 4.679 * tata
                - 0.327 * lvgi
        },
    );
    let signals = vec![
        value_signal("dsri", dsri, "receivables_to_revenue current / previous"),
        value_signal("gmi", gmi, "gross_margin previous / current"),
        value_signal("aqi", aqi, "asset_quality current / previous"),
        value_signal("sgi", sgi, "revenue current / previous"),
        value_signal("depi", depi, "depreciation_rate previous / current"),
        value_signal("sgai", sgai, "sga_to_revenue current / previous"),
        value_signal("lvgi", lvgi, "liabilities_to_assets current / previous"),
        value_signal(
            "tata",
            tata,
            "(net_income - operating_cash_flow) / total_assets",
        ),
    ];
    value_record(
        cik,
        current,
        "beneish_m_score",
        score,
        "Beneish M-Score approximation over SEC-derived ratios; values above -1.78 are manipulation-risk watch signals",
        signals,
        beneish_rating,
    )
}

fn scored_record(
    cik: u64,
    current: PeriodView<'_>,
    score_name: &str,
    calculation: &str,
    signals: Vec<HealthScoreSignalRecord>,
    max_score: Option<f64>,
    rating: fn(Option<f64>) -> String,
) -> HealthScoreRecord {
    let score = Some(signals.iter().map(|signal| signal.points).sum());
    HealthScoreRecord {
        cik,
        company: current.company(),
        score_name: score_name.to_string(),
        score,
        max_score,
        rating: rating(score),
        fiscal_year: current.fiscal_year(),
        fiscal_period: current.fiscal_period(),
        form: current.form(),
        period_end: Some(current.period.to_string()),
        calculation: calculation.to_string(),
        source_urls: source_urls(current, &signals),
        signals,
    }
}

fn value_record(
    cik: u64,
    current: PeriodView<'_>,
    score_name: &str,
    score: Option<f64>,
    calculation: &str,
    signals: Vec<HealthScoreSignalRecord>,
    rating: fn(Option<f64>) -> String,
) -> HealthScoreRecord {
    HealthScoreRecord {
        cik,
        company: current.company(),
        score_name: score_name.to_string(),
        score,
        max_score: None,
        rating: rating(score),
        fiscal_year: current.fiscal_year(),
        fiscal_period: current.fiscal_period(),
        form: current.form(),
        period_end: Some(current.period.to_string()),
        calculation: calculation.to_string(),
        source_urls: source_urls(current, &signals),
        signals,
    }
}
