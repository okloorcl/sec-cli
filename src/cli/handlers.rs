use anyhow::{Context, Result};
use chrono::Utc;
use sec_cli::sec::{
    ArchiveQuery, CompanyReportQuery, DailyIndexQuery, EftsSearchQuery, EightKExhibitQuery,
    ExportFormat, FactQuery, FilingQuery, ForeignIssuerQuery, FundDisclosureQuery,
    HealthScoreQuery, HtmlTableQuery, InlineXbrlQuery, MetricsQuery, ProspectusQuery, ProxyQuery,
    SecClient, StatementQuery, StatementStitchQuery, XbrlCalculationQuery, XbrlLinkbaseQuery,
    XbrlStatementQuery, XbrlTreeQuery,
    daily::latest_sec_index_date,
    efts::{parse_forms, require_query},
    export_records, print_records,
};

use super::{
    analysis_args::{
        MetricsArgs, ScoresArgs, StatementsArgs, StitchArgs, XbrlCalcArgs, XbrlLinkbaseArgs,
        XbrlStatementArgs, XbrlTreeArgs,
    },
    archive_args::ArchiveArgs,
    args::{EightKExhibitsArgs, InlineXbrlArgs, TablesArgs},
    common::{output_mode, resolve_cik, statement_period_form},
    disclosure_args::{CompanyReportArgs, ForeignArgs, FundArgs, ProspectusArgs, ProxyArgs},
    export_args::{ExportArgs, ExportFormatArg, ExportKindArg},
    monitoring_args::{DailyArgs, EftsArgs},
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

pub(super) async fn statements(client: &SecClient, args: StatementsArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let records = client
        .financial_statements(StatementQuery {
            cik,
            statement: args.statement,
            form: statement_period_form(args.period),
            unit: args.unit,
            latest: args.latest,
        })
        .await?;
    print_records(&records, output)
}

pub(super) async fn stitch(client: &SecClient, args: StitchArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let records = client
        .stitched_statements(StatementStitchQuery {
            cik,
            statement: args.statement,
            unit: args.unit,
            latest: args.latest,
        })
        .await?;
    print_records(&records, output)
}

pub(super) async fn metrics(client: &SecClient, args: MetricsArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let records = client
        .financial_metrics(MetricsQuery {
            cik,
            form: statement_period_form(args.period),
            unit: args.unit,
            latest: args.latest,
        })
        .await?;
    print_records(&records, output)
}

pub(super) async fn scores(client: &SecClient, args: ScoresArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let records = client
        .health_scores(HealthScoreQuery {
            cik,
            form: statement_period_form(args.period),
            unit: args.unit,
            latest: args.latest,
        })
        .await?;
    print_records(&records, output)
}

pub(super) async fn export(client: &SecClient, args: ExportArgs) -> Result<()> {
    let format = match args.format {
        ExportFormatArg::Arrow => ExportFormat::Arrow,
        ExportFormatArg::Parquet => ExportFormat::Parquet,
    };
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let count = match args.kind {
        ExportKindArg::Filings => {
            let records = client
                .filings(FilingQuery {
                    cik,
                    form: args.form,
                    latest: args.latest,
                    from: None,
                    to: None,
                    include_amends: args.include_amends,
                })
                .await?;
            export_records(&records, format, &args.out)?
        }
        ExportKindArg::Facts => {
            let concept = args
                .concept
                .context("--concept is required for --kind facts")?;
            let records = client
                .facts(FactQuery {
                    cik,
                    concept,
                    form: args.form,
                    unit: args.unit,
                    latest: args.latest,
                })
                .await?;
            export_records(&records, format, &args.out)?
        }
        ExportKindArg::Statements => {
            let records = client
                .financial_statements(StatementQuery {
                    cik,
                    statement: args.statement,
                    form: statement_period_form(args.period),
                    unit: args.unit,
                    latest: args.latest,
                })
                .await?;
            export_records(&records, format, &args.out)?
        }
        ExportKindArg::Stitch => {
            let records = client
                .stitched_statements(StatementStitchQuery {
                    cik,
                    statement: args.statement,
                    unit: args.unit,
                    latest: args.latest,
                })
                .await?;
            export_records(&records, format, &args.out)?
        }
        ExportKindArg::Metrics => {
            let records = client
                .financial_metrics(MetricsQuery {
                    cik,
                    form: statement_period_form(args.period),
                    unit: args.unit,
                    latest: args.latest,
                })
                .await?;
            export_records(&records, format, &args.out)?
        }
        ExportKindArg::Scores => {
            let records = client
                .health_scores(HealthScoreQuery {
                    cik,
                    form: statement_period_form(args.period),
                    unit: args.unit,
                    latest: args.latest,
                })
                .await?;
            export_records(&records, format, &args.out)?
        }
    };
    eprintln!("wrote {count} records to {}", args.out.display());
    Ok(())
}

pub(super) async fn archive(client: &SecClient, args: ArchiveArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let manifest = client
        .archive_filings(ArchiveQuery {
            cik,
            form: args.form,
            latest: args.latest,
            include_amends: args.include_amends,
            primary_only: args.primary_only,
            limit_bytes: args.limit_bytes,
            out_dir: args.out_dir,
        })
        .await?;
    print_records(&[manifest], output)
}

pub(super) async fn xbrl_links(client: &SecClient, args: XbrlLinkbaseArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let records = client
        .xbrl_linkbases(XbrlLinkbaseQuery {
            cik,
            form: Some(args.form),
            latest: args.latest,
            include_amends: args.include_amends,
            linkbase: args.linkbase,
            role: args.role,
            concept: args.concept,
            limit: Some(args.limit),
        })
        .await?;
    print_records(&records, output)
}

pub(super) async fn xbrl_tree(client: &SecClient, args: XbrlTreeArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let records = client
        .xbrl_presentation_tree(XbrlTreeQuery {
            cik,
            form: Some(args.form),
            latest: args.latest,
            include_amends: args.include_amends,
            role: args.role,
            concept: args.concept,
            limit: Some(args.limit),
        })
        .await?;
    print_records(&records, output)
}

pub(super) async fn xbrl_calc(client: &SecClient, args: XbrlCalcArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let records = client
        .xbrl_calculation_checks(XbrlCalculationQuery {
            cik,
            form: Some(args.form),
            latest: args.latest,
            include_amends: args.include_amends,
            role: args.role,
            concept: args.concept,
            unit: Some(args.unit),
            tolerance: args.tolerance,
            limit: Some(args.limit),
        })
        .await?;
    print_records(&records, output)
}

pub(super) async fn xbrl_statement(client: &SecClient, args: XbrlStatementArgs) -> Result<()> {
    let output = output_mode(args.jsonl, args.pretty);
    let cik = resolve_cik(client, args.ticker.as_deref(), args.cik).await?;
    let records = client
        .xbrl_statement(XbrlStatementQuery {
            cik,
            form: Some(args.form),
            latest: args.latest,
            include_amends: args.include_amends,
            role: args.role,
            concept: args.concept,
            unit: Some(args.unit),
            tolerance: args.tolerance,
            values_only: args.values_only,
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
