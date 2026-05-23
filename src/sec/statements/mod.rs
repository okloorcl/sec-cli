use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::sec::{
    client::SecClient,
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
        let json: Value = self.get_json(&company_facts_url(query.cik)).await?;
        let company = json
            .get("entityName")
            .and_then(Value::as_str)
            .map(str::to_string);
        let facts_root = json
            .get("facts")
            .and_then(Value::as_object)
            .ok_or_else(|| anyhow!("companyfacts JSON missing facts"))?;

        let mut records = Vec::new();
        for line in statement_lines(&query.statement)? {
            let mut line_records = collect_line_records(&query, company.clone(), facts_root, line);
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
    facts_root: &serde_json::Map<String, Value>,
    line: StatementLine,
) -> Vec<FinancialStatementRecord> {
    let mut records = Vec::new();
    for (taxonomy, concepts) in facts_root {
        let Some(concepts) = concepts.as_object() else {
            continue;
        };
        for concept_name in line.concepts {
            let Some(concept_data) = concepts.get(*concept_name) else {
                continue;
            };
            let label = concept_data
                .get("label")
                .and_then(Value::as_str)
                .map(str::to_string);
            let Some(units) = concept_data.get("units").and_then(Value::as_object) else {
                continue;
            };
            for (unit, values) in units {
                if query
                    .unit
                    .as_deref()
                    .is_some_and(|filter| !unit.eq_ignore_ascii_case(filter))
                {
                    continue;
                }
                let Some(values) = values.as_array() else {
                    continue;
                };
                for item in values.iter().rev() {
                    let item_form = item.get("form").and_then(Value::as_str);
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
    item: &Value,
    line: StatementLine,
) -> FinancialStatementRecord {
    let accession = item.get("accn").and_then(Value::as_str).map(str::to_string);
    FinancialStatementRecord {
        cik: query.cik,
        company,
        statement: line.statement.to_string(),
        line_order: line.order,
        line_item: line.name.to_string(),
        concept: format!("{taxonomy}:{concept}"),
        taxonomy: taxonomy.to_string(),
        label,
        value: item.get("val").cloned().unwrap_or(Value::Null),
        numeric_value: item.get("val").and_then(Value::as_f64),
        unit: unit.to_string(),
        fiscal_year: item.get("fy").and_then(Value::as_i64),
        fiscal_period: item.get("fp").and_then(Value::as_str).map(str::to_string),
        form: item.get("form").and_then(Value::as_str).map(str::to_string),
        filed: item
            .get("filed")
            .and_then(Value::as_str)
            .map(str::to_string),
        start: item
            .get("start")
            .and_then(Value::as_str)
            .map(str::to_string),
        end: item.get("end").and_then(Value::as_str).map(str::to_string),
        frame: item
            .get("frame")
            .and_then(Value::as_str)
            .map(str::to_string),
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
        line(
            "income",
            "revenue",
            &[
                "RevenueFromContractWithCustomerExcludingAssessedTax",
                "Revenues",
                "SalesRevenueNet",
            ],
        ),
        line(
            "income",
            "cost_of_revenue",
            &["CostOfRevenue", "CostOfGoodsAndServicesSold"],
        ),
        line("income", "gross_profit", &["GrossProfit"]),
        line("income", "operating_expenses", &["OperatingExpenses"]),
        line("income", "operating_income", &["OperatingIncomeLoss"]),
        line(
            "income",
            "income_before_tax",
            &[
                "IncomeLossFromContinuingOperationsBeforeIncomeTaxesExtraordinaryItemsNoncontrollingInterest",
            ],
        ),
        line("income", "income_tax_expense", &["IncomeTaxExpenseBenefit"]),
        line("income", "net_income", &["NetIncomeLoss"]),
        line("income", "eps_basic", &["EarningsPerShareBasic"]),
        line("income", "eps_diluted", &["EarningsPerShareDiluted"]),
        line(
            "income",
            "shares_basic",
            &["WeightedAverageNumberOfSharesOutstandingBasic"],
        ),
        line(
            "income",
            "shares_diluted",
            &["WeightedAverageNumberOfDilutedSharesOutstanding"],
        ),
    ])
}

fn balance_lines() -> Vec<StatementLine> {
    with_orders(vec![
        line(
            "balance",
            "cash_and_equivalents",
            &["CashAndCashEquivalentsAtCarryingValue"],
        ),
        line(
            "balance",
            "marketable_securities_current",
            &["MarketableSecuritiesCurrent"],
        ),
        line(
            "balance",
            "accounts_receivable",
            &["AccountsReceivableNetCurrent"],
        ),
        line("balance", "inventory", &["InventoryNet"]),
        line("balance", "current_assets", &["AssetsCurrent"]),
        line(
            "balance",
            "property_plant_equipment",
            &["PropertyPlantAndEquipmentNet"],
        ),
        line("balance", "total_assets", &["Assets"]),
        line("balance", "accounts_payable", &["AccountsPayableCurrent"]),
        line("balance", "current_liabilities", &["LiabilitiesCurrent"]),
        line("balance", "total_liabilities", &["Liabilities"]),
        line("balance", "stockholders_equity", &["StockholdersEquity"]),
        line(
            "balance",
            "liabilities_and_equity",
            &["LiabilitiesAndStockholdersEquity"],
        ),
    ])
}

fn cashflow_lines() -> Vec<StatementLine> {
    with_orders(vec![
        line("cashflow", "net_income", &["NetIncomeLoss"]),
        line(
            "cashflow",
            "depreciation_amortization",
            &["DepreciationDepletionAndAmortization"],
        ),
        line(
            "cashflow",
            "stock_based_compensation",
            &["ShareBasedCompensation"],
        ),
        line(
            "cashflow",
            "operating_cash_flow",
            &["NetCashProvidedByUsedInOperatingActivities"],
        ),
        line(
            "cashflow",
            "capital_expenditures",
            &["PaymentsToAcquirePropertyPlantAndEquipment"],
        ),
        line(
            "cashflow",
            "investing_cash_flow",
            &["NetCashProvidedByUsedInInvestingActivities"],
        ),
        line("cashflow", "dividends_paid", &["PaymentsOfDividends"]),
        line(
            "cashflow",
            "share_repurchases",
            &["PaymentsForRepurchaseOfCommonStock"],
        ),
        line(
            "cashflow",
            "financing_cash_flow",
            &["NetCashProvidedByUsedInFinancingActivities"],
        ),
        line(
            "cashflow",
            "cash_change",
            &[
                "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseIncludingExchangeRateEffect",
            ],
        ),
        line(
            "cashflow",
            "ending_cash",
            &["CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents"],
        ),
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
        assert_eq!(statement_lines("income").unwrap().len(), 12);
        assert_eq!(
            statement_lines("balance-sheet").unwrap()[0].statement,
            "balance"
        );
        assert_eq!(statement_lines("balance-sheet").unwrap()[0].order, 1);
        assert_eq!(
            statement_lines("cash_flow").unwrap()[0].statement,
            "cashflow"
        );
        assert!(statement_lines("all").unwrap().len() > 20);
    }
}
