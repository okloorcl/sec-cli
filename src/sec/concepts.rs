pub(crate) const REVENUE: &[&str] = &[
    "RevenueFromContractWithCustomerExcludingAssessedTax",
    "RevenueFromContractWithCustomerIncludingAssessedTax",
    "Revenues",
    "SalesRevenueNet",
    "SalesRevenueGoodsNet",
    "SalesRevenueServicesNet",
    "InterestAndDividendIncomeOperating",
    "RealEstateRevenueNet",
];
pub(crate) const COST_OF_REVENUE: &[&str] = &[
    "CostOfRevenue",
    "CostOfGoodsAndServicesSold",
    "CostOfGoodsSold",
    "CostOfGoodsAndServiceExcludingDepreciationDepletionAndAmortization",
    "CostOfRevenueExcludingDepreciationDepletionAndAmortization",
    "CostOfServicesRevenue",
    "CostOfGoodsRevenue",
];
pub(crate) const GROSS_PROFIT: &[&str] = &["GrossProfit"];
pub(crate) const RESEARCH_DEVELOPMENT: &[&str] = &[
    "ResearchAndDevelopmentExpense",
    "ResearchAndDevelopmentExpenseExcludingAcquiredInProcessCost",
];
pub(crate) const SELLING_GENERAL_ADMIN: &[&str] = &[
    "SellingGeneralAndAdministrativeExpense",
    "GeneralAndAdministrativeExpense",
    "SellingAndMarketingExpense",
];
pub(crate) const OPERATING_EXPENSES: &[&str] = &[
    "OperatingExpenses",
    "CostsAndExpenses",
    "NoninterestExpense",
];
pub(crate) const OPERATING_INCOME: &[&str] = &[
    "OperatingIncomeLoss",
    "IncomeLossFromContinuingOperationsBeforeIncomeTaxesMinorityInterestAndIncomeLossFromEquityMethodInvestments",
];
pub(crate) const INTEREST_EXPENSE: &[&str] = &[
    "InterestExpenseNonOperating",
    "InterestExpense",
    "InterestExpenseDebt",
];
pub(crate) const PRETAX_INCOME: &[&str] = &[
    "IncomeLossFromContinuingOperationsBeforeIncomeTaxesExtraordinaryItemsNoncontrollingInterest",
    "IncomeLossFromContinuingOperationsBeforeIncomeTaxes",
    "IncomeLossFromContinuingOperationsBeforeIncomeTaxesMinorityInterestAndIncomeLossFromEquityMethodInvestments",
];
pub(crate) const TAX_EXPENSE: &[&str] =
    &["IncomeTaxExpenseBenefit", "CurrentIncomeTaxExpenseBenefit"];
pub(crate) const NET_INCOME: &[&str] = &[
    "NetIncomeLoss",
    "ProfitLoss",
    "NetIncomeLossAvailableToCommonStockholdersBasic",
];
pub(crate) const EPS_BASIC: &[&str] = &["EarningsPerShareBasic"];
pub(crate) const EPS_DILUTED: &[&str] = &["EarningsPerShareDiluted"];
pub(crate) const SHARES_BASIC: &[&str] = &[
    "WeightedAverageNumberOfSharesOutstandingBasic",
    "WeightedAverageNumberOfShareOutstandingBasicAndDiluted",
];
pub(crate) const SHARES_DILUTED: &[&str] = &[
    "WeightedAverageNumberOfDilutedSharesOutstanding",
    "WeightedAverageNumberOfShareOutstandingBasicAndDiluted",
];
pub(crate) const COMPREHENSIVE_INCOME: &[&str] = &["ComprehensiveIncomeNetOfTax"];

