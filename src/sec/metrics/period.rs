use std::collections::{BTreeMap, HashMap};

use crate::sec::models::FinancialStatementRecord;

use super::calc::{free_cash_flow, safe_div};

#[derive(Debug)]
pub(super) struct PeriodMap<'a> {
    pub(super) rows: HashMap<String, &'a FinancialStatementRecord>,
    pub(super) derived: HashMap<String, DerivedValue<'a>>,
}

#[derive(Debug)]
pub(super) struct DerivedValue<'a> {
    pub(super) value: f64,
    pub(super) unit: &'static str,
    pub(super) components: Vec<&'a FinancialStatementRecord>,
}

pub(super) fn period_maps(rows: &[FinancialStatementRecord]) -> Vec<PeriodMap<'_>> {
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
    add_sum(
        period,
        "derived:quick_assets",
        &[
            "balance:cash_and_equivalents",
            "balance:marketable_securities_current",
            "balance:accounts_receivable",
        ],
    );
    add_sum(
        period,
        "derived:cash_and_securities",
        &[
            "balance:cash_and_equivalents",
            "balance:marketable_securities_current",
        ],
    );
    add_sum(
        period,
        "derived:total_debt",
        &["balance:current_debt", "balance:long_term_debt"],
    );
    add_sum(
        period,
        "derived:total_capital",
        &["derived:total_debt", "balance:stockholders_equity"],
    );
    add_difference(
        period,
        "derived:working_capital",
        "balance:current_assets",
        "balance:current_liabilities",
    );
    add_difference(
        period,
        "derived:net_debt",
        "derived:total_debt",
        "balance:cash_and_equivalents",
    );
    add_difference(
        period,
        "derived:invested_capital",
        "derived:total_debt",
        "balance:cash_and_equivalents",
    );
    add_sum(
        period,
        "derived:invested_capital",
        &["derived:invested_capital", "balance:stockholders_equity"],
    );

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

    if let (Some(operating_income), Some(tax_expense), Some(pretax_income)) = (
        period.row("income:operating_income"),
        period.row("income:income_tax_expense"),
        period.row("income:income_before_tax"),
    ) {
        if let (Some(operating), Some(tax), Some(pretax)) = (
            operating_income.numeric_value,
            tax_expense.numeric_value,
            pretax_income.numeric_value,
        ) {
            let tax_rate = safe_div(tax, pretax).unwrap_or(0.0).clamp(0.0, 1.0);
            period.derived.insert(
                "derived:nopat".to_string(),
                DerivedValue {
                    value: operating * (1.0 - tax_rate),
                    unit: "USD",
                    components: vec![operating_income, tax_expense, pretax_income],
                },
            );
        }
    }

    add_absolute(
        period,
        "derived:absolute_capex",
        "cashflow:capital_expenditures",
    );
    add_absolute(
        period,
        "derived:absolute_dividends_paid",
        "cashflow:dividends_paid",
    );
    add_absolute(
        period,
        "derived:absolute_share_repurchases",
        "cashflow:share_repurchases",
    );
}

fn add_sum<'a>(period: &mut PeriodMap<'a>, key: &str, component_keys: &[&str]) {
    let mut value = 0.0;
    let mut components = Vec::new();
    for component_key in component_keys {
        if let Some(component) = period.row(component_key) {
            if let Some(component_value) = component.numeric_value {
                value += component_value;
                components.push(component);
            }
        } else if let Some(component) = period.derived.get(*component_key) {
            value += component.value;
            components.extend(component.components.iter().copied());
        }
    }
    if !components.is_empty() {
        period.derived.insert(
            key.to_string(),
            DerivedValue {
                value,
                unit: "USD",
                components,
            },
        );
    }
}

fn add_difference<'a>(period: &mut PeriodMap<'a>, key: &str, left_key: &str, right_key: &str) {
    let Some(left) = period.value(left_key) else {
        return;
    };
    let Some(right) = period.value(right_key) else {
        return;
    };
    let mut components = period.component_rows(left_key);
    components.extend(period.component_rows(right_key));
    period.derived.insert(
        key.to_string(),
        DerivedValue {
            value: left - right,
            unit: "USD",
            components,
        },
    );
}

fn add_absolute<'a>(period: &mut PeriodMap<'a>, key: &str, source_key: &str) {
    let Some(source) = period.row(source_key) else {
        return;
    };
    let Some(value) = source.numeric_value else {
        return;
    };
    period.derived.insert(
        key.to_string(),
        DerivedValue {
            value: value.abs(),
            unit: "USD",
            components: vec![source],
        },
    );
}

impl<'a> PeriodMap<'a> {
    pub(super) fn row(&self, key: &str) -> Option<&'a FinancialStatementRecord> {
        self.rows.get(key).copied()
    }

    pub(super) fn value(&self, key: &str) -> Option<f64> {
        self.row(key)
            .and_then(|row| row.numeric_value)
            .or_else(|| self.derived.get(key).map(|derived| derived.value))
    }

    pub(super) fn component_rows(&self, key: &str) -> Vec<&'a FinancialStatementRecord> {
        if let Some(row) = self.row(key) {
            vec![row]
        } else {
            self.derived
                .get(key)
                .map(|derived| derived.components.clone())
                .unwrap_or_default()
        }
    }

    pub(super) fn first_row(&self) -> Option<&FinancialStatementRecord> {
        self.rows.values().next().copied()
    }

    pub(super) fn company(&self) -> Option<String> {
        self.first_row().and_then(|row| row.company.clone())
    }

    pub(super) fn fiscal_year(&self) -> Option<i64> {
        self.first_row().and_then(|row| row.fiscal_year)
    }

    pub(super) fn fiscal_period(&self) -> Option<String> {
        self.first_row().and_then(|row| row.fiscal_period.clone())
    }

    pub(super) fn form(&self) -> Option<String> {
        self.first_row().and_then(|row| row.form.clone())
    }

    pub(super) fn period_end(&self) -> Option<String> {
        self.first_row().and_then(|row| row.end.clone())
    }
}

fn metric_key(statement: &str, line_item: &str) -> String {
    format!("{statement}:{line_item}")
}
