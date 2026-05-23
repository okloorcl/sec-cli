use std::collections::BTreeSet;

use crate::sec::models::{FinancialMetricRecord, FinancialStatementRecord, MetricComponentRecord};

use super::{
    calc::{display_value, growth, safe_div},
    period::PeriodMap,
};

pub(super) fn push_ratio(
    records: &mut Vec<FinancialMetricRecord>,
    cik: u64,
    period: &PeriodMap<'_>,
    metric: &str,
    category: &str,
    calculation: &str,
    keys: &[&str; 2],
) {
    push_quotient(
        records,
        cik,
        period,
        metric,
        category,
        calculation,
        "ratio",
        keys,
    );
}

pub(super) fn push_quotient(
    records: &mut Vec<FinancialMetricRecord>,
    cik: u64,
    period: &PeriodMap<'_>,
    metric: &str,
    category: &str,
    calculation: &str,
    unit: &str,
    keys: &[&str; 2],
) {
    let Some(numerator) = period.value(keys[0]) else {
        return;
    };
    let Some(denominator) = period.value(keys[1]) else {
        return;
    };
    let Some(value) = safe_div(numerator, denominator) else {
        return;
    };
    records.push(metric_record(
        cik,
        period,
        metric,
        category,
        Some(value),
        unit,
        calculation,
        components(period, keys),
    ));
}

pub(super) fn push_growth(
    records: &mut Vec<FinancialMetricRecord>,
    cik: u64,
    period: &PeriodMap<'_>,
    previous: Option<&PeriodMap<'_>>,
    metric: &str,
    category: &str,
    calculation: &str,
    key: &str,
) {
    let Some(current) = period.value(key) else {
        return;
    };
    let Some(previous_period) = previous else {
        return;
    };
    let Some(previous_value) = previous_period.value(key) else {
        return;
    };
    let Some(value) = growth(current, previous_value) else {
        return;
    };

    let mut sources = components(period, &[key]);
    sources.extend(components(previous_period, &[key]));
    records.push(metric_record(
        cik,
        period,
        metric,
        category,
        Some(value),
        "ratio",
        calculation,
        sources,
    ));
}

pub(super) fn push_fcf(records: &mut Vec<FinancialMetricRecord>, cik: u64, period: &PeriodMap<'_>) {
    let Some(fcf) = period.derived.get("derived:free_cash_flow") else {
        return;
    };
    records.push(metric_record(
        cik,
        period,
        "free_cash_flow",
        "cashflow",
        Some(fcf.value),
        fcf.unit,
        "operating_cash_flow - absolute capital_expenditures",
        fcf.components
            .iter()
            .map(|row| ComponentSource::Row(row))
            .collect(),
    ));
}

pub(super) fn push_derived_metric(
    records: &mut Vec<FinancialMetricRecord>,
    cik: u64,
    period: &PeriodMap<'_>,
    metric: &str,
    category: &str,
    calculation: &str,
    unit: &str,
    key: &str,
) {
    let Some(derived) = period.derived.get(key) else {
        return;
    };
    records.push(metric_record(
        cik,
        period,
        metric,
        category,
        Some(derived.value),
        unit,
        calculation,
        components(period, &[key]),
    ));
}

fn metric_record(
    cik: u64,
    period: &PeriodMap<'_>,
    metric: &str,
    category: &str,
    value: Option<f64>,
    unit: &str,
    calculation: &str,
    components: Vec<ComponentSource<'_>>,
) -> FinancialMetricRecord {
    let source_urls = components
        .iter()
        .filter_map(ComponentSource::source_url)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    FinancialMetricRecord {
        cik,
        company: period.company(),
        metric: metric.to_string(),
        category: category.to_string(),
        value,
        display_value: value.map(|value| display_value(value, unit)),
        unit: unit.to_string(),
        fiscal_year: period.fiscal_year(),
        fiscal_period: period.fiscal_period(),
        form: period.form(),
        period_end: period.period_end(),
        calculation: calculation.to_string(),
        components: components
            .into_iter()
            .map(ComponentSource::component_record)
            .collect(),
        source_urls,
    }
}

enum ComponentSource<'a> {
    Row(&'a FinancialStatementRecord),
    Derived {
        key: String,
        value: f64,
        unit: String,
    },
}

impl<'a> ComponentSource<'a> {
    fn source_url(&self) -> Option<String> {
        match self {
            Self::Row(row) => row.source_url.clone(),
            Self::Derived { .. } => None,
        }
    }

    fn component_record(self) -> MetricComponentRecord {
        match self {
            Self::Row(row) => MetricComponentRecord {
                line_item: row.line_item.clone(),
                statement: row.statement.clone(),
                value: row.numeric_value,
                unit: row.unit.clone(),
                accession: row.accession.clone(),
                fact_id: row.fact_id.clone(),
                source_url: row.source_url.clone(),
            },
            Self::Derived { key, value, unit } => MetricComponentRecord {
                line_item: key,
                statement: "derived".to_string(),
                value: Some(value),
                unit,
                accession: None,
                fact_id: None,
                source_url: None,
            },
        }
    }
}

fn components<'a>(period: &'a PeriodMap<'a>, keys: &[&str]) -> Vec<ComponentSource<'a>> {
    keys.iter()
        .flat_map(|key| {
            if let Some(row) = period.row(key) {
                vec![ComponentSource::Row(row)]
            } else if let Some(derived) = period.derived.get(*key) {
                let mut sources = vec![ComponentSource::Derived {
                    key: (*key).to_string(),
                    value: derived.value,
                    unit: derived.unit.to_string(),
                }];
                sources.extend(
                    derived
                        .components
                        .iter()
                        .map(|row| ComponentSource::Row(row)),
                );
                sources
            } else {
                Vec::new()
            }
        })
        .collect()
}
