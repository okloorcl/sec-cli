use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::sec::models::{
    FinancialMetricRecord, FinancialStatementRecord, HealthScoreSignalRecord,
};

#[derive(Clone, Copy)]
pub(super) struct PeriodView<'a> {
    pub(super) period: &'a str,
    pub(super) metrics: &'a [FinancialMetricRecord],
    pub(super) statements: Option<&'a HashMap<String, &'a FinancialStatementRecord>>,
}

pub(super) fn pass_gt(value: Option<f64>, threshold: f64, name: &str) -> HealthScoreSignalRecord {
    binary_signal(
        name,
        value,
        value.map(|value| value > threshold),
        &format!("> {threshold}"),
    )
}

pub(super) fn pass_gt_pair(
    left: Option<f64>,
    right: Option<f64>,
    name: &str,
) -> HealthScoreSignalRecord {
    binary_signal(
        name,
        left.zip(right).map(|(a, b)| a - b),
        left.zip(right).map(|(a, b)| a > b),
        "left > right",
    )
}

pub(super) fn pass_improved_metric(
    current: PeriodView<'_>,
    previous: Option<PeriodView<'_>>,
    metric: &str,
    name: &str,
) -> HealthScoreSignalRecord {
    let value = current.metric(metric);
    let previous = previous.and_then(|period| period.metric(metric));
    binary_signal(
        name,
        value.zip(previous).map(|(a, b)| a - b),
        value.zip(previous).map(|(a, b)| a > b),
        "current > previous",
    )
}

pub(super) fn pass_decreased_metric(
    current: PeriodView<'_>,
    previous: Option<PeriodView<'_>>,
    metric: &str,
    name: &str,
) -> HealthScoreSignalRecord {
    let value = current.metric(metric);
    let previous = previous.and_then(|period| period.metric(metric));
    binary_signal(
        name,
        value.zip(previous).map(|(a, b)| a - b),
        value.zip(previous).map(|(a, b)| a < b),
        "current < previous",
    )
}

pub(super) fn pass_not_increased_statement(
    current: PeriodView<'_>,
    previous: Option<PeriodView<'_>>,
    key: &str,
    name: &str,
) -> HealthScoreSignalRecord {
    let value = current.statement(key);
    let previous = previous.and_then(|period| period.statement(key));
    binary_signal(
        name,
        value.zip(previous).map(|(a, b)| a - b),
        value.zip(previous).map(|(a, b)| a <= b),
        "current <= previous",
    )
}

pub(super) fn value_signal(
    name: &str,
    value: Option<f64>,
    calculation: &str,
) -> HealthScoreSignalRecord {
    HealthScoreSignalRecord {
        name: name.to_string(),
        passed: None,
        points: 0.0,
        max_points: 0.0,
        value,
        threshold: "reported for formula input".to_string(),
        calculation: calculation.to_string(),
        source_urls: Vec::new(),
    }
}

fn binary_signal(
    name: &str,
    value: Option<f64>,
    passed: Option<bool>,
    threshold: &str,
) -> HealthScoreSignalRecord {
    HealthScoreSignalRecord {
        name: name.to_string(),
        passed,
        points: f64::from(passed.unwrap_or(false)),
        max_points: 1.0,
        value,
        threshold: threshold.to_string(),
        calculation: name.to_string(),
        source_urls: Vec::new(),
    }
}

pub(super) fn metric_periods(metrics: &[FinancialMetricRecord]) -> Vec<String> {
    let mut periods = metrics
        .iter()
        .filter_map(|metric| metric.period_end.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    periods.sort_by(|a, b| b.cmp(a));
    periods
}

pub(super) fn previous_period<'a>(period: &str, periods: &'a [String]) -> Option<&'a str> {
    periods
        .iter()
        .position(|candidate| candidate == period)
        .and_then(|index| periods.get(index + 1))
        .map(String::as_str)
}

pub(super) fn statement_period_maps(
    statements: &[FinancialStatementRecord],
) -> BTreeMap<String, HashMap<String, &FinancialStatementRecord>> {
    let mut periods = BTreeMap::new();
    for row in statements {
        let Some(end) = row.end.clone().or_else(|| row.filed.clone()) else {
            continue;
        };
        periods
            .entry(end)
            .or_insert_with(HashMap::new)
            .entry(format!("{}:{}", row.statement, row.line_item))
            .or_insert(row);
    }
    periods
}

impl<'a> PeriodView<'a> {
    pub(super) fn metric(&self, name: &str) -> Option<f64> {
        self.metrics
            .iter()
            .find(|metric| {
                metric.metric == name && metric.period_end.as_deref() == Some(self.period)
            })
            .and_then(|metric| metric.value)
    }

