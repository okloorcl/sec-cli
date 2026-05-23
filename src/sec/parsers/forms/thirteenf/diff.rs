use std::collections::BTreeMap;

use crate::sec::models::{ThirteenFAggregateHoldingRecord, ThirteenFDiffRecord};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DiffKey {
    cusip: Option<String>,
    class: Option<String>,
    put_call: Option<String>,
}

pub fn diff_holdings(
    current: Vec<ThirteenFAggregateHoldingRecord>,
    previous: Vec<ThirteenFAggregateHoldingRecord>,
) -> Vec<ThirteenFDiffRecord> {
    let current_accession = current.first().map(|row| row.accession.clone());
    let current_report_date = current.first().and_then(|row| row.report_date.clone());
    let current_source_url = current.first().map(|row| row.source_url.clone());
    let previous_accession = previous.first().map(|row| row.accession.clone());
    let mut current_map = index_by_key(current);
    let mut previous_map = index_by_key(previous);

    let mut records = Vec::new();
    for key in current_map.keys().cloned().collect::<Vec<_>>() {
        let current = current_map.remove(&key).expect("key from map");
        let previous = previous_map.remove(&key);
        records.push(diff_record(
            current,
            previous,
            previous_accession.as_deref().unwrap_or_default(),
        ));
    }

    for previous in previous_map.into_values() {
        records.push(exit_record(
            previous,
            current_accession.as_deref().unwrap_or_default(),
            current_report_date.clone(),
            current_source_url.as_deref().unwrap_or_default(),
        ));
    }

    records.sort_by(|a, b| {
        is_position_change(b)
            .cmp(&is_position_change(a))
            .then_with(|| b.change_value_usd.abs().cmp(&a.change_value_usd.abs()))
            .then_with(|| a.cusip.cmp(&b.cusip))
    });
    records
}

fn is_position_change(record: &ThirteenFDiffRecord) -> bool {
    record.change_type != "unchanged"
}

fn index_by_key(
    rows: Vec<ThirteenFAggregateHoldingRecord>,
) -> BTreeMap<DiffKey, ThirteenFAggregateHoldingRecord> {
    rows.into_iter()
        .map(|row| {
            (
                DiffKey {
                    cusip: row.cusip.clone(),
                    class: row.class.clone(),
                    put_call: row.put_call.clone(),
                },
                row,
            )
        })
        .collect()
}

fn diff_record(
    current: ThirteenFAggregateHoldingRecord,
    previous: Option<ThirteenFAggregateHoldingRecord>,
    previous_accession: &str,
) -> ThirteenFDiffRecord {
    let previous_value = previous
        .as_ref()
        .map(|row| row.value_usd)
        .unwrap_or_default();
    let previous_shares = previous.as_ref().map(|row| row.shares).unwrap_or_default();
    let change_value = current.value_usd as i128 - previous_value as i128;
    let change_shares = current.shares - previous_shares;
    let change_type = change_type(previous.is_some(), current.shares, previous_shares);

    ThirteenFDiffRecord {
        cik: current.cik,
        manager: current.manager.clone(),
        current_accession: current.accession.clone(),
        previous_accession: previous
            .as_ref()
            .map(|row| row.accession.clone())
            .unwrap_or_else(|| previous_accession.to_string()),
        current_report_date: current.report_date.clone(),
        previous_report_date: previous.as_ref().and_then(|row| row.report_date.clone()),
        issuer: current
            .issuer
            .clone()
            .or_else(|| previous.as_ref()?.issuer.clone()),
        class: current
            .class
            .clone()
            .or_else(|| previous.as_ref()?.class.clone()),
        cusip: current
            .cusip
            .clone()
            .or_else(|| previous.as_ref()?.cusip.clone()),
        put_call: current
            .put_call
            .clone()
            .or_else(|| previous.as_ref()?.put_call.clone()),
        change_type,
        current_value_usd: current.value_usd,
        previous_value_usd: previous_value,
        change_value_usd: change_value,
        current_shares: current.shares,
        previous_shares,
        change_shares,
        current_source_url: current.source_url.clone(),
        previous_source_url: previous
            .as_ref()
            .map(|row| row.source_url.clone())
            .unwrap_or_default(),
    }
}

fn exit_record(
    previous: ThirteenFAggregateHoldingRecord,
    current_accession: &str,
    current_report_date: Option<String>,
    current_source_url: &str,
) -> ThirteenFDiffRecord {
    ThirteenFDiffRecord {
        cik: previous.cik,
        manager: previous.manager.clone(),
        current_accession: current_accession.to_string(),
        previous_accession: previous.accession.clone(),
        current_report_date,
        previous_report_date: previous.report_date.clone(),
        issuer: previous.issuer.clone(),
        class: previous.class.clone(),
        cusip: previous.cusip.clone(),
        put_call: previous.put_call.clone(),
        change_type: "exited".to_string(),
        current_value_usd: 0,
        previous_value_usd: previous.value_usd,
        change_value_usd: -(previous.value_usd as i128),
        current_shares: 0.0,
        previous_shares: previous.shares,
        change_shares: -previous.shares,
        current_source_url: current_source_url.to_string(),
        previous_source_url: previous.source_url.clone(),
    }
}

fn change_type(has_previous: bool, current_shares: f64, previous_shares: f64) -> String {
    if !has_previous {
        return "new".to_string();
    }
    let diff = current_shares - previous_shares;
    if diff.abs() < f64::EPSILON {
        "unchanged"
    } else if diff > 0.0 {
        "increased"
    } else {
        "reduced"
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diffs_new_reduced_and_exited_holdings() {
        let records = diff_holdings(
            vec![sample("CUR", "A", 120, 12.0), sample("CUR", "B", 10, 1.0)],
            vec![sample("PREV", "A", 100, 10.0), sample("PREV", "C", 50, 5.0)],
        );

        assert_eq!(records.len(), 3);
        assert!(records.iter().any(|row| row.change_type == "increased"));
        assert!(records.iter().any(|row| row.change_type == "new"));
        assert!(records.iter().any(|row| row.change_type == "exited"));
    }

    fn sample(
        accession: &str,
        cusip: &str,
        value: u64,
        shares: f64,
    ) -> ThirteenFAggregateHoldingRecord {
        ThirteenFAggregateHoldingRecord {
            accession: accession.to_string(),
            cik: 1,
            manager: "Manager".to_string(),
            filing_date: "2026-01-01".to_string(),
            report_date: Some("2025-12-31".to_string()),
            issuer: Some(cusip.to_string()),
            class: Some("COM".to_string()),
            cusip: Some(cusip.to_string()),
            put_call: None,
            value_reported: value,
            value_scale: "usd".to_string(),
            value_usd: value,
            shares,
            voting_sole: 0,
            voting_shared: 0,
            voting_none: 0,
            rows: 1,
            source_url: "https://example.test".to_string(),
        }
    }
}
