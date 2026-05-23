use std::sync::OnceLock;

use anyhow::{Context, Result, bail};
use sec_cli::sec::{
    OutputMode, SecClient,
    resolve::{resolve_verified_13f_cik, resolve_verified_13f_manager},
};

use super::analysis_args::StatementPeriodArg;

static OUTPUT_OVERRIDE: OnceLock<Option<OutputMode>> = OnceLock::new();

pub(super) fn set_output_override(mode: Option<OutputMode>) {
    let _ = OUTPUT_OVERRIDE.set(mode);
}

pub(super) fn output_mode(jsonl: bool, pretty: bool) -> OutputMode {
    if let Some(Some(mode)) = OUTPUT_OVERRIDE.get() {
        return *mode;
    }
    if jsonl {
        OutputMode::JsonLines
    } else if pretty {
        OutputMode::PrettyJson
    } else {
        OutputMode::Json
    }
}

pub(super) async fn resolve_cik(
    client: &SecClient,
    ticker: Option<&str>,
    cik: Option<u64>,
) -> Result<u64> {
    if let Some(cik) = cik {
        return Ok(cik);
    }
    if let Some(ticker) = ticker {
        return client
            .cik_for_ticker(ticker)
            .await
            .with_context(|| format!("unknown ticker '{}'", ticker));
    }
    bail!("provide --ticker or --cik");
}

pub(super) async fn resolve_subject(
    client: &SecClient,
    ticker: Option<&str>,
    cik: Option<u64>,
    investor: Option<&str>,
    manager: Option<&str>,
) -> Result<(u64, String)> {
    if let Some(investor) = investor {
        return resolve_verified_13f_cik(client, investor).await;
    }
    if let Some(manager) = manager {
        return resolve_verified_13f_manager(client, manager).await;
    }
    if let Some(cik) = cik {
        return Ok((cik, cik.to_string()));
    }
    if let Some(ticker) = ticker {
        let cik = client
            .cik_for_ticker(ticker)
            .await
            .with_context(|| format!("unknown ticker '{}'", ticker))?;
        return Ok((cik, ticker.to_ascii_uppercase()));
    }
    bail!("provide --ticker, --cik, --manager, or --investor");
}

pub(super) fn statement_period_form(period: StatementPeriodArg) -> Option<String> {
    match period {
        StatementPeriodArg::Annual => Some("10-K".to_string()),
        StatementPeriodArg::Quarterly => Some("10-Q".to_string()),
        StatementPeriodArg::All => None,
    }
}
