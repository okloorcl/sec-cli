use anyhow::{Context, Result, bail};
use clap::Parser;
use sec_cli::sec::documents::read::{content_for_terminal, validate_doc_args};
use sec_cli::sec::{
    DocumentQuery, DocumentReadQuery, EightKQuery, FactQuery, FilingQuery, Form4Query, OutputMode,
    ParseQuery, ReportKind, ReportQuery, Schedule13Query, SearchQuery, SecClient, SectionQuery,
    StatementQuery, ThirteenFQuery, accession_text_url, find_matches,
    llm::{LlmConfig, LlmProvider},
    print_records,
    resolve::{ResolveInput, resolve_verified_13f_cik, resolve_verified_13f_manager},
    supported_parsers,
};

use super::args::{Cli, Command, LlmProviderArg, ReportKindArg, ResolveArgs, StatementPeriodArg};
use super::handlers;
use super::identity::resolve_identity;

pub(crate) async fn run() -> Result<()> {
    let cli = Cli::parse();
    let identity = resolve_identity(cli.identity)?;

    let client = SecClient::new(identity, cli.cache_dir)?;

    match cli.command {
        Command::Filings(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let cik = resolve_cik(&client, args.ticker.as_deref(), args.cik).await?;
            let filings = client
                .filings(FilingQuery {
                    cik,
                    form: args.form,
                    latest: args.latest,
                    from: args.from,
                    to: args.to,
                    include_amends: args.include_amends,
                })
                .await?;
            print_records(&filings, output)?;
        }
        Command::Facts(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let cik = resolve_cik(&client, args.ticker.as_deref(), args.cik).await?;
            let facts = client
                .facts(FactQuery {
                    cik,
                    concept: args.concept,
                    form: args.form,
                    unit: args.unit,
                    latest: args.latest,
                })
                .await?;
            print_records(&facts, output)?;
        }
        Command::Statements(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let cik = resolve_cik(&client, args.ticker.as_deref(), args.cik).await?;
            let records = client
                .financial_statements(StatementQuery {
                    cik,
                    statement: args.statement,
                    form: statement_period_form(args.period),
                    unit: args.unit,
                    latest: args.latest,
                })
                .await?;
            print_records(&records, output)?;
        }
        Command::Ixbrl(args) => handlers::ixbrl(&client, args).await?,
        Command::Tables(args) => handlers::tables(&client, args).await?,
        Command::Proxy(args) => handlers::proxy(&client, args).await?,
        Command::Prospectus(args) => handlers::prospectus(&client, args).await?,
        Command::Foreign(args) => handlers::foreign(&client, args).await?,
        Command::Fund(args) => handlers::fund(&client, args).await?,
        Command::Search(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let cik = resolve_cik(&client, args.ticker.as_deref(), args.cik).await?;
            let filings = client
                .filings(FilingQuery {
                    cik,
                    form: args.form,
                    latest: args.latest,
                    from: None,
                    to: None,
                    include_amends: args.include_amends,
                })
                .await?;

            let mut matches = Vec::new();
            for filing in filings {
                let url = accession_text_url(filing.cik, &filing.accession);
                let text = client
                    .get_text(&url)
                    .await
                    .with_context(|| format!("failed to download {}", url))?;
                matches.extend(find_matches(
                    &filing,
                    &text,
                    &SearchQuery {
                        query: args.query.clone(),
                        context: args.context,
                    },
                ));
            }
            print_records(&matches, output)?;
        }
        Command::Section(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let cik = resolve_cik(&client, args.ticker.as_deref(), args.cik).await?;
            let records = client
                .sections(SectionQuery {
                    cik,
                    form: Some(args.form),
                    latest: args.latest,
                    include_amends: args.include_amends,
                    accession: args.accession,
                    item: args.item,
                    limit_bytes: args.limit_bytes,
                })
                .await?;
            print_records(&records, output)?;
        }
        Command::Report(args) => {
            let (cik, subject) = resolve_subject(
                &client,
                args.ticker.as_deref(),
                args.cik,
                args.investor.as_deref(),
                args.manager.as_deref(),
            )
            .await?;
            let report = client
                .markdown_report(
                    report_kind(args.kind),
                    ReportQuery {
                        cik,
                        subject,
                        latest: args.latest,
                        limit: args.limit,
                        include_amends: args.include_amends,
                        limit_bytes: args.limit_bytes,
                    },
                )
                .await?;
            println!("{report}");
        }
        Command::Resolve(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let records = if let Some(cik) = args.cik {
                client.resolve_input(ResolveInput::Cik(cik)).await?
            } else if let Some(manager) = args.manager.as_deref() {
                client
                    .resolve_input(ResolveInput::Manager(manager.to_string()))
                    .await?
            } else if let Some(query) = args.query.as_deref() {
                client
                    .resolve_query(query, !args.no_verify, llm_overrides(&args))
                    .await?
            } else {
                bail!("provide --query, --cik, or --manager");
            };
            print_records(&records, output)?;
        }
        Command::Docs(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let cik = resolve_cik(&client, args.ticker.as_deref(), args.cik).await?;
            let documents = client
                .document_records(DocumentQuery {
                    cik,
                    form: args.form,
                    latest: args.latest,
                    include_amends: args.include_amends,
                    limit: args.limit,
                })
                .await?;
            print_records(&documents, output)?;
        }
        Command::Doc(args) => {
            validate_doc_args(&args.filename, &args.sequence)?;
            let cik = resolve_cik(&client, args.ticker.as_deref(), args.cik).await?;
            let limit_bytes = if args.text { None } else { args.limit_bytes };
            let record = client
                .document_content(DocumentReadQuery {
                    cik,
                    form: args.form,
                    latest: args.latest,
                    include_amends: args.include_amends,
                    accession: args.accession,
                    filename: args.filename,
                    sequence: args.sequence,
                    primary: args.primary,
                    limit_bytes,
                })
                .await?;

            if args.raw || args.text {
                print!(
                    "{}",
                    content_for_terminal(&record, args.text, args.limit_bytes)
                );
            } else {
                print_records(&[record], output_mode(args.jsonl, args.pretty))?;
            }
        }
        Command::Form4(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let cik = resolve_cik(&client, args.ticker.as_deref(), args.cik).await?;
            let mut transactions = client
                .form4_transactions(Form4Query {
                    cik,
                    latest: args.latest,
                    include_amends: args.include_amends,
                })
                .await?;
            if let Some(limit) = args.limit {
                transactions.truncate(limit);
            }
            print_records(&transactions, output)?;
        }
        Command::Form4Summary(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let cik = resolve_cik(&client, args.ticker.as_deref(), args.cik).await?;
            let mut reports = client
                .form4_reports(Form4Query {
                    cik,
                    latest: args.latest,
                    include_amends: args.include_amends,
                })
                .await?;
            if let Some(limit) = args.limit {
                reports.truncate(limit);
            }
            print_records(&reports, output)?;
        }
        Command::EightK(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let cik = resolve_cik(&client, args.ticker.as_deref(), args.cik).await?;
            let mut events = client
                .eightk_events(EightKQuery {
                    cik,
                    latest: args.latest,
                    include_amends: args.include_amends,
                    item: args.item,
                    limit_bytes: args.limit_bytes,
                })
                .await?;
            if let Some(limit) = args.limit {
                events.truncate(limit);
            }
            print_records(&events, output)?;
        }
        Command::Schedule13(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let cik = resolve_cik(&client, args.ticker.as_deref(), args.cik).await?;
            let reports = client
                .schedule13_reports(Schedule13Query {
                    cik,
                    form: args.form,
                    latest: args.latest,
                    include_amends: args.include_amends,
                    limit_bytes: args.limit_bytes,
                })
                .await?;
            print_records(&reports, output)?;
        }
        Command::ThirteenF(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let (cik, _) = resolve_subject(
                &client,
                args.ticker.as_deref(),
                args.cik,
                args.investor.as_deref(),
                args.manager.as_deref(),
            )
            .await?;
            let mut holdings = client
                .thirteenf_holdings(ThirteenFQuery {
                    cik,
                    latest: args.latest,
                    include_amends: args.include_amends,
                })
                .await?;
            if let Some(limit) = args.limit {
                holdings.truncate(limit);
            }
            print_records(&holdings, output)?;
        }
        Command::ThirteenFAggregate(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let (cik, _) = resolve_subject(
                &client,
                args.ticker.as_deref(),
                args.cik,
                args.investor.as_deref(),
                args.manager.as_deref(),
            )
            .await?;
            let mut holdings = client
                .thirteenf_aggregate_holdings(ThirteenFQuery {
                    cik,
                    latest: args.latest,
                    include_amends: args.include_amends,
                })
                .await?;
            if let Some(limit) = args.limit {
                holdings.truncate(limit);
            }
            print_records(&holdings, output)?;
        }
        Command::ThirteenFDiff(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let (cik, _) = resolve_subject(
                &client,
                args.ticker.as_deref(),
                args.cik,
                args.investor.as_deref(),
                args.manager.as_deref(),
            )
            .await?;
            let mut changes = client
                .thirteenf_diff_holdings(ThirteenFQuery {
                    cik,
                    latest: args.latest,
                    include_amends: args.include_amends,
                })
                .await?;
            if let Some(limit) = args.limit {
                changes.truncate(limit);
            }
            print_records(&changes, output)?;
        }
        Command::ThirteenFSummary(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let (cik, _) = resolve_subject(
                &client,
                args.ticker.as_deref(),
                args.cik,
                args.investor.as_deref(),
                args.manager.as_deref(),
            )
            .await?;
            let mut reports = client
                .thirteenf_reports(ThirteenFQuery {
                    cik,
                    latest: args.latest,
                    include_amends: args.include_amends,
                })
                .await?;
            if let Some(limit) = args.limit {
                reports.truncate(limit);
            }
            print_records(&reports, output)?;
        }
        Command::Parse(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let cik = resolve_cik(&client, args.ticker.as_deref(), args.cik).await?;
            let records = client
                .parse_form(ParseQuery {
                    cik,
                    form: args.form,
                    latest: args.latest,
                    include_amends: args.include_amends,
                    limit: args.limit,
                })
                .await?;
            print_records(&records, output)?;
        }
        Command::Forms(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            print_records(supported_parsers(), output)?;
        }
        Command::Serve(args) => {
            sec_cli::server::serve(client, args.host, args.port).await?;
        }
    }

    Ok(())
}

