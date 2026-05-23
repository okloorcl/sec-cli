use std::env;

use anyhow::{Context, Result, bail};
use clap::Parser;
use sec_cli::sec::{
    DocumentQuery, FactQuery, FilingQuery, Form4Query, OutputMode, ParseQuery, SearchQuery,
    SecClient, ThirteenFQuery, accession_text_url, find_matches, print_records, supported_parsers,
};

use super::args::{Cli, Command};

pub(crate) async fn run() -> Result<()> {
    let cli = Cli::parse();
    let identity = cli
        .identity
        .or_else(|| env::var("EDGAR_IDENTITY").ok())
        .or_else(|| env::var("SEC_IDENTITY").ok())
        .unwrap_or_else(|| "sec-cli/0.1.0 contact@example.com".to_string());

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
        Command::ThirteenF(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let cik = resolve_cik(&client, args.ticker.as_deref(), args.cik).await?;
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
        Command::ThirteenFSummary(args) => {
            let output = output_mode(args.jsonl, args.pretty);
            let cik = resolve_cik(&client, args.ticker.as_deref(), args.cik).await?;
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
    }

    Ok(())
}

fn output_mode(jsonl: bool, pretty: bool) -> OutputMode {
    if jsonl {
        OutputMode::JsonLines
    } else if pretty {
        OutputMode::PrettyJson
    } else {
        OutputMode::Json
    }
}

async fn resolve_cik(client: &SecClient, ticker: Option<&str>, cik: Option<u64>) -> Result<u64> {
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
