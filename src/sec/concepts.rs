pub(crate) const REVENUE: &[&str] = &[
    "RevenueFromContractWithCustomerExcludingAssessedTax",
    "RevenueFromContractWithCustomerIncludingAssessedTax",
    "Revenues",
    "SalesRevenueNet",
    "SalesRevenueGoodsNet",
    "SalesRevenueServicesNet",
    "InterestAndDividendIncomeOperating",
    "RealEstateRevenueNet",
    "RegulatedAndUnregulatedOperatingRevenue",
    "AdvertisingRevenue",
    "SubscriptionRevenue",
    "PremiumsEarnedNet",
];
pub(crate) const COST_OF_REVENUE: &[&str] = &[
    "CostOfRevenue",
    "CostOfGoodsAndServicesSold",
    "CostOfGoodsSold",
    "CostOfGoodsAndServiceExcludingDepreciationDepletionAndAmortization",
    "CostOfRevenueExcludingDepreciationDepletionAndAmortization",
    "CostOfGoodsSoldExcludingDepreciationDepletionAndAmortization",
    "CostOfServicesRevenue",
    "CostOfGoodsRevenue",
    "PolicyholderBenefitsAndClaimsIncurredNet",
];
pub(crate) const GROSS_PROFIT: &[&str] = &["GrossProfit"];
pub(crate) const RESEARCH_DEVELOPMENT: &[&str] = &[
    "ResearchAndDevelopmentExpense",
    "ResearchAndDevelopmentExpenseExcludingAcquiredInProcessCost",
    "ResearchAndDevelopmentExpenseSoftwareExcludingAcquiredInProcessCost",
];
pub(crate) const SELLING_GENERAL_ADMIN: &[&str] = &[
    "SellingGeneralAndAdministrativeExpense",
    "GeneralAndAdministrativeExpense",
    "SellingAndMarketingExpense",
    "SellingExpense",
    "MarketingExpense",
    "AdministrativeExpense",
];
pub(crate) const OPERATING_EXPENSES: &[&str] = &[
    "OperatingExpenses",
    "CostsAndExpenses",
    "NoninterestExpense",
    "OperatingCostsAndExpenses",
    "OtherOperatingIncomeExpenseNet",
];
pub(crate) const OPERATING_INCOME: &[&str] = &[
    "OperatingIncomeLoss",
    "IncomeLossFromContinuingOperationsBeforeIncomeTaxesMinorityInterestAndIncomeLossFromEquityMethodInvestments",
];
pub(crate) const INTEREST_EXPENSE: &[&str] = &[
    "InterestExpenseNonOperating",
    "InterestExpense",
    "InterestExpenseDebt",
    "InterestAndDebtExpense",
    "InterestExpenseBorrowings",
];
pub(crate) const PRETAX_INCOME: &[&str] = &[
    "IncomeLossFromContinuingOperationsBeforeIncomeTaxesExtraordinaryItemsNoncontrollingInterest",
    "IncomeLossFromContinuingOperationsBeforeIncomeTaxes",
    "IncomeLossFromContinuingOperationsBeforeIncomeTaxesMinorityInterestAndIncomeLossFromEquityMethodInvestments",
];
pub(crate) const TAX_EXPENSE: &[&str] = &[
    "IncomeTaxExpenseBenefit",
    "CurrentIncomeTaxExpenseBenefit",
    "DeferredTaxExpenseBenefit",
];
pub(crate) const NET_INCOME: &[&str] = &[
    "NetIncomeLoss",
    "ProfitLoss",
    "NetIncomeLossAvailableToCommonStockholdersBasic",
    "IncomeLossFromContinuingOperations",
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

pub(crate) const CASH: &[&str] = &[
    "CashAndCashEquivalentsAtCarryingValue",
    "Cash",
    "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
    "CashAndDueFromBanks",
];
pub(crate) const MARKETABLE_SECURITIES_CURRENT: &[&str] = &[
    "MarketableSecuritiesCurrent",
    "ShortTermInvestments",
    "AvailableForSaleSecuritiesCurrent",
    "AvailableForSaleSecuritiesDebtSecuritiesCurrent",
    "TradingSecuritiesCurrent",
];
pub(crate) const ACCOUNTS_RECEIVABLE: &[&str] = &[
    "AccountsReceivableNetCurrent",
    "ReceivablesNetCurrent",
    "AccountsNotesAndLoansReceivableNetCurrent",
    "AccountsReceivableNet",
    "TradeAccountsReceivableNetCurrent",
    "ContractWithCustomerAssetNetCurrent",
];
pub(crate) const INVENTORY: &[&str] = &[
    "InventoryNet",
    "InventoryFinishedGoodsNetOfReserves",
    "InventoryRawMaterialsAndSupplies",
    "InventoryWorkInProcessAndRawMaterialsNetOfReserves",
    "FinishedGoodsInventoryNet",
];
pub(crate) const CURRENT_ASSETS: &[&str] = &["AssetsCurrent"];
pub(crate) const PPE: &[&str] = &[
    "PropertyPlantAndEquipmentNet",
    "PropertyPlantAndEquipmentAndFinanceLeaseRightOfUseAssetAfterAccumulatedDepreciationAndAmortization",
    "PropertyPlantAndEquipmentGross",
    "PropertyPlantAndEquipmentNetIncludingFinanceLeaseRightOfUseAsset",
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
    "AccountsPayableTradeCurrent",
    "AccruedLiabilitiesCurrent",
];
pub(crate) const CURRENT_LIABILITIES: &[&str] = &[
    "LiabilitiesCurrent",
    "AccountsPayableAccruedLiabilitiesCurrent",
    "ContractWithCustomerLiabilityCurrent",
];
pub(crate) const DEBT_CURRENT: &[&str] = &[
    "ShortTermBorrowings",
    "ShortTermDebtCurrent",
    "LongTermDebtCurrent",
    "CurrentPortionOfLongTermDebt",
    "DebtCurrent",
    "FinanceLeaseLiabilityCurrent",
];
pub(crate) const LONG_TERM_DEBT: &[&str] = &[
    "LongTermDebtNoncurrent",
    "LongTermDebt",
    "LongTermDebtAndFinanceLeaseObligations",
    "FinanceLeaseLiabilityNoncurrent",
];
pub(crate) const OPERATING_LEASE_LIABILITIES: &[&str] = &[
    "OperatingLeaseLiability",
    "OperatingLeaseLiabilityCurrent",
    "OperatingLeaseLiabilityNoncurrent",
];
pub(crate) const TOTAL_LIABILITIES: &[&str] = &[
    "Liabilities",
    "LiabilitiesAndTemporaryEquity",
    "LiabilitiesCurrentAndNoncurrent",
];
pub(crate) const STOCKHOLDERS_EQUITY: &[&str] = &[
    "StockholdersEquity",
    "StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest",
    "PartnersCapital",
    "CommonStocksIncludingAdditionalPaidInCapital",
    "AccumulatedOtherComprehensiveIncomeLossNetOfTax",
];
pub(crate) const RETAINED_EARNINGS: &[&str] = &[
    "RetainedEarningsAccumulatedDeficit",
    "RetainedEarnings",
    "AccumulatedDeficit",
];
pub(crate) const LIABILITIES_AND_EQUITY: &[&str] = &[
    "LiabilitiesAndStockholdersEquity",
    "LiabilitiesAndPartnersCapital",
];

pub(crate) const DEPRECIATION_AMORTIZATION: &[&str] = &[
    "DepreciationDepletionAndAmortization",
    "DepreciationDepletionAndAmortizationExpense",
    "DepreciationAndAmortization",
    "Depreciation",
    "AmortizationOfIntangibleAssets",
];
pub(crate) const STOCK_BASED_COMPENSATION: &[&str] = &[
    "ShareBasedCompensation",
    "ShareBasedCompensationArrangementByShareBasedPaymentAwardExpense",
    "EmployeeServiceShareBasedCompensationNonvestedAwardsTotalCompensationCostNotYetRecognized",
];
pub(crate) const CHANGE_RECEIVABLES: &[&str] = &[
    "IncreaseDecreaseInAccountsReceivable",
    "IncreaseDecreaseInReceivables",
    "IncreaseDecreaseInOperatingAssets",
];
pub(crate) const CHANGE_INVENTORY: &[&str] = &[
    "IncreaseDecreaseInInventories",
    "IncreaseDecreaseInInventory",
];
pub(crate) const CHANGE_PAYABLES: &[&str] = &[
    "IncreaseDecreaseInAccountsPayable",
    "IncreaseDecreaseInAccountsPayableAndAccruedLiabilities",
    "IncreaseDecreaseInOperatingLiabilities",
];
pub(crate) const OPERATING_CASH_FLOW: &[&str] = &[
    "NetCashProvidedByUsedInOperatingActivities",
    "NetCashProvidedByUsedInOperatingActivitiesContinuingOperations",
    "NetCashProvidedByUsedInOperatingActivitiesContinuingAndDiscontinuedOperations",
];
pub(crate) const CAPEX: &[&str] = &[
    "PaymentsToAcquirePropertyPlantAndEquipment",
    "PaymentsToAcquireProductiveAssets",
    "PaymentsToAcquirePropertyPlantAndEquipmentAndIntangibleAssets",
    "CapitalExpendituresIncurredButNotYetPaid",
];
pub(crate) const ACQUISITIONS: &[&str] = &[
    "PaymentsToAcquireBusinessesNetOfCashAcquired",
    "PaymentsToAcquireBusinessesGross",
];
pub(crate) const INVESTING_CASH_FLOW: &[&str] = &["NetCashProvidedByUsedInInvestingActivities"];
pub(crate) const DIVIDENDS_PAID: &[&str] = &[
    "PaymentsOfDividends",
    "PaymentsOfDividendsCommonStock",
    "PaymentsOfOrdinaryDividends",
];
pub(crate) const SHARE_REPURCHASES: &[&str] = &[
    "PaymentsForRepurchaseOfCommonStock",
    "PaymentsForRepurchaseOfEquity",
    "PaymentsForRepurchaseOfPreferredStockAndPreferenceStock",
];
pub(crate) const DEBT_ISSUANCE: &[&str] = &[
    "ProceedsFromIssuanceOfLongTermDebt",
    "ProceedsFromBorrowings",
    "ProceedsFromIssuanceOfDebt",
    "ProceedsFromShortTermDebt",
];
pub(crate) const DEBT_REPAYMENT: &[&str] = &[
    "RepaymentsOfLongTermDebt",
    "RepaymentsOfDebt",
    "PaymentsForRepurchaseOfDebt",
    "RepaymentsOfShortTermDebt",
];
pub(crate) const FINANCING_CASH_FLOW: &[&str] = &["NetCashProvidedByUsedInFinancingActivities"];
pub(crate) const CASH_CHANGE: &[&str] = &[
    "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseIncludingExchangeRateEffect",
    "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseExcludingExchangeRateEffect",
    "CashAndCashEquivalentsPeriodIncreaseDecrease",
];
pub(crate) const ENDING_CASH: &[&str] = &[
    "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
    "CashAndCashEquivalentsAtCarryingValue",
    "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsIncludingDisposalGroupAndDiscontinuedOperations",
];

pub(crate) fn aliases_for(query: &str) -> Option<&'static [&'static str]> {
    match normalize_alias(query).as_str() {
        "revenue" | "revenues" | "sales" => Some(REVENUE),
        "costofrevenue" | "costofsales" | "cogs" => Some(COST_OF_REVENUE),
        "grossprofit" => Some(GROSS_PROFIT),
        "researchanddevelopment" | "rd" | "r&d" => Some(RESEARCH_DEVELOPMENT),
        "sga" | "sg&a" | "sellinggeneralandadministrative" => Some(SELLING_GENERAL_ADMIN),
        "operatingexpenses" => Some(OPERATING_EXPENSES),
        "operatingincome" => Some(OPERATING_INCOME),
        "interestexpense" => Some(INTEREST_EXPENSE),
        "pretaxincome" | "incomebeforetax" => Some(PRETAX_INCOME),
        "tax" | "incometax" => Some(TAX_EXPENSE),
        "netincome" | "profit" => Some(NET_INCOME),
        "assets" | "totalassets" => Some(TOTAL_ASSETS),
        "currentassets" => Some(CURRENT_ASSETS),
        "cash" => Some(CASH),
        "securities" | "shortterminvestments" => Some(MARKETABLE_SECURITIES_CURRENT),
        "receivables" | "accountsreceivable" => Some(ACCOUNTS_RECEIVABLE),
        "inventory" => Some(INVENTORY),
        "ppe" | "propertyplantandequipment" => Some(PPE),
        "equity" | "stockholdersequity" => Some(STOCKHOLDERS_EQUITY),
        "liabilities" | "totalliabilities" => Some(TOTAL_LIABILITIES),
        "currentliabilities" => Some(CURRENT_LIABILITIES),
        "retainedearnings" | "accumulateddeficit" => Some(RETAINED_EARNINGS),
        "debt" | "longtermdebt" => Some(LONG_TERM_DEBT),
        "currentdebt" | "shorttermdebt" => Some(DEBT_CURRENT),
        "operatingcashflow" | "ocf" => Some(OPERATING_CASH_FLOW),
        "capex" | "capitalexpenditures" => Some(CAPEX),
        "freecashflow" => Some(OPERATING_CASH_FLOW),
        _ => None,
    }
}

