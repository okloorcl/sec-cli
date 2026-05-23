use std::io::{BufRead, Write};

use anyhow::{Result, anyhow};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::sec::{
    DocumentQuery, FactQuery, FilingQuery, Form4Query, MetricsQuery, ParseQuery, ReportKind,
    ReportQuery, SecClient, StatementQuery, ThirteenFQuery, supported_parsers,
};

const PROTOCOL_VERSION: &str = "2025-06-18";

#[derive(Debug, Deserialize)]
struct RpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

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
        "sec_docs" => json!(
            client
                .document_records(document_query(client, &args).await?)
                .await?
        ),
        "sec_form4_summary" => {
            json!(
                client
                    .form4_reports(form4_query(client, &args).await?)
                    .await?
            )
        }
        "sec_13f_diff" => json!(
            client
                .thirteenf_diff_holdings(thirteenf_query(client, &args).await?)
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

async fn filing_query(client: &SecClient, args: &Value) -> Result<FilingQuery> {
    Ok(FilingQuery {
        cik: resolve_cik(client, args).await?,
        form: optional_string(args, "form"),
        latest: optional_usize(args, "latest").unwrap_or(10),
        from: None,
        to: None,
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
    })
}

async fn fact_query(client: &SecClient, args: &Value) -> Result<FactQuery> {
    Ok(FactQuery {
        cik: resolve_cik(client, args).await?,
        concept: required_string(args, "concept")?,
        form: optional_string(args, "form"),
        unit: optional_string(args, "unit"),
        latest: optional_usize(args, "latest").unwrap_or(20),
    })
}

async fn statement_query(client: &SecClient, args: &Value) -> Result<StatementQuery> {
    Ok(StatementQuery {
        cik: resolve_cik(client, args).await?,
        statement: optional_string(args, "statement").unwrap_or_else(|| "all".to_string()),
        form: period_form(optional_string(args, "period").as_deref()),
        unit: optional_string(args, "unit"),
        latest: optional_usize(args, "latest").unwrap_or(4),
    })
}

async fn metrics_query(client: &SecClient, args: &Value) -> Result<MetricsQuery> {
    Ok(MetricsQuery {
        cik: resolve_cik(client, args).await?,
        form: period_form(optional_string(args, "period").as_deref()),
        unit: optional_string(args, "unit"),
        latest: optional_usize(args, "latest").unwrap_or(4),
    })
}

async fn document_query(client: &SecClient, args: &Value) -> Result<DocumentQuery> {
    Ok(DocumentQuery {
        cik: resolve_cik(client, args).await?,
        form: optional_string(args, "form"),
        latest: optional_usize(args, "latest").unwrap_or(1),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        limit: optional_usize(args, "limit"),
    })
}

async fn form4_query(client: &SecClient, args: &Value) -> Result<Form4Query> {
    Ok(Form4Query {
        cik: resolve_cik(client, args).await?,
        latest: optional_usize(args, "latest").unwrap_or(3),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
    })
}

async fn thirteenf_query(client: &SecClient, args: &Value) -> Result<ThirteenFQuery> {
    Ok(ThirteenFQuery {
        cik: resolve_cik(client, args).await?,
        latest: optional_usize(args, "latest").unwrap_or(1),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
    })
}

async fn report_query(client: &SecClient, args: &Value) -> Result<ReportQuery> {
    let cik = resolve_cik(client, args).await?;
    Ok(ReportQuery {
        cik,
        subject: optional_string(args, "subject").unwrap_or_else(|| cik.to_string()),
        latest: optional_usize(args, "latest").unwrap_or(5),
        limit: optional_usize(args, "limit").unwrap_or(10),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        limit_bytes: optional_usize(args, "limit_bytes").unwrap_or(4000),
    })
}

async fn parse_query(client: &SecClient, args: &Value) -> Result<ParseQuery> {
    Ok(ParseQuery {
        cik: resolve_cik(client, args).await?,
        form: required_string(args, "form")?,
        latest: optional_usize(args, "latest").unwrap_or(1),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        limit: optional_usize(args, "limit"),
    })
}

async fn resolve_cik(client: &SecClient, args: &Value) -> Result<u64> {
    match (optional_string(args, "ticker"), optional_u64(args, "cik")) {
        (Some(ticker), None) => client.cik_for_ticker(&ticker).await,
        (None, Some(cik)) => Ok(cik),
        (Some(_), Some(_)) => Err(anyhow!("provide either ticker or cik, not both")),
        (None, None) => Err(anyhow!("provide ticker or cik")),
    }
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "tools": {
                "listChanged": false
            }
        },
        "serverInfo": {
            "name": "sec-cli",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

fn tools() -> Vec<Value> {
    vec![
        tool(
            "sec_forms",
            "List supported structured SEC form parsers.",
            json!({"type":"object","properties":{}}),
        ),
        tool(
            "sec_filings",
            "Find SEC filings by ticker or CIK.",
            company_schema(
                json!({"form":{"type":"string"},"latest":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_facts",
            "Query SEC CompanyFacts by concept.",
            company_schema(
                json!({"concept":{"type":"string"},"form":{"type":"string"},"unit":{"type":"string"},"latest":{"type":"integer"}}),
            ),
        ),
        tool(
            "sec_statements",
            "Build standardized statement rows from CompanyFacts.",
            company_schema(
                json!({"statement":{"type":"string"},"period":{"type":"string"},"unit":{"type":"string"},"latest":{"type":"integer"}}),
            ),
        ),
        tool(
            "sec_metrics",
            "Calculate source-backed financial ratios, growth, free cash flow, returns, liquidity, and leverage.",
            company_schema(
                json!({"period":{"type":"string"},"unit":{"type":"string"},"latest":{"type":"integer"}}),
            ),
        ),
        tool(
            "sec_docs",
            "List documents and attachments inside SEC complete submissions.",
            company_schema(
                json!({"form":{"type":"string"},"latest":{"type":"integer"},"limit":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_form4_summary",
            "Summarize Form 4 ownership reports for a company.",
            company_schema(
                json!({"latest":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_13f_diff",
            "Compare latest two 13F portfolios by CIK or ticker.",
            company_schema(
                json!({"latest":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_report",
            "Generate source-backed Markdown reports for insider, portfolio, or risk workflows.",
            company_schema(
                json!({"kind":{"type":"string"},"subject":{"type":"string"},"latest":{"type":"integer"},"limit":{"type":"integer"},"limit_bytes":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_parse",
            "Run the unified parser pipeline for a supported form.",
            company_schema(
                json!({"form":{"type":"string"},"latest":{"type":"integer"},"limit":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
    ]
}

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema
    })
}

fn company_schema(extra: Value) -> Value {
    let mut properties = serde_json::Map::new();
    properties.insert("ticker".to_string(), json!({"type": "string"}));
    properties.insert("cik".to_string(), json!({"type": "integer"}));
    if let Some(extra) = extra.as_object() {
        for (key, value) in extra {
            properties.insert(key.clone(), value.clone());
        }
    }
    json!({
        "type": "object",
        "properties": properties,
        "additionalProperties": false
    })
}

fn period_form(period: Option<&str>) -> Option<String> {
    match period.unwrap_or("annual").to_ascii_lowercase().as_str() {
        "annual" => Some("10-K".to_string()),
        "quarterly" => Some("10-Q".to_string()),
        "all" => None,
        other => Some(other.to_string()),
    }
}

fn report_kind(args: &Value) -> Result<ReportKind> {
    match optional_string(args, "kind")
        .unwrap_or_else(|| "risk".to_string())
        .to_ascii_lowercase()
        .as_str()
    {
        "insider" => Ok(ReportKind::Insider),
        "portfolio" => Ok(ReportKind::Portfolio),
        "risk" => Ok(ReportKind::Risk),
        other => Err(anyhow!("unsupported report kind '{other}'")),
    }
}

fn required_string(args: &Value, key: &str) -> Result<String> {
    optional_string(args, key).ok_or_else(|| anyhow!("missing required argument '{key}'"))
}

fn optional_string(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn optional_u64(args: &Value, key: &str) -> Option<u64> {
    args.get(key).and_then(Value::as_u64)
}

fn optional_usize(args: &Value, key: &str) -> Option<usize> {
    optional_u64(args, key).and_then(|value| usize::try_from(value).ok())
}

fn optional_bool(args: &Value, key: &str) -> Option<bool> {
    args.get(key).and_then(Value::as_bool)
}

fn success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn error_response(id: Option<Value>, code: i64, message: String) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declares_core_tools() {
        let names = tools()
            .into_iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str).map(str::to_string))
            .collect::<Vec<_>>();

        assert!(names.contains(&"sec_forms".to_string()));
        assert!(names.contains(&"sec_metrics".to_string()));
        assert!(names.contains(&"sec_report".to_string()));
        assert!(names.contains(&"sec_parse".to_string()));
    }

    #[test]
    fn maps_statement_periods() {
        assert_eq!(period_form(Some("annual")).as_deref(), Some("10-K"));
        assert_eq!(period_form(Some("quarterly")).as_deref(), Some("10-Q"));
        assert_eq!(period_form(Some("all")), None);
    }
}
