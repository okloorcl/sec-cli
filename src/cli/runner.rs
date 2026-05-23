use anyhow::{Result, bail};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use sec_cli::sec::{
    DocumentQuery, DocumentReadQuery, EightKQuery, FactQuery, FilingQuery, Form4Query, OutputMode,
    ParseQuery, ReportKind, ReportQuery, Schedule13Query, SearchQuery, SecClient, SectionQuery,
    ThirteenFQuery,
    documents::read::{content_for_terminal, validate_doc_args},
    find_matches,
    llm::{LlmConfig, LlmProvider},
    print_records,
    resolve::ResolveInput,
    supported_parsers,
};

use super::args::{Cli, Command, LlmProviderArg, ReportKindArg, ResolveArgs};
use super::common::{output_mode, resolve_cik, resolve_subject, set_output_override};
use super::config::{config_path, read_config, set_identity};
use super::handlers;
use super::identity::resolve_identity;
use super::system_args::ConfigCommand;

pub(crate) async fn run() -> Result<()> {
    let cli = Cli::parse();
    set_output_override(cli.output.map(OutputMode::from));

    match &cli.command {
        Command::Completions(args) => {
            let mut command = Cli::command();
            let shell: clap_complete::Shell = args.shell.into();
            generate(shell, &mut command, "sec", &mut std::io::stdout());
            return Ok(());
        }
        Command::Config(args) => {
            handle_config(args.command.clone())?;
            return Ok(());
        }
        _ => {}
    }

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
        Command::Daily(args) => handlers::daily(&client, args).await?,
        Command::Efts(args) => handlers::efts(&client, args).await?,
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
        Command::Statements(args) => handlers::statements(&client, args).await?,
        Command::Stitch(args) => handlers::stitch(&client, args).await?,
        Command::Metrics(args) => handlers::metrics(&client, args).await?,
        Command::Scores(args) => handlers::scores(&client, args).await?,
        Command::Export(args) => handlers::export(&client, args).await?,
        Command::XbrlLinks(args) => handlers::xbrl_links(&client, args).await?,
        Command::XbrlTree(args) => handlers::xbrl_tree(&client, args).await?,
        Command::XbrlCalc(args) => handlers::xbrl_calc(&client, args).await?,
        Command::XbrlStatement(args) => handlers::xbrl_statement(&client, args).await?,
        Command::Ixbrl(args) => handlers::ixbrl(&client, args).await?,
        Command::Tables(args) => handlers::tables(&client, args).await?,
        Command::CompanyReport(args) => handlers::company_report(&client, args).await?,
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
            for (filing, text) in client.filing_texts_batch(filings).await? {
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
        Command::EightKExhibits(args) => handlers::eightk_exhibits(&client, args).await?,
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
        Command::Mcp(_) => {
            sec_cli::mcp::serve_stdio(client).await?;
        }
        Command::Completions(_) | Command::Config(_) => unreachable!("handled before client init"),
    }

    Ok(())
}

fn handle_config(command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::SetIdentity { identity } => {
            let config = set_identity(identity)?;
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        ConfigCommand::Show => {
            let config = read_config()?;
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        ConfigCommand::Path => println!("{}", config_path().display()),
    }
    Ok(())
}

fn report_kind(kind: ReportKindArg) -> ReportKind {
    match kind {
        ReportKindArg::Financial => ReportKind::Financial,
        ReportKindArg::Insider => ReportKind::Insider,
        ReportKindArg::Portfolio => ReportKind::Portfolio,
        ReportKindArg::Risk => ReportKind::Risk,
    }
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