pub(crate) const CASH: &[&str] = &["CashAndCashEquivalentsAtCarryingValue", "Cash"];
pub(crate) const MARKETABLE_SECURITIES_CURRENT: &[&str] = &[
    "MarketableSecuritiesCurrent",
    "ShortTermInvestments",
    "AvailableForSaleSecuritiesCurrent",
];
pub(crate) const ACCOUNTS_RECEIVABLE: &[&str] = &[
    "AccountsReceivableNetCurrent",
    "ReceivablesNetCurrent",
    "AccountsNotesAndLoansReceivableNetCurrent",
];
pub(crate) const INVENTORY: &[&str] = &["InventoryNet", "InventoryFinishedGoodsNetOfReserves"];
pub(crate) const CURRENT_ASSETS: &[&str] = &["AssetsCurrent"];
pub(crate) const PPE: &[&str] = &[
    "PropertyPlantAndEquipmentNet",
    "PropertyPlantAndEquipmentAndFinanceLeaseRightOfUseAssetAfterAccumulatedDepreciationAndAmortization",
];
pub(crate) const GOODWILL: &[&str] = &["Goodwill"];
pub(crate) const INTANGIBLES: &[&str] = &[
    "FiniteLivedIntangibleAssetsNet",
    "IntangibleAssetsNetExcludingGoodwill",
];
pub(crate) const OPERATING_LEASE_ASSETS: &[&str] = &["OperatingLeaseRightOfUseAsset"];
pub(crate) const TOTAL_ASSETS: &[&str] = &["Assets"];
pub(crate) const ACCOUNTS_PAYABLE: &[&str] = &[
    "AccountsPayableCurrent",
    "AccountsPayableAndAccruedLiabilitiesCurrent",
];
pub(crate) const CURRENT_LIABILITIES: &[&str] = &["LiabilitiesCurrent"];
pub(crate) const DEBT_CURRENT: &[&str] = &[
    "ShortTermBorrowings",
    "ShortTermDebtCurrent",
    "LongTermDebtCurrent",
    "CurrentPortionOfLongTermDebt",
];
pub(crate) const LONG_TERM_DEBT: &[&str] = &["LongTermDebtNoncurrent", "LongTermDebt"];
pub(crate) const OPERATING_LEASE_LIABILITIES: &[&str] = &[
    "OperatingLeaseLiability",
    "OperatingLeaseLiabilityCurrent",
    "OperatingLeaseLiabilityNoncurrent",
];
pub(crate) const TOTAL_LIABILITIES: &[&str] = &["Liabilities"];
pub(crate) const STOCKHOLDERS_EQUITY: &[&str] = &[
    "StockholdersEquity",
    "StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest",
    "PartnersCapital",
];
pub(crate) const LIABILITIES_AND_EQUITY: &[&str] = &[
    "LiabilitiesAndStockholdersEquity",
    "LiabilitiesAndPartnersCapital",
];

pub(crate) const DEPRECIATION_AMORTIZATION: &[&str] = &[
    "DepreciationDepletionAndAmortization",
    "DepreciationDepletionAndAmortizationExpense",
    "DepreciationAndAmortization",
];
pub(crate) const STOCK_BASED_COMPENSATION: &[&str] = &[
    "ShareBasedCompensation",
    "ShareBasedCompensationArrangementByShareBasedPaymentAwardExpense",
];
pub(crate) const CHANGE_RECEIVABLES: &[&str] = &[
    "IncreaseDecreaseInAccountsReceivable",
    "IncreaseDecreaseInReceivables",
];
pub(crate) const CHANGE_INVENTORY: &[&str] = &["IncreaseDecreaseInInventories"];
pub(crate) const CHANGE_PAYABLES: &[&str] = &[
    "IncreaseDecreaseInAccountsPayable",
    "IncreaseDecreaseInAccountsPayableAndAccruedLiabilities",
];
pub(crate) const OPERATING_CASH_FLOW: &[&str] = &[
    "NetCashProvidedByUsedInOperatingActivities",
    "NetCashProvidedByUsedInOperatingActivitiesContinuingOperations",
];
pub(crate) const CAPEX: &[&str] = &[
    "PaymentsToAcquirePropertyPlantAndEquipment",
    "PaymentsToAcquireProductiveAssets",
];
pub(crate) const ACQUISITIONS: &[&str] = &[
    "PaymentsToAcquireBusinessesNetOfCashAcquired",
    "PaymentsToAcquireBusinessesGross",
];
pub(crate) const INVESTING_CASH_FLOW: &[&str] = &["NetCashProvidedByUsedInInvestingActivities"];
pub(crate) const DIVIDENDS_PAID: &[&str] =
    &["PaymentsOfDividends", "PaymentsOfDividendsCommonStock"];
