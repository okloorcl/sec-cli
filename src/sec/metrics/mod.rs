mod calc;
mod emit;
mod extended;
mod period;

use anyhow::Result;
use emit::{push_derived_metric, push_fcf, push_growth, push_quotient, push_ratio};
use extended::push_extended_metrics;
use period::period_maps;

use crate::sec::{
    client::SecClient,
    models::{FinancialMetricRecord, FinancialStatementRecord, MetricsQuery, StatementQuery},
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
        push_derived_metric(
            &mut records,
            cik,
            period,
            "working_capital",
            "liquidity",
            "current_assets - current_liabilities",
            "USD",
            "derived:working_capital",
        );
        push_derived_metric(
            &mut records,
            cik,
            period,
            "total_debt",
            "leverage",
            "current_debt + long_term_debt",
            "USD",
            "derived:total_debt",
        );
        push_derived_metric(
            &mut records,
            cik,
            period,
            "net_debt",
            "leverage",
            "total_debt - cash_and_equivalents",
            "USD",
            "derived:net_debt",
        );
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
        push_quotient(
            &mut records,
            cik,
            period,
            "quick_ratio",
            "liquidity",
            "(cash + marketable_securities_current + accounts_receivable) / current_liabilities",
            "multiple",
            &["derived:quick_assets", "balance:current_liabilities"],
        );
        push_quotient(
            &mut records,
            cik,
            period,
            "cash_ratio",
            "liquidity",
            "cash_and_equivalents / current_liabilities",
            "multiple",
            &[
                "balance:cash_and_equivalents",
                "balance:current_liabilities",
            ],
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
            "debt_to_equity",
            "leverage",
            "total_debt / stockholders_equity",
            &["derived:total_debt", "balance:stockholders_equity"],
        );
        push_ratio(
            &mut records,
            cik,
            period,
            "debt_to_assets",
            "leverage",
            "total_debt / total_assets",
            &["derived:total_debt", "balance:total_assets"],
        );
        push_ratio(
            &mut records,
            cik,
            period,
            "net_debt_to_equity",
            "leverage",
            "net_debt / stockholders_equity",
            &["derived:net_debt", "balance:stockholders_equity"],
        );
        push_quotient(
            &mut records,
            cik,
            period,
            "interest_coverage",
            "solvency",
            "operating_income / interest_expense",
            "multiple",
            &["income:operating_income", "income:interest_expense"],
        );
        push_ratio(
            &mut records,
            cik,
            period,
            "effective_tax_rate",
            "profitability",
            "income_tax_expense / income_before_tax",
            &["income:income_tax_expense", "income:income_before_tax"],
        );
        push_ratio(
            &mut records,
            cik,
            period,
            "roic",
            "returns",
            "nopat / invested_capital",
            &["derived:nopat", "derived:invested_capital"],
        );
        push_quotient(
            &mut records,
            cik,
            period,
            "asset_turnover",
            "efficiency",
            "revenue / total_assets",
            "multiple",
            &["income:revenue", "balance:total_assets"],
        );
        push_ratio(
            &mut records,
            cik,
            period,
            "operating_cash_flow_margin",
            "cashflow",
            "operating_cash_flow / revenue",
            &["cashflow:operating_cash_flow", "income:revenue"],
        );
        push_ratio(
            &mut records,
            cik,
            period,
            "free_cash_flow_to_net_income",
            "cashflow",
            "free_cash_flow / net_income",
            &["derived:free_cash_flow", "income:net_income"],
        );
        push_ratio(
            &mut records,
            cik,
            period,
            "capex_to_revenue",
            "capital_intensity",
            "absolute capital_expenditures / revenue",
            &["derived:absolute_capex", "income:revenue"],
        );
        push_ratio(
            &mut records,
            cik,
            period,
            "dividend_payout_ratio",
            "capital_return",
            "absolute dividends_paid / net_income",
            &["derived:absolute_dividends_paid", "income:net_income"],
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
        push_ratio(
            &mut records,
            cik,
            period,
            "share_repurchases_to_free_cash_flow",
            "capital_return",
            "absolute share_repurchases / free_cash_flow",
            &[
                "derived:absolute_share_repurchases",
                "derived:free_cash_flow",
            ],
        );
        push_extended_metrics(&mut records, cik, period);
    }

    records
}
