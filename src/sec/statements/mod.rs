use anyhow::{Result, anyhow};
use std::collections::BTreeMap;

use crate::sec::{
    client::SecClient,
    concepts,
    edgar::types::{CompanyFactConcept, CompanyFactValue, CompanyFactsResponse},
    edgar::{
        filings::matches_form,
        urls::{accession_index_url, company_facts_url},
    },
    models::{FinancialStatementRecord, StatementQuery},
};

impl SecClient {
    pub async fn financial_statements(
        &self,
        query: StatementQuery,
    ) -> Result<Vec<FinancialStatementRecord>> {
        let response: CompanyFactsResponse = self.get_json(&company_facts_url(query.cik)).await?;
        let company = response.entity_name;

        let mut records = Vec::new();
        for line in statement_lines(&query.statement)? {
            let mut line_records =
                collect_line_records(&query, company.clone(), &response.facts, line);
            line_records.sort_by(|a, b| {
                b.filed
                    .cmp(&a.filed)
                    .then_with(|| b.end.cmp(&a.end))
                    .then_with(|| b.accession.cmp(&a.accession))
            });
            line_records.truncate(query.latest);
            records.extend(line_records);
        }

        records.sort_by(|a, b| {
            statement_rank(&a.statement)
                .cmp(&statement_rank(&b.statement))
                .then_with(|| a.line_order.cmp(&b.line_order))
                .then_with(|| b.filed.cmp(&a.filed))
                .then_with(|| b.end.cmp(&a.end))
        });
        Ok(records)
    }
}

fn collect_line_records(
    query: &StatementQuery,
    company: Option<String>,
    facts_root: &BTreeMap<String, BTreeMap<String, CompanyFactConcept>>,
    line: StatementLine,
) -> Vec<FinancialStatementRecord> {
    let mut records = Vec::new();
    for (taxonomy, concepts) in facts_root {
        for concept_name in line.concepts {
            let Some(concept_data) = concepts.get(*concept_name) else {
                continue;
            };
            let label = concept_data.label.clone();
            for (unit, values) in &concept_data.units {
                if query
                    .unit
                    .as_deref()
                    .is_some_and(|filter| !unit.eq_ignore_ascii_case(filter))
                {
                    continue;
                }
                for item in values.iter().rev() {
                    let item_form = item.form.as_deref();
                    if query.form.as_deref().is_some_and(|filter| {
                        !matches_form(item_form.unwrap_or(""), Some(filter), true)
                    }) {
                        continue;
                    }
                    records.push(statement_record(
                        query,
                        company.clone(),
                        taxonomy,
                        concept_name,
                        label.clone(),
                        unit,
                        item,
                        line,
                    ));
                }
            }
            if !records.is_empty() {
                return records;
            }
        }
    }
    records
}

fn statement_record(
    query: &StatementQuery,
    company: Option<String>,
    taxonomy: &str,
    concept: &str,
    label: Option<String>,
    unit: &str,
    item: &CompanyFactValue,
    line: StatementLine,
) -> FinancialStatementRecord {
    let accession = item.accn.clone();
    FinancialStatementRecord {
        cik: query.cik,
        company,
        statement: line.statement.to_string(),
        line_order: line.order,
        line_item: line.name.to_string(),
        concept: format!("{taxonomy}:{concept}"),
        taxonomy: taxonomy.to_string(),
        label,
        value: item.val.clone(),
        numeric_value: item.val.as_f64(),
        unit: unit.to_string(),
        fiscal_year: item.fy,
        fiscal_period: item.fp.clone(),
        form: item.form.clone(),
        filed: item.filed.clone(),
        start: item.start.clone(),
        end: item.end.clone(),
        frame: item.frame.clone(),
        source_url: accession
            .as_deref()
            .map(|acc| accession_index_url(query.cik, acc)),
        fact_id: accession
            .as_deref()
            .map(|acc| format!("{taxonomy}:{concept}:{acc}:{unit}")),
        accession,
    }
}

#[derive(Clone, Copy)]
struct StatementLine {
    statement: &'static str,
    order: usize,
    name: &'static str,
    concepts: &'static [&'static str],
}

fn statement_lines(statement: &str) -> Result<Vec<StatementLine>> {
    let normalized = statement.trim().to_ascii_lowercase().replace('-', "_");
    let lines = match normalized.as_str() {
        "income" | "income_statement" => income_lines(),
        "balance" | "balance_sheet" => balance_lines(),
        "cashflow" | "cash_flow" | "cash" => cashflow_lines(),
        "all" => {
            let mut lines = income_lines();
            lines.extend(balance_lines());
            lines.extend(cashflow_lines());
            lines
        }
        _ => return Err(anyhow!("unsupported statement '{}'", statement)),
    };
    Ok(lines)
}

fn statement_rank(statement: &str) -> u8 {
    match statement {
        "income" => 0,
        "balance" => 1,
        "cashflow" => 2,
        _ => 9,
    }
}

