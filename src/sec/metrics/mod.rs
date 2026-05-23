use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::Result;

mod calc;

use calc::{display_value, free_cash_flow, growth, safe_div};

use crate::sec::{
    client::SecClient,
    models::{
        FinancialMetricRecord, FinancialStatementRecord, MetricComponentRecord, MetricsQuery,
        StatementQuery,
    },
};

impl SecClient {
    pub async fn financial_metrics(
        &self,
        query: MetricsQuery,
    ) -> Result<Vec<FinancialMetricRecord>> {
        let statement_rows = self
            .financial_statements(StatementQuery {
                cik: query.cik,
                statement: "all".to_string(),
                form: query.form.clone(),
                unit: query.unit.clone(),
                latest: query.latest.saturating_add(1).max(2),
            })
            .await?;

        Ok(build_metrics(query.cik, query.latest, &statement_rows))
    }
}

fn build_metrics(
    cik: u64,
    latest: usize,
    rows: &[FinancialStatementRecord],
) -> Vec<FinancialMetricRecord> {
    let periods = period_maps(rows);
    let mut records = Vec::new();

    for index in 0..periods.len().min(latest) {
        let period = &periods[index];
        let previous = periods.get(index + 1);

        push_ratio(
            &mut records,
            cik,
            period,
            "gross_margin",
            "profitability",
            "gross_profit / revenue",
            &["income:gross_profit", "income:revenue"],
        );
        push_ratio(
            &mut records,
            cik,
            period,
            "operating_margin",
            "profitability",
            "operating_income / revenue",
            &["income:operating_income", "income:revenue"],
        );
        push_ratio(
            &mut records,
            cik,
            period,
            "net_margin",
            "profitability",
            "net_income / revenue",
            &["income:net_income", "income:revenue"],
        );
        push_growth(
            &mut records,
            cik,
            period,
            previous,
            "revenue_growth",
            "growth",
            "current revenue / previous revenue - 1",
            "income:revenue",
        );
        push_growth(
            &mut records,
            cik,
            period,
            previous,
            "net_income_growth",
            "growth",
            "current net_income / previous net_income - 1",
            "income:net_income",
        );
        push_fcf(&mut records, cik, period);
        push_ratio(
            &mut records,
            cik,
            period,
            "free_cash_flow_margin",
            "cashflow",
            "free_cash_flow / revenue",
            &["derived:free_cash_flow", "income:revenue"],
        );
        push_ratio(
            &mut records,
            cik,
            period,
            "return_on_assets",
            "returns",
            "net_income / total_assets",
            &["income:net_income", "balance:total_assets"],
        );
        push_ratio(
            &mut records,
            cik,
            period,
            "return_on_equity",
            "returns",
            "net_income / stockholders_equity",
            &["income:net_income", "balance:stockholders_equity"],
        );
        push_quotient(
            &mut records,
            cik,
            period,
            "current_ratio",
            "liquidity",
            "current_assets / current_liabilities",
            "multiple",
            &["balance:current_assets", "balance:current_liabilities"],
        );
        push_ratio(
            &mut records,
            cik,
            period,
            "liabilities_to_assets",
            "leverage",
            "total_liabilities / total_assets",
            &["balance:total_liabilities", "balance:total_assets"],
        );
        push_ratio(
            &mut records,
            cik,
            period,
            "cash_to_assets",
            "liquidity",
            "cash_and_equivalents / total_assets",
            &["balance:cash_and_equivalents", "balance:total_assets"],
        );
        push_ratio(
            &mut records,
            cik,
            period,
            "share_repurchases_to_revenue",
            "capital_return",
            "absolute share_repurchases / revenue",
            &["derived:absolute_share_repurchases", "income:revenue"],
        );
    }

    records
}

#[derive(Debug)]
struct PeriodMap<'a> {
    rows: HashMap<String, &'a FinancialStatementRecord>,
    derived: HashMap<String, DerivedValue<'a>>,
}

#[derive(Debug)]
struct DerivedValue<'a> {
    value: f64,
    unit: &'static str,
    components: Vec<&'a FinancialStatementRecord>,
}

