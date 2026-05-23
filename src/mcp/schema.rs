use serde_json::{Value, json};

pub fn tools() -> Vec<Value> {
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
            "sec_daily",
            "Scan the SEC all-market daily master index.",
            json!({"type":"object","properties":{"date":{"type":"string"},"form":{"type":"string"},"company":{"type":"string"},"limit":{"type":"integer"},"include_amends":{"type":"boolean"}},"additionalProperties":false}),
        ),
        tool(
            "sec_efts",
            "Search SEC EDGAR Full-Text Search across the market.",
            company_schema(
                json!({"query":{"type":"string"},"form":{"type":["string","array"]},"forms":{"type":"array","items":{"type":"string"}},"from":{"type":"string"},"to":{"type":"string"},"limit":{"type":"integer"}}),
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
            "sec_stitch",
            "Stitch 10-K and 10-Q statement rows into a de-duplicated CompanyFacts time series.",
            company_schema(
                json!({"statement":{"type":"string"},"unit":{"type":"string"},"latest":{"type":"integer"}}),
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
            "sec_scores",
            "Calculate SEC-derived financial-health scores: Piotroski F-Score, Altman Z'' approximation, and Beneish M-Score approximation.",
            company_schema(
                json!({"period":{"type":"string"},"unit":{"type":"string"},"latest":{"type":"integer"}}),
            ),
        ),
        tool(
            "sec_agent_pack",
            "Build a source-backed research packet with filings, sections, metrics, scores, source URLs, and next commands.",
            company_schema(
                json!({"form":{"type":"string"},"latest":{"type":"integer"},"sections":{"type":["string","array"],"items":{"type":"string"}},"section_limit_bytes":{"type":"integer"},"metrics_latest":{"type":"integer"}}),
            ),
        ),
        tool(
            "sec_ixbrl",
            "Stream Inline XBRL facts from primary filing HTML.",
            company_schema(
                json!({"form":{"type":"string"},"concept":{"type":"string"},"latest":{"type":"integer"},"limit":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_tables",
            "Extract HTML tables from primary filing documents.",
            company_schema(
                json!({"form":{"type":"string"},"latest":{"type":"integer"},"limit_tables":{"type":"integer"},"limit_rows":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_company_report",
            "Parse 10-K/10-Q topic tables such as segment revenue, geography, debt maturities, obligations, leases, taxes, and repurchases.",
            company_schema(
                json!({"form":{"type":"string"},"topic":{"type":"string"},"latest":{"type":"integer"},"limit_tables":{"type":"integer"},"limit_rows":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_proxy",
            "Parse DEF 14A proxy statement governance and compensation signals.",
            company_schema(
                json!({"latest":{"type":"integer"},"limit_rows":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_prospectus",
            "Parse S-1/F-1/424B prospectus signals and offering tables.",
            company_schema(
                json!({"form":{"type":"string"},"latest":{"type":"integer"},"limit_bytes":{"type":"integer"},"limit_tables":{"type":"integer"},"limit_rows":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_foreign",
            "Parse 20-F/6-K/40-F foreign issuer disclosures.",
            company_schema(
                json!({"form":{"type":"string"},"latest":{"type":"integer"},"limit_bytes":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_fund",
            "Parse N-PORT/N-CSR/N-CEN/N-PX/497K/24F-2NT fund disclosures.",
            company_schema(
                json!({"form":{"type":"string"},"latest":{"type":"integer"},"limit_holdings":{"type":"integer"},"limit_bytes":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_search",
            "Search selected company filing text and return source-backed snippets.",
            company_schema(
                json!({"form":{"type":"string"},"query":{"type":"string"},"latest":{"type":"integer"},"context":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_section",
            "Extract a named 10-K/10-Q filing section.",
            company_schema(
                json!({"form":{"type":"string"},"item":{"type":"string"},"latest":{"type":"integer"},"accession":{"type":"string"},"limit_bytes":{"type":"integer"},"include_amends":{"type":"boolean"}}),
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
            "sec_doc",
            "Read one document from a complete SEC submission.",
            company_schema(
                json!({"form":{"type":"string"},"latest":{"type":"integer"},"accession":{"type":"string"},"filename":{"type":"string"},"sequence":{"type":"string"},"primary":{"type":"boolean"},"limit_bytes":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_form4",
            "Parse Form 4 row-level insider transactions.",
            company_schema(
                json!({"latest":{"type":"integer"},"include_amends":{"type":"boolean"}}),
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
            "sec_8k",
            "Parse Form 8-K current-report events by item.",
            company_schema(
                json!({"item":{"type":"string"},"latest":{"type":"integer"},"limit_bytes":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_8k_exhibits",
            "Discover and classify 8-K exhibits such as earnings releases, press releases, contracts, agreements, XBRL, and accountant letters.",
            company_schema(
                json!({"latest":{"type":"integer"},"category":{"type":"string"},"limit_bytes":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_schedule13",
            "Parse Schedule 13D/13G beneficial ownership reports.",
            company_schema(
                json!({"form":{"type":"string"},"latest":{"type":"integer"},"limit_bytes":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_13f",
            "Parse 13F information-table holdings.",
            company_schema(
                json!({"latest":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_13f_aggregate",
            "Aggregate 13F holdings by CUSIP/class/put-call.",
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
            "sec_13f_summary",
            "Parse 13F cover, summary, signature, and manager metadata.",
            company_schema(
                json!({"latest":{"type":"integer"},"include_amends":{"type":"boolean"}}),
            ),
        ),
        tool(
            "sec_report",
            "Generate source-backed Markdown reports for financial, insider, portfolio, or risk workflows.",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declares_broad_tool_surface() {
        let names = tools()
            .into_iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str).map(str::to_string))
            .collect::<Vec<_>>();

        for name in [
            "sec_forms",
            "sec_filings",
            "sec_daily",
            "sec_efts",
            "sec_scores",
            "sec_agent_pack",
            "sec_stitch",
            "sec_ixbrl",
            "sec_tables",
            "sec_proxy",
            "sec_prospectus",
            "sec_foreign",
            "sec_fund",
            "sec_search",
            "sec_section",
            "sec_schedule13",
            "sec_13f_summary",
            "sec_parse",
        ] {
            assert!(names.contains(&name.to_string()), "missing {name}");
        }
    }
}