#[cfg(test)]
fn mapped_concept_count() -> usize {
    [
        REVENUE,
        COST_OF_REVENUE,
        GROSS_PROFIT,
        RESEARCH_DEVELOPMENT,
        SELLING_GENERAL_ADMIN,
        OPERATING_EXPENSES,
        OPERATING_INCOME,
        INTEREST_EXPENSE,
        PRETAX_INCOME,
        TAX_EXPENSE,
        NET_INCOME,
        EPS_BASIC,
        EPS_DILUTED,
        SHARES_BASIC,
        SHARES_DILUTED,
        COMPREHENSIVE_INCOME,
        CASH,
        MARKETABLE_SECURITIES_CURRENT,
        ACCOUNTS_RECEIVABLE,
        INVENTORY,
        CURRENT_ASSETS,
        PPE,
        GOODWILL,
        INTANGIBLES,
        OPERATING_LEASE_ASSETS,
        TOTAL_ASSETS,
        ACCOUNTS_PAYABLE,
        CURRENT_LIABILITIES,
        DEBT_CURRENT,
        LONG_TERM_DEBT,
        OPERATING_LEASE_LIABILITIES,
        TOTAL_LIABILITIES,
        STOCKHOLDERS_EQUITY,
        RETAINED_EARNINGS,
        LIABILITIES_AND_EQUITY,
        DEPRECIATION_AMORTIZATION,
        STOCK_BASED_COMPENSATION,
        CHANGE_RECEIVABLES,
        CHANGE_INVENTORY,
        CHANGE_PAYABLES,
        OPERATING_CASH_FLOW,
        CAPEX,
        ACQUISITIONS,
        INVESTING_CASH_FLOW,
        DIVIDENDS_PAID,
        SHARE_REPURCHASES,
        DEBT_ISSUANCE,
        DEBT_REPAYMENT,
        FINANCING_CASH_FLOW,
        CASH_CHANGE,
        ENDING_CASH,
    ]
    .iter()
    .map(|aliases| aliases.len())
    .sum()
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
        assert!(concept_matches_alias(
            "receivables",
            "TradeAccountsReceivableNetCurrent"
        ));
        assert!(!concept_matches_alias("cash", "NetIncomeLoss"));
    }

    #[test]
    fn covers_more_than_one_hundred_standard_concepts() {
        assert!(mapped_concept_count() >= 100);
    }
}
