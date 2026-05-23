use anyhow::Result;
use chrono::Utc;
use sec_cli::sec::{
    CompanyReportQuery, DailyIndexQuery, EftsSearchQuery, EightKExhibitQuery, ForeignIssuerQuery,
    FundDisclosureQuery, HtmlTableQuery, InlineXbrlQuery, ProspectusQuery, ProxyQuery, SecClient,
    daily::latest_sec_index_date,
    efts::{parse_forms, require_query},
    print_records,
};

use super::{
    args::{EightKExhibitsArgs, InlineXbrlArgs, ProxyArgs, TablesArgs},
    disclosure_args::{CompanyReportArgs, ForeignArgs, FundArgs, ProspectusArgs},
    monitoring_args::{DailyArgs, EftsArgs},
    runner::{output_mode, resolve_cik},
};

pub(super) async fn daily(client: &SecClient, args: DailyArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let date = args
        .date
        .unwrap_or_else(|| latest_sec_index_date(Utc::now().date_naive()));
    let records = client
        .daily_filings(DailyIndexQuery {
            date,
            form: args.form,
            company: args.company,
            limit: Some(args.limit),
            include_amends: args.include_amends,
        })
        .await?;
    print_records(&records, output)
}

pub(super) async fn efts(client: &SecClient, args: EftsArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_optional_cik(client, args.ticker.as_deref(), args.cik).await?;
    let records = client
        .efts_search(EftsSearchQuery {
            query: require_query(&args.query)?,
            ciks: cik.into_iter().collect(),
            forms: parse_forms(&args.form),
            from: args.from,
            to: args.to,
            limit: Some(args.limit),
        })
        .await?;
    print_records(&records, output)
}

async fn resolve_optional_cik(
    client: &SecClient,
    ticker: Option<&str>,
    cik: Option<u64>,
) -> Result<Option<u64>> {
    match (ticker, cik) {
        (Some(ticker), None) => Ok(Some(client.cik_for_ticker(ticker).await?)),
        (None, Some(cik)) => Ok(Some(cik)),
        (None, None) => Ok(None),
        (Some(_), Some(_)) => anyhow::bail!("provide either --ticker or --cik, not both"),
    }
}

pub(super) async fn ixbrl(client: &SecClient, args: InlineXbrlArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let records = client
        .inline_xbrl_facts(InlineXbrlQuery {
            cik,
            form: Some(args.form),
            latest: args.latest,
            include_amends: args.include_amends,
            concept: args.concept,
            limit: Some(args.limit),
        })
        .await?;
    print_records(&records, output)
}

pub(super) async fn tables(client: &SecClient, args: TablesArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let records = client
        .html_tables(HtmlTableQuery {
            cik,
            form: Some(args.form),
            latest: args.latest,
            include_amends: args.include_amends,
            limit_tables: Some(args.limit_tables),
            limit_rows: Some(args.limit_rows),
        })
        .await?;
    print_records(&records, output)
}

pub(super) async fn company_report(client: &SecClient, args: CompanyReportArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let records = client
        .company_reports(CompanyReportQuery {
            cik,
            form: Some(args.form),
            latest: args.latest,
            include_amends: args.include_amends,
            topic: args.topic,
            limit_tables: Some(args.limit_tables),
            limit_rows: Some(args.limit_rows),
        })
        .await?;
    print_records(&records, output)
}

pub(super) async fn eightk_exhibits(client: &SecClient, args: EightKExhibitsArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let mut records = client
        .eightk_exhibits(EightKExhibitQuery {
            cik,
            latest: args.latest,
            include_amends: args.include_amends,
            category: args.category,
            limit_bytes: args.limit_bytes,
        })
        .await?;
    if let Some(limit) = args.limit {
        records.truncate(limit);
    }
    print_records(&records, output)
}

pub(super) async fn proxy(client: &SecClient, args: ProxyArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let records = client
        .proxy_statements(ProxyQuery {
            cik,
            latest: args.latest,
            include_amends: args.include_amends,
            limit_rows: Some(args.limit_rows),
        })
        .await?;
    print_records(&records, output)
}

pub(super) async fn prospectus(client: &SecClient, args: ProspectusArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let records = client
        .prospectuses(ProspectusQuery {
            cik,
            form: Some(args.form),
            latest: args.latest,
            include_amends: args.include_amends,
            limit_bytes: Some(args.limit_bytes),
            limit_tables: Some(args.limit_tables),
            limit_rows: Some(args.limit_rows),
        })
        .await?;
    print_records(&records, output)
}

pub(super) async fn foreign(client: &SecClient, args: ForeignArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let records = client
        .foreign_issuer_reports(ForeignIssuerQuery {
            cik,
            form: Some(args.form),
            latest: args.latest,
            include_amends: args.include_amends,
            limit_bytes: Some(args.limit_bytes),
        })
        .await?;
    print_records(&records, output)
}

pub(super) async fn fund(client: &SecClient, args: FundArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let records = client
        .fund_disclosures(FundDisclosureQuery {
            cik,
            form: Some(args.form),
            latest: args.latest,
            include_amends: args.include_amends,
            limit_holdings: Some(args.limit_holdings),
            limit_bytes: Some(args.limit_bytes),
        })
        .await?;
    print_records(&records, output)
}
