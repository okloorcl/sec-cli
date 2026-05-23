use std::collections::BTreeMap;

use crate::sec::models::{ThirteenFAggregateHoldingRecord, ThirteenFHoldingRecord};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct HoldingKey {
    accession: String,
    cusip: Option<String>,
    class: Option<String>,
    put_call: Option<String>,
}

#[derive(Default)]
struct AggregateState {
    first: Option<ThirteenFAggregateHoldingRecord>,
}

pub fn aggregate_holdings(
    holdings: Vec<ThirteenFHoldingRecord>,
) -> Vec<ThirteenFAggregateHoldingRecord> {
    let mut groups: BTreeMap<HoldingKey, AggregateState> = BTreeMap::new();

    for holding in holdings {
        let key = HoldingKey {
            accession: holding.accession.clone(),
            cusip: holding.cusip.clone(),
            class: holding.class.clone(),
            put_call: holding.put_call.clone(),
        };

        let state = groups.entry(key).or_default();
        let row = state.first.get_or_insert_with(|| aggregate_from(&holding));
        row.value_reported = row
            .value_reported
            .saturating_add(holding.value_reported.unwrap_or_default());
        row.value_usd = row
            .value_usd
            .saturating_add(holding.value_usd.unwrap_or_default());
        row.shares += holding.shares.unwrap_or_default();
        row.voting_sole = row
            .voting_sole
            .saturating_add(holding.voting_sole.unwrap_or_default());
        row.voting_shared = row
            .voting_shared
            .saturating_add(holding.voting_shared.unwrap_or_default());
        row.voting_none = row
            .voting_none
            .saturating_add(holding.voting_none.unwrap_or_default());
        row.rows += 1;
    }

    let mut records: Vec<_> = groups
        .into_values()
        .filter_map(|state| state.first)
        .collect();
    records.sort_by(|a, b| {
        b.value_usd
            .cmp(&a.value_usd)
            .then_with(|| a.cusip.cmp(&b.cusip))
    });
    records
}

fn aggregate_from(holding: &ThirteenFHoldingRecord) -> ThirteenFAggregateHoldingRecord {
    ThirteenFAggregateHoldingRecord {
        accession: holding.accession.clone(),
        cik: holding.cik,
        manager: holding.manager.clone(),
        filing_date: holding.filing_date.clone(),
        report_date: holding.report_date.clone(),
        issuer: holding.issuer.clone(),
        class: holding.class.clone(),
        cusip: holding.cusip.clone(),
        put_call: holding.put_call.clone(),
        value_reported: 0,
        value_scale: holding.value_scale.clone(),
        value_usd: 0,
        shares: 0.0,
        voting_sole: 0,
        voting_shared: 0,
        voting_none: 0,
        rows: 0,
        source_url: holding.source_url.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_holdings_by_cusip_class_and_put_call() {
        let holdings = vec![
            sample_holding("A", 10, 100.0),
            sample_holding("A", 20, 200.0),
            sample_holding("B", 5, 50.0),
        ];

        let records = aggregate_holdings(holdings);

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].cusip.as_deref(), Some("A"));
        assert_eq!(records[0].value_usd, 30);
        assert_eq!(records[0].shares, 300.0);
        assert_eq!(records[0].rows, 2);
    }

    fn sample_holding(cusip: &str, value: u64, shares: f64) -> ThirteenFHoldingRecord {
        ThirteenFHoldingRecord {
            accession: "000".to_string(),
            cik: 1,
            manager: "Manager".to_string(),
            filing_date: "2026-01-01".to_string(),
            report_date: Some("2025-12-31".to_string()),
            issuer: Some(cusip.to_string()),
            class: Some("COM".to_string()),
            cusip: Some(cusip.to_string()),
            value_reported: Some(value),
            value_scale: "usd".to_string(),
            value_usd: Some(value),
            shares: Some(shares),
            share_type: Some("SH".to_string()),
            put_call: None,
            investment_discretion: None,
            other_manager: None,
            voting_sole: Some(1),
            voting_shared: Some(2),
            voting_none: Some(3),
            document: None,
            document_sequence: None,
            document_description: None,
            source_url: "https://example.test".to_string(),
        }
    }
}