fn period_maps(rows: &[FinancialStatementRecord]) -> Vec<PeriodMap<'_>> {
    let mut grouped: BTreeMap<String, Vec<&FinancialStatementRecord>> = BTreeMap::new();
    for row in rows {
        grouped.entry(period_key(row)).or_default().push(row);
    }

    let mut periods: Vec<_> = grouped
        .into_iter()
        .map(|(_, rows)| {
            let mut period = PeriodMap {
                rows: HashMap::new(),
                derived: HashMap::new(),
            };
            for row in rows {
                period
                    .rows
                    .entry(metric_key(&row.statement, &row.line_item))
                    .or_insert(row);
            }
            add_derived_values(&mut period);
            period
        })
        .collect();
    periods.sort_by(|a, b| b.period_end().cmp(&a.period_end()));
    periods
}

fn period_key(row: &FinancialStatementRecord) -> String {
    row.end
        .as_deref()
        .or(row.filed.as_deref())
        .unwrap_or("unknown")
        .to_string()
}

fn add_derived_values<'a>(period: &mut PeriodMap<'a>) {
    if let (Some(ocf), Some(capex)) = (
        period.row("cashflow:operating_cash_flow"),
        period.row("cashflow:capital_expenditures"),
    ) {
        if let (Some(ocf_value), Some(capex_value)) = (ocf.numeric_value, capex.numeric_value) {
            period.derived.insert(
                "derived:free_cash_flow".to_string(),
                DerivedValue {
                    value: free_cash_flow(ocf_value, capex_value),
                    unit: "USD",
                    components: vec![ocf, capex],
                },
            );
        }
    }

    if let Some(repurchase) = period.row("cashflow:share_repurchases") {
        if let Some(value) = repurchase.numeric_value {
            period.derived.insert(
                "derived:absolute_share_repurchases".to_string(),
                DerivedValue {
                    value: value.abs(),
                    unit: "USD",
                    components: vec![repurchase],
                },
            );
        }
    }
}

fn push_ratio(
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

fn push_quotient(
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
        period.components(keys),
    ));
}

fn push_growth(
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

    let mut components = period.components(&[key]);
    components.extend(previous_period.components(&[key]));
    records.push(metric_record(
        cik,
        period,
        metric,
        category,
        Some(value),
        "ratio",
        calculation,
        components,
    ));
}

fn push_fcf(records: &mut Vec<FinancialMetricRecord>, cik: u64, period: &PeriodMap<'_>) {
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
                line_item: key.to_string(),
                statement: "derived".to_string(),
                value: Some(value),
                unit: unit.to_string(),
                accession: None,
                fact_id: None,
                source_url: None,
            },
        }
    }
}

impl<'a> PeriodMap<'a> {
    fn row(&self, key: &str) -> Option<&'a FinancialStatementRecord> {
        self.rows.get(key).copied()
    }

    fn value(&self, key: &str) -> Option<f64> {
        self.row(key)
            .and_then(|row| row.numeric_value)
            .or_else(|| self.derived.get(key).map(|derived| derived.value))
    }

    fn components(&self, keys: &[&str]) -> Vec<ComponentSource<'a>> {
        keys.iter()
            .flat_map(|key| {
                if let Some(row) = self.row(key) {
                    vec![ComponentSource::Row(row)]
                } else if let Some(derived) = self.derived.get(*key) {
                    let mut components = vec![ComponentSource::Derived {
                        key: (*key).to_string(),
                        value: derived.value,
                        unit: derived.unit.to_string(),
                    }];
                    components.extend(
                        derived
                            .components
                            .iter()
                            .map(|row| ComponentSource::Row(row)),
                    );
                    components
                } else {
                    Vec::new()
                }
            })
            .collect()
    }

    fn first_row(&self) -> Option<&FinancialStatementRecord> {
        self.rows.values().next().copied()
    }

    fn company(&self) -> Option<String> {
        self.first_row().and_then(|row| row.company.clone())
    }

    fn fiscal_year(&self) -> Option<i64> {
        self.first_row().and_then(|row| row.fiscal_year)
    }

    fn fiscal_period(&self) -> Option<String> {
        self.first_row().and_then(|row| row.fiscal_period.clone())
    }

    fn form(&self) -> Option<String> {
        self.first_row().and_then(|row| row.form.clone())
    }

    fn period_end(&self) -> Option<String> {
        self.first_row().and_then(|row| row.end.clone())
    }
}

fn metric_key(statement: &str, line_item: &str) -> String {
    format!("{statement}:{line_item}")
}