fn report_kind(kind: ReportKindArg) -> ReportKind {
    match kind {
        ReportKindArg::Insider => ReportKind::Insider,
        ReportKindArg::Portfolio => ReportKind::Portfolio,
        ReportKindArg::Risk => ReportKind::Risk,
    }
}

pub(super) fn output_mode(jsonl: bool, pretty: bool) -> OutputMode {
    if jsonl {
        OutputMode::JsonLines
    } else if pretty {
        OutputMode::PrettyJson
    } else {
        OutputMode::Json
    }
}

fn statement_period_form(period: StatementPeriodArg) -> Option<String> {
    match period {
        StatementPeriodArg::Annual => Some("10-K".to_string()),
        StatementPeriodArg::Quarterly => Some("10-Q".to_string()),
        StatementPeriodArg::All => None,
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

async fn resolve_subject(
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

fn llm_overrides(args: &ResolveArgs) -> Option<LlmConfig> {
    if args.llm_provider.is_none()
        && args.llm_base_url.is_none()
        && args.llm_model.is_none()
        && args.llm_api_key_env.is_none()
    {
        return None;
    }
    Some(LlmConfig {
        provider: args.llm_provider.as_ref().map(|provider| match provider {
            LlmProviderArg::Openai => LlmProvider::OpenAi,
            LlmProviderArg::Anthropic => LlmProvider::Anthropic,
        }),
        base_url: args.llm_base_url.clone(),
        model: args.llm_model.clone(),
        api_key: None,
        api_key_env: args.llm_api_key_env.clone(),
        max_tokens: None,
    })
}