    pub(super) fn metric_record(&self) -> Option<&'a FinancialMetricRecord> {
        self.metrics
            .iter()
            .find(|metric| metric.period_end.as_deref() == Some(self.period))
    }

    pub(super) fn statement(&self, key: &str) -> Option<f64> {
        self.statements
            .and_then(|rows| rows.get(key))
            .and_then(|row| row.numeric_value)
    }

    pub(super) fn company(&self) -> Option<String> {
        self.metric_record()
            .and_then(|metric| metric.company.clone())
            .or_else(|| {
                self.statements
                    .and_then(|rows| rows.values().next())
                    .and_then(|row| row.company.clone())
            })
    }

    pub(super) fn fiscal_year(&self) -> Option<i64> {
        self.metric_record().and_then(|metric| metric.fiscal_year)
    }

    pub(super) fn fiscal_period(&self) -> Option<String> {
        self.metric_record()
            .and_then(|metric| metric.fiscal_period.clone())
    }

    pub(super) fn form(&self) -> Option<String> {
        self.metric_record().and_then(|metric| metric.form.clone())
    }

    pub(super) fn source_urls(&self) -> Vec<String> {
        let mut urls = self
            .metrics
            .iter()
            .filter(|metric| metric.period_end.as_deref() == Some(self.period))
            .flat_map(|metric| metric.source_urls.iter().cloned())
            .collect::<BTreeSet<_>>();
        if let Some(statements) = self.statements {
            urls.extend(statements.values().filter_map(|row| row.source_url.clone()));
        }
        urls.into_iter().collect()
    }
}

pub(super) fn asset_quality(period: PeriodView<'_>) -> Option<f64> {
    let current_assets = period.statement("balance:current_assets")?;
    let ppe = period.statement("balance:property_plant_equipment")?;
    let assets = period.statement("balance:total_assets")?;
    safe_div(assets - current_assets - ppe, assets)
}

pub(super) fn depreciation_rate(period: Option<PeriodView<'_>>) -> Option<f64> {
    let period = period?;
    let depreciation = period
        .statement("cashflow:depreciation_amortization")?
        .abs();
    let ppe = period.statement("balance:property_plant_equipment")?.abs();
    safe_div(depreciation, depreciation + ppe)
}

pub(super) fn ratio(numerator: Option<f64>, denominator: Option<f64>) -> Option<f64> {
    numerator.zip(denominator).and_then(|(a, b)| safe_div(a, b))
}

pub(super) fn safe_div(numerator: f64, denominator: f64) -> Option<f64> {
    if denominator.abs() < f64::EPSILON {
        None
    } else {
        Some(numerator / denominator)
    }
}

pub(super) fn source_urls(
    current: PeriodView<'_>,
    signals: &[HealthScoreSignalRecord],
) -> Vec<String> {
    let mut urls = current.source_urls().into_iter().collect::<BTreeSet<_>>();
    urls.extend(
        signals
            .iter()
            .flat_map(|signal| signal.source_urls.iter().cloned()),
    );
    urls.into_iter().collect()
}

pub(super) fn piotroski_rating(score: Option<f64>) -> String {
    match score {
        Some(value) if value >= 8.0 => "strong".to_string(),
        Some(value) if value >= 5.0 => "mixed".to_string(),
        Some(_) => "weak".to_string(),
        None => "insufficient_data".to_string(),
    }
}

pub(super) fn altman_rating(score: Option<f64>) -> String {
    match score {
        Some(value) if value > 2.6 => "safe_zone".to_string(),
        Some(value) if value >= 1.1 => "gray_zone".to_string(),
        Some(_) => "distress_zone".to_string(),
        None => "insufficient_data".to_string(),
    }
}

pub(super) fn beneish_rating(score: Option<f64>) -> String {
    match score {
        Some(value) if value > -1.78 => "watch".to_string(),
        Some(_) => "low_risk".to_string(),
        None => "insufficient_data".to_string(),
    }
}

pub(super) fn zip4<T>(
    a: Option<T>,
    b: Option<T>,
    c: Option<T>,
    d: Option<T>,
) -> Option<(T, T, T, T)> {
    Some((a?, b?, c?, d?))
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(super) fn zip8<T>(
    a: Option<T>,
    b: Option<T>,
    c: Option<T>,
    d: Option<T>,
    e: Option<T>,
    f: Option<T>,
    g: Option<T>,
    h: Option<T>,
) -> Option<(T, T, T, T, T, T, T, T)> {
    Some((a?, b?, c?, d?, e?, f?, g?, h?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratings_are_classified() {
        assert_eq!(piotroski_rating(Some(8.0)), "strong");
        assert_eq!(altman_rating(Some(1.5)), "gray_zone");
        assert_eq!(beneish_rating(Some(-1.0)), "watch");
    }
}
