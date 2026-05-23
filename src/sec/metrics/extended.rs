use crate::sec::models::FinancialMetricRecord;

use super::{
    emit::{push_quotient, push_ratio},
    period::PeriodMap,
};

pub(super) fn push_extended_metrics(
    records: &mut Vec<FinancialMetricRecord>,
    cik: u64,
    period: &PeriodMap<'_>,
) {
    for (metric, category, calculation, keys) in ratio_specs() {
        push_ratio(records, cik, period, metric, category, calculation, &keys);
    }
    for (metric, category, calculation, keys) in quotient_specs() {
        push_quotient(
            records,
            cik,
            period,
            metric,
            category,
            calculation,
            "multiple",
            &keys,
        );
    }
}

type MetricSpec = (&'static str, &'static str, &'static str, [&'static str; 2]);

fn ratio_specs() -> [MetricSpec; 20] {
    [
        (
            "cost_of_revenue_margin",
            "profitability",
            "cost_of_revenue / revenue",
            ["income:cost_of_revenue", "income:revenue"],
        ),
        (
            "rd_to_revenue",
            "expense_intensity",
            "research_and_development / revenue",
            ["income:research_and_development", "income:revenue"],
        ),
        (
            "sga_to_revenue",
            "expense_intensity",
            "selling_general_admin / revenue",
            ["income:selling_general_admin", "income:revenue"],
        ),
        (
            "operating_expense_ratio",
            "expense_intensity",
            "operating_expenses / revenue",
            ["income:operating_expenses", "income:revenue"],
        ),
        (
            "pretax_margin",
            "profitability",
            "income_before_tax / revenue",
            ["income:income_before_tax", "income:revenue"],
        ),
        (
            "cash_flow_return_on_assets",
            "returns",
            "operating_cash_flow / total_assets",
            ["cashflow:operating_cash_flow", "balance:total_assets"],
        ),
        (
            "cash_flow_to_debt",
            "solvency",
            "operating_cash_flow / total_debt",
            ["cashflow:operating_cash_flow", "derived:total_debt"],
        ),
        (
            "equity_ratio",
            "leverage",
            "stockholders_equity / total_assets",
            ["balance:stockholders_equity", "balance:total_assets"],
        ),
        (
            "debt_to_capital",
            "leverage",
            "total_debt / total_capital",
            ["derived:total_debt", "derived:total_capital"],
        ),
        (
            "working_capital_to_assets",
            "liquidity",
            "working_capital / total_assets",
            ["derived:working_capital", "balance:total_assets"],
        ),
        (
            "working_capital_to_revenue",
            "liquidity",
            "working_capital / revenue",
            ["derived:working_capital", "income:revenue"],
        ),
        (
            "inventory_to_current_assets",
            "efficiency",
            "inventory / current_assets",
            ["balance:inventory", "balance:current_assets"],
        ),
        (
            "receivables_to_revenue",
            "efficiency",
            "accounts_receivable / revenue",
            ["balance:accounts_receivable", "income:revenue"],
        ),
        (
            "fcf_to_debt",
            "solvency",
            "free_cash_flow / total_debt",
            ["derived:free_cash_flow", "derived:total_debt"],
        ),
        (
            "lease_liabilities_to_assets",
            "leverage",
            "operating_lease_liabilities / total_assets",
            [
                "balance:operating_lease_liabilities",
                "balance:total_assets",
            ],
        ),
        (
            "goodwill_to_assets",
            "asset_quality",
            "goodwill / total_assets",
            ["balance:goodwill", "balance:total_assets"],
        ),
        (
            "intangibles_to_assets",
            "asset_quality",
            "intangible_assets / total_assets",
            ["balance:intangible_assets", "balance:total_assets"],
        ),
        (
            "marketable_securities_to_assets",
            "liquidity",
            "marketable_securities_current / total_assets",
            [
                "balance:marketable_securities_current",
                "balance:total_assets",
            ],
        ),
        (
            "cash_and_securities_to_assets",
            "liquidity",
            "cash_and_securities / total_assets",
            ["derived:cash_and_securities", "balance:total_assets"],
        ),
        (
            "capex_to_operating_cash_flow",
            "capital_intensity",
            "absolute capital_expenditures / operating_cash_flow",
            ["derived:absolute_capex", "cashflow:operating_cash_flow"],
        ),
    ]
}

fn quotient_specs() -> [MetricSpec; 4] {
    [
        (
            "cash_conversion",
            "cashflow",
            "operating_cash_flow / net_income",
            ["cashflow:operating_cash_flow", "income:net_income"],
        ),
        (
            "inventory_turnover",
            "efficiency",
            "cost_of_revenue / inventory",
            ["income:cost_of_revenue", "balance:inventory"],
        ),
        (
            "receivables_turnover",
            "efficiency",
            "revenue / accounts_receivable",
            ["income:revenue", "balance:accounts_receivable"],
        ),
        (
            "cash_and_securities_coverage",
            "liquidity",
            "cash_and_securities / current_liabilities",
            ["derived:cash_and_securities", "balance:current_liabilities"],
        ),
    ]
}
