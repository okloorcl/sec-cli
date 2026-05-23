mod args;
mod protocol;
mod schema;

use std::io::{BufRead, Write};

use anyhow::{Result, anyhow};
use serde_json::{Value, json};

use crate::sec::{SecClient, find_matches, supported_parsers};

use args::{
    company_report_query, daily_query, document_query, document_read_query, efts_query,
    eightk_exhibit_query, eightk_query, fact_query, filing_query, foreign_query, form4_query,
    fund_query, ixbrl_query, metrics_query, parse_query, prospectus_query, proxy_query,
    report_kind, report_query, schedule13_query, search_inputs, section_query, statement_query,
    table_query, thirteenf_query,
};
use protocol::{RpcRequest, error_response, initialize_result, success_response};
use schema::tools;

pub async fn serve_stdio(client: SecClient) -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let response = handle_line(&client, &line).await;
        if let Some(response) = response {
            writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
            stdout.flush()?;
        }
    }

    Ok(())
}

async fn handle_line(client: &SecClient, line: &str) -> Option<Value> {
    let request = match serde_json::from_str::<RpcRequest>(line) {
        Ok(request) => request,
        Err(error) => return Some(error_response(None, -32700, error.to_string())),
    };

    let Some(id) = request.id.clone() else {
        return None;
    };

    let result = match request.method.as_str() {
        "initialize" => Ok(initialize_result()),
        "tools/list" => Ok(json!({ "tools": tools() })),
        "tools/call" => call_tool(client, request.params.unwrap_or_default()).await,
        _ => Err(anyhow!("unsupported MCP method '{}'", request.method)),
    };

    Some(match result {
        Ok(value) => success_response(id, value),
        Err(error) => error_response(Some(id), -32603, error.to_string()),
    })
}

async fn call_tool(client: &SecClient, params: Value) -> Result<Value> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("tools/call requires params.name"))?;
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    let data = match name {
        "sec_forms" => json!(supported_parsers()),
        "sec_filings" => json!(client.filings(filing_query(client, &args).await?).await?),
        "sec_daily" => json!(client.daily_filings(daily_query(&args).await?).await?),
        "sec_efts" => json!(client.efts_search(efts_query(client, &args).await?).await?),
        "sec_facts" => json!(client.facts(fact_query(client, &args).await?).await?),
        "sec_statements" => json!(
            client
                .financial_statements(statement_query(client, &args).await?)
                .await?
        ),
        "sec_metrics" => json!(
            client
                .financial_metrics(metrics_query(client, &args).await?)
                .await?
        ),
        "sec_ixbrl" => json!(
            client
                .inline_xbrl_facts(ixbrl_query(client, &args).await?)
                .await?
        ),
        "sec_tables" => json!(
            client
                .html_tables(table_query(client, &args).await?)
                .await?
        ),
        "sec_company_report" => json!(
            client
                .company_reports(company_report_query(client, &args).await?)
                .await?
        ),
        "sec_proxy" => json!(
            client
                .proxy_statements(proxy_query(client, &args).await?)
                .await?
        ),
        "sec_prospectus" => json!(
            client
                .prospectuses(prospectus_query(client, &args).await?)
                .await?
        ),
        "sec_foreign" => json!(
            client
                .foreign_issuer_reports(foreign_query(client, &args).await?)
                .await?
        ),
        "sec_fund" => json!(
            client
                .fund_disclosures(fund_query(client, &args).await?)
                .await?
        ),
        "sec_search" => json!(search(client, &args).await?),
        "sec_section" => json!(client.sections(section_query(client, &args).await?).await?),
        "sec_docs" => json!(
            client
                .document_records(document_query(client, &args).await?)
                .await?
        ),
        "sec_doc" => json!(
            client
                .document_content(document_read_query(client, &args).await?)
                .await?
        ),
        "sec_form4" => json!(
            client
                .form4_transactions(form4_query(client, &args).await?)
                .await?
        ),
        "sec_form4_summary" => {
            json!(
                client
                    .form4_reports(form4_query(client, &args).await?)
                    .await?
            )
        }
        "sec_8k" => json!(
            client
                .eightk_events(eightk_query(client, &args).await?)
                .await?
        ),
        "sec_8k_exhibits" => json!(
            client
                .eightk_exhibits(eightk_exhibit_query(client, &args).await?)
                .await?
        ),
        "sec_schedule13" => json!(
            client
                .schedule13_reports(schedule13_query(client, &args).await?)
                .await?
        ),
        "sec_13f" => json!(
            client
                .thirteenf_holdings(thirteenf_query(client, &args).await?)
                .await?
        ),
        "sec_13f_aggregate" => json!(
            client
                .thirteenf_aggregate_holdings(thirteenf_query(client, &args).await?)
                .await?
        ),
        "sec_13f_diff" => json!(
            client
                .thirteenf_diff_holdings(thirteenf_query(client, &args).await?)
                .await?
        ),
        "sec_13f_summary" => json!(
            client
                .thirteenf_reports(thirteenf_query(client, &args).await?)
                .await?
        ),
        "sec_report" => json!({
            "markdown": client
                .markdown_report(report_kind(&args)?, report_query(client, &args).await?)
                .await?
        }),
        "sec_parse" => json!(client.parse_form(parse_query(client, &args).await?).await?),
        _ => return Err(anyhow!("unknown MCP tool '{name}'")),
    };

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": serde_json::to_string_pretty(&data)?
            }
        ],
        "structuredContent": data,
        "isError": false
    }))
}

async fn search(client: &SecClient, args: &Value) -> Result<Value> {
    let (filing_query, search_query) = search_inputs(client, args).await?;
    let filings = client.filings(filing_query).await?;
    let mut matches = Vec::new();
    for (filing, text) in client.filing_texts_batch(filings).await? {
        matches.extend(find_matches(&filing, &text, &search_query));
    }
    Ok(json!(matches))
}
