use anyhow::Result;
use sec_cli::sec::{HtmlTableQuery, InlineXbrlQuery, ProxyQuery, SecClient, print_records};

use super::{
    args::{InlineXbrlArgs, ProxyArgs, TablesArgs},
    runner::{output_mode, resolve_cik},
};

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