fn income_lines() -> Vec<StatementLine> {
    with_orders(vec![
        line("income", "revenue", concepts::REVENUE),
        line("income", "cost_of_revenue", concepts::COST_OF_REVENUE),
        line("income", "gross_profit", concepts::GROSS_PROFIT),
        line(
            "income",
            "research_and_development",
            concepts::RESEARCH_DEVELOPMENT,
        ),
        line(
            "income",
            "selling_general_admin",
            concepts::SELLING_GENERAL_ADMIN,
        ),
        line("income", "operating_expenses", concepts::OPERATING_EXPENSES),
        line("income", "operating_income", concepts::OPERATING_INCOME),
        line("income", "interest_expense", concepts::INTEREST_EXPENSE),
        line("income", "income_before_tax", concepts::PRETAX_INCOME),
        line("income", "income_tax_expense", concepts::TAX_EXPENSE),
        line("income", "net_income", concepts::NET_INCOME),
        line("income", "eps_basic", concepts::EPS_BASIC),
        line("income", "eps_diluted", concepts::EPS_DILUTED),
        line("income", "shares_basic", concepts::SHARES_BASIC),
        line("income", "shares_diluted", concepts::SHARES_DILUTED),
        line(
            "income",
            "comprehensive_income",
            concepts::COMPREHENSIVE_INCOME,
        ),
    ])
}

fn balance_lines() -> Vec<StatementLine> {
    with_orders(vec![
        line("balance", "cash_and_equivalents", concepts::CASH),
        line(
            "balance",
            "marketable_securities_current",
            concepts::MARKETABLE_SECURITIES_CURRENT,
        ),
        line(
            "balance",
            "accounts_receivable",
            concepts::ACCOUNTS_RECEIVABLE,
        ),
        line("balance", "inventory", concepts::INVENTORY),
        line("balance", "current_assets", concepts::CURRENT_ASSETS),
        line("balance", "property_plant_equipment", concepts::PPE),
        line("balance", "goodwill", concepts::GOODWILL),
        line("balance", "intangible_assets", concepts::INTANGIBLES),
        line(
            "balance",
            "operating_lease_assets",
            concepts::OPERATING_LEASE_ASSETS,
        ),
        line("balance", "total_assets", concepts::TOTAL_ASSETS),
        line("balance", "accounts_payable", concepts::ACCOUNTS_PAYABLE),
        line(
            "balance",
            "current_liabilities",
            concepts::CURRENT_LIABILITIES,
        ),
        line("balance", "current_debt", concepts::DEBT_CURRENT),
        line("balance", "long_term_debt", concepts::LONG_TERM_DEBT),
        line(
            "balance",
            "operating_lease_liabilities",
            concepts::OPERATING_LEASE_LIABILITIES,
        ),
        line("balance", "total_liabilities", concepts::TOTAL_LIABILITIES),
        line(
            "balance",
            "stockholders_equity",
            concepts::STOCKHOLDERS_EQUITY,
        ),
        line(
            "balance",
            "liabilities_and_equity",
            concepts::LIABILITIES_AND_EQUITY,
        ),
    ])
}

fn cashflow_lines() -> Vec<StatementLine> {
    with_orders(vec![
        line("cashflow", "net_income", concepts::NET_INCOME),
        line(
            "cashflow",
            "depreciation_amortization",
            concepts::DEPRECIATION_AMORTIZATION,
        ),
        line(
            "cashflow",
            "stock_based_compensation",
            concepts::STOCK_BASED_COMPENSATION,
        ),
        line(
            "cashflow",
            "change_receivables",
            concepts::CHANGE_RECEIVABLES,
        ),
        line("cashflow", "change_inventory", concepts::CHANGE_INVENTORY),
        line("cashflow", "change_payables", concepts::CHANGE_PAYABLES),
        line(
            "cashflow",
            "operating_cash_flow",
            concepts::OPERATING_CASH_FLOW,
        ),
        line("cashflow", "capital_expenditures", concepts::CAPEX),
        line("cashflow", "acquisitions", concepts::ACQUISITIONS),
        line(
            "cashflow",
            "investing_cash_flow",
            concepts::INVESTING_CASH_FLOW,
        ),
        line("cashflow", "dividends_paid", concepts::DIVIDENDS_PAID),
        line("cashflow", "share_repurchases", concepts::SHARE_REPURCHASES),
        line("cashflow", "debt_issuance", concepts::DEBT_ISSUANCE),
        line("cashflow", "debt_repayment", concepts::DEBT_REPAYMENT),
        line(
            "cashflow",
            "financing_cash_flow",
            concepts::FINANCING_CASH_FLOW,
        ),
        line("cashflow", "cash_change", concepts::CASH_CHANGE),
        line("cashflow", "ending_cash", concepts::ENDING_CASH),
    ])
}

fn with_orders(mut lines: Vec<StatementLine>) -> Vec<StatementLine> {
    for (index, line) in lines.iter_mut().enumerate() {
        line.order = index + 1;
    }
    lines
}

const fn line(
    statement: &'static str,
    name: &'static str,
    concepts: &'static [&'static str],
) -> StatementLine {
    StatementLine {
        statement,
        order: 0,
        name,
        concepts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_statement_aliases() {
        assert_eq!(statement_lines("income").unwrap().len(), 16);
        assert_eq!(
            statement_lines("balance-sheet").unwrap()[0].statement,
            "balance"
        );
        assert_eq!(statement_lines("balance-sheet").unwrap()[0].order, 1);
        assert_eq!(
            statement_lines("cash_flow").unwrap()[0].statement,
            "cashflow"
        );
        assert!(statement_lines("all").unwrap().len() > 40);
    }
}