pub(crate) const SHARE_REPURCHASES: &[&str] = &[
    "PaymentsForRepurchaseOfCommonStock",
    "PaymentsForRepurchaseOfEquity",
];
pub(crate) const DEBT_ISSUANCE: &[&str] = &[
    "ProceedsFromIssuanceOfLongTermDebt",
    "ProceedsFromBorrowings",
];
pub(crate) const DEBT_REPAYMENT: &[&str] = &[
    "RepaymentsOfLongTermDebt",
    "RepaymentsOfDebt",
    "PaymentsForRepurchaseOfDebt",
];
pub(crate) const FINANCING_CASH_FLOW: &[&str] = &["NetCashProvidedByUsedInFinancingActivities"];
pub(crate) const CASH_CHANGE: &[&str] = &[
    "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseIncludingExchangeRateEffect",
    "CashAndCashEquivalentsPeriodIncreaseDecrease",
];
pub(crate) const ENDING_CASH: &[&str] = &[
    "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
    "CashAndCashEquivalentsAtCarryingValue",
];

pub(crate) fn aliases_for(query: &str) -> Option<&'static [&'static str]> {
    match normalize_alias(query).as_str() {
        "revenue" | "revenues" | "sales" => Some(REVENUE),
        "costofrevenue" | "costofsales" => Some(COST_OF_REVENUE),
        "grossprofit" => Some(GROSS_PROFIT),
        "researchanddevelopment" | "rd" => Some(RESEARCH_DEVELOPMENT),
        "sga" | "sellinggeneralandadministrative" => Some(SELLING_GENERAL_ADMIN),
        "operatingexpenses" => Some(OPERATING_EXPENSES),
        "operatingincome" => Some(OPERATING_INCOME),
        "interestexpense" => Some(INTEREST_EXPENSE),
        "pretaxincome" | "incomebeforetax" => Some(PRETAX_INCOME),
        "tax" | "incometax" => Some(TAX_EXPENSE),
        "netincome" | "profit" => Some(NET_INCOME),
        "assets" | "totalassets" => Some(TOTAL_ASSETS),
        "cash" => Some(CASH),
        "inventory" => Some(INVENTORY),
        "equity" | "stockholdersequity" => Some(STOCKHOLDERS_EQUITY),
        "debt" | "longtermdebt" => Some(LONG_TERM_DEBT),
        "operatingcashflow" | "ocf" => Some(OPERATING_CASH_FLOW),
        "capex" | "capitalexpenditures" => Some(CAPEX),
        "freecashflow" => Some(OPERATING_CASH_FLOW),
        _ => None,
    }
}

pub(crate) fn concept_matches_alias(query: &str, concept: &str) -> bool {
    aliases_for(query).is_some_and(|aliases| {
        aliases
            .iter()
            .any(|alias| alias.eq_ignore_ascii_case(concept))
    })
}

fn normalize_alias(value: &str) -> String {
    value
        .rsplit(':')
        .next()
        .unwrap_or(value)
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_common_financial_aliases() {
        assert!(concept_matches_alias(
            "revenue",
            "RevenueFromContractWithCustomerIncludingAssessedTax"
        ));
        assert!(concept_matches_alias(
            "capex",
            "PaymentsToAcquireProductiveAssets"
        ));
        assert!(!concept_matches_alias("cash", "NetIncomeLoss"));
    }
}
