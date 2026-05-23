use anyhow::{Result, anyhow};
use chrono::{NaiveDate, Utc};
use serde_json::Value;

use crate::sec::{
    CompanyReportQuery, DailyIndexQuery, DocumentQuery, DocumentReadQuery, EftsSearchQuery,
    EightKExhibitQuery, EightKQuery, FactQuery, FilingQuery, ForeignIssuerQuery, Form4Query,
    FundDisclosureQuery, HealthScoreQuery, HtmlTableQuery, InlineXbrlQuery, MetricsQuery,
    ParseQuery, ProspectusQuery, ProxyQuery, ReportKind, ReportQuery, Schedule13Query, SearchQuery,
    SecClient, SectionQuery, StatementQuery, ThirteenFQuery,
    daily::latest_sec_index_date,
    efts::{parse_forms, require_query},
};

pub async fn filing_query(client: &SecClient, args: &Value) -> Result<FilingQuery> {
    Ok(FilingQuery {
        cik: resolve_cik(client, args).await?,
        form: optional_string(args, "form"),
        latest: optional_usize(args, "latest").unwrap_or(10),
        from: optional_date(args, "from")?,
        to: optional_date(args, "to")?,
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
    })
}

pub async fn daily_query(args: &Value) -> Result<DailyIndexQuery> {
    Ok(DailyIndexQuery {
        date: optional_date(args, "date")?
            .unwrap_or_else(|| latest_sec_index_date(Utc::now().date_naive())),
        form: optional_string(args, "form"),
        company: optional_string(args, "company"),
        limit: optional_usize(args, "limit"),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
    })
}

pub async fn efts_query(client: &SecClient, args: &Value) -> Result<EftsSearchQuery> {
    let cik = resolve_optional_cik(client, args).await?;
    Ok(EftsSearchQuery {
        query: require_query(&required_string(args, "query")?)?,
        ciks: cik.into_iter().collect(),
        forms: forms(args),
        from: optional_date(args, "from")?,
        to: optional_date(args, "to")?,
        limit: optional_usize(args, "limit"),
    })
}

pub async fn fact_query(client: &SecClient, args: &Value) -> Result<FactQuery> {
    Ok(FactQuery {
        cik: resolve_cik(client, args).await?,
        concept: required_string(args, "concept")?,
        form: optional_string(args, "form"),
        unit: optional_string(args, "unit"),
        latest: optional_usize(args, "latest").unwrap_or(20),
    })
}

pub async fn statement_query(client: &SecClient, args: &Value) -> Result<StatementQuery> {
    Ok(StatementQuery {
        cik: resolve_cik(client, args).await?,
        statement: optional_string(args, "statement").unwrap_or_else(|| "all".to_string()),
        form: period_form(optional_string(args, "period").as_deref()),
        unit: optional_string(args, "unit"),
        latest: optional_usize(args, "latest").unwrap_or(4),
    })
}

pub async fn metrics_query(client: &SecClient, args: &Value) -> Result<MetricsQuery> {
    Ok(MetricsQuery {
        cik: resolve_cik(client, args).await?,
        form: period_form(optional_string(args, "period").as_deref()),
        unit: optional_string(args, "unit"),
        latest: optional_usize(args, "latest").unwrap_or(4),
    })
}

pub async fn health_score_query(client: &SecClient, args: &Value) -> Result<HealthScoreQuery> {
    Ok(HealthScoreQuery {
        cik: resolve_cik(client, args).await?,
        form: period_form(optional_string(args, "period").as_deref()),
        unit: optional_string(args, "unit"),
        latest: optional_usize(args, "latest").unwrap_or(1),
    })
}

pub async fn ixbrl_query(client: &SecClient, args: &Value) -> Result<InlineXbrlQuery> {
    Ok(InlineXbrlQuery {
        cik: resolve_cik(client, args).await?,
        form: Some(optional_string(args, "form").unwrap_or_else(|| "10-K".to_string())),
        latest: optional_usize(args, "latest").unwrap_or(1),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        concept: optional_string(args, "concept"),
        limit: optional_usize(args, "limit"),
    })
}

pub async fn table_query(client: &SecClient, args: &Value) -> Result<HtmlTableQuery> {
    Ok(HtmlTableQuery {
        cik: resolve_cik(client, args).await?,
        form: Some(optional_string(args, "form").unwrap_or_else(|| "10-K".to_string())),
        latest: optional_usize(args, "latest").unwrap_or(1),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        limit_tables: optional_usize(args, "limit_tables"),
        limit_rows: optional_usize(args, "limit_rows"),
    })
}

pub async fn company_report_query(client: &SecClient, args: &Value) -> Result<CompanyReportQuery> {
    Ok(CompanyReportQuery {
        cik: resolve_cik(client, args).await?,
        form: Some(optional_string(args, "form").unwrap_or_else(|| "10-K".to_string())),
        latest: optional_usize(args, "latest").unwrap_or(1),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        topic: optional_string(args, "topic"),
        limit_tables: optional_usize(args, "limit_tables"),
        limit_rows: optional_usize(args, "limit_rows"),
    })
}

pub async fn proxy_query(client: &SecClient, args: &Value) -> Result<ProxyQuery> {
    Ok(ProxyQuery {
        cik: resolve_cik(client, args).await?,
        latest: optional_usize(args, "latest").unwrap_or(1),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        limit_rows: optional_usize(args, "limit_rows"),
    })
}

pub async fn prospectus_query(client: &SecClient, args: &Value) -> Result<ProspectusQuery> {
    Ok(ProspectusQuery {
        cik: resolve_cik(client, args).await?,
        form: Some(optional_string(args, "form").unwrap_or_else(|| "all".to_string())),
        latest: optional_usize(args, "latest").unwrap_or(1),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        limit_bytes: optional_usize(args, "limit_bytes"),
        limit_tables: optional_usize(args, "limit_tables"),
        limit_rows: optional_usize(args, "limit_rows"),
    })
}

pub async fn foreign_query(client: &SecClient, args: &Value) -> Result<ForeignIssuerQuery> {
    Ok(ForeignIssuerQuery {
        cik: resolve_cik(client, args).await?,
        form: Some(optional_string(args, "form").unwrap_or_else(|| "all".to_string())),
        latest: optional_usize(args, "latest").unwrap_or(1),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        limit_bytes: optional_usize(args, "limit_bytes"),
    })
}

pub async fn fund_query(client: &SecClient, args: &Value) -> Result<FundDisclosureQuery> {
    Ok(FundDisclosureQuery {
        cik: resolve_cik(client, args).await?,
        form: Some(optional_string(args, "form").unwrap_or_else(|| "all".to_string())),
        latest: optional_usize(args, "latest").unwrap_or(1),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        limit_holdings: optional_usize(args, "limit_holdings"),
        limit_bytes: optional_usize(args, "limit_bytes"),
    })
}

pub async fn search_inputs(client: &SecClient, args: &Value) -> Result<(FilingQuery, SearchQuery)> {
    Ok((
        FilingQuery {
            cik: resolve_cik(client, args).await?,
            form: optional_string(args, "form"),
            latest: optional_usize(args, "latest").unwrap_or(10),
            from: None,
            to: None,
            include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        },
        SearchQuery {
            query: required_string(args, "query")?,
            context: optional_usize(args, "context").unwrap_or(240),
        },
    ))
}

pub async fn section_query(client: &SecClient, args: &Value) -> Result<SectionQuery> {
    Ok(SectionQuery {
        cik: resolve_cik(client, args).await?,
        form: Some(optional_string(args, "form").unwrap_or_else(|| "10-K".to_string())),
        latest: optional_usize(args, "latest").unwrap_or(1),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        accession: optional_string(args, "accession"),
        item: required_string(args, "item")?,
        limit_bytes: optional_usize(args, "limit_bytes"),
    })
}

pub async fn document_query(client: &SecClient, args: &Value) -> Result<DocumentQuery> {
    Ok(DocumentQuery {
        cik: resolve_cik(client, args).await?,
        form: optional_string(args, "form"),
        latest: optional_usize(args, "latest").unwrap_or(1),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        limit: optional_usize(args, "limit"),
    })
}

pub async fn document_read_query(client: &SecClient, args: &Value) -> Result<DocumentReadQuery> {
    Ok(DocumentReadQuery {
        cik: resolve_cik(client, args).await?,
        form: optional_string(args, "form"),
        latest: optional_usize(args, "latest").unwrap_or(1),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        accession: optional_string(args, "accession"),
        filename: optional_string(args, "filename"),
        sequence: optional_string(args, "sequence"),
        primary: optional_bool(args, "primary").unwrap_or(false),
        limit_bytes: optional_usize(args, "limit_bytes"),
    })
}

pub async fn form4_query(client: &SecClient, args: &Value) -> Result<Form4Query> {
    Ok(Form4Query {
        cik: resolve_cik(client, args).await?,
        latest: optional_usize(args, "latest").unwrap_or(3),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
    })
}

pub async fn eightk_query(client: &SecClient, args: &Value) -> Result<EightKQuery> {
    Ok(EightKQuery {
        cik: resolve_cik(client, args).await?,
        latest: optional_usize(args, "latest").unwrap_or(5),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        item: optional_string(args, "item"),
        limit_bytes: optional_usize(args, "limit_bytes"),
    })
}

pub async fn eightk_exhibit_query(client: &SecClient, args: &Value) -> Result<EightKExhibitQuery> {
    Ok(EightKExhibitQuery {
        cik: resolve_cik(client, args).await?,
        latest: optional_usize(args, "latest").unwrap_or(5),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        category: optional_string(args, "category"),
        limit_bytes: optional_usize(args, "limit_bytes"),
    })
}

pub async fn schedule13_query(client: &SecClient, args: &Value) -> Result<Schedule13Query> {
    Ok(Schedule13Query {
        cik: resolve_cik(client, args).await?,
        form: optional_string(args, "form"),
        latest: optional_usize(args, "latest").unwrap_or(2),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        limit_bytes: optional_usize(args, "limit_bytes"),
    })
}

pub async fn thirteenf_query(client: &SecClient, args: &Value) -> Result<ThirteenFQuery> {
    Ok(ThirteenFQuery {
        cik: resolve_cik(client, args).await?,
        latest: optional_usize(args, "latest").unwrap_or(1),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
    })
}

pub async fn report_query(client: &SecClient, args: &Value) -> Result<ReportQuery> {
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

pub async fn parse_query(client: &SecClient, args: &Value) -> Result<ParseQuery> {
    Ok(ParseQuery {
        cik: resolve_cik(client, args).await?,
        form: required_string(args, "form")?,
        latest: optional_usize(args, "latest").unwrap_or(1),
        include_amends: optional_bool(args, "include_amends").unwrap_or(false),
        limit: optional_usize(args, "limit"),
    })
}

pub fn report_kind(args: &Value) -> Result<ReportKind> {
    match optional_string(args, "kind")
        .unwrap_or_else(|| "risk".to_string())
        .to_ascii_lowercase()
        .as_str()
    {
        "financial" => Ok(ReportKind::Financial),
        "insider" => Ok(ReportKind::Insider),
        "portfolio" => Ok(ReportKind::Portfolio),
        "risk" => Ok(ReportKind::Risk),
        other => Err(anyhow!("unsupported report kind '{other}'")),
    }
}

async fn resolve_cik(client: &SecClient, args: &Value) -> Result<u64> {
    match (optional_string(args, "ticker"), optional_u64(args, "cik")) {
        (Some(ticker), None) => client.cik_for_ticker(&ticker).await,
        (None, Some(cik)) => Ok(cik),
        (Some(_), Some(_)) => Err(anyhow!("provide either ticker or cik, not both")),
        (None, None) => Err(anyhow!("provide ticker or cik")),
    }
}

async fn resolve_optional_cik(client: &SecClient, args: &Value) -> Result<Option<u64>> {
    match (optional_string(args, "ticker"), optional_u64(args, "cik")) {
        (Some(ticker), None) => Ok(Some(client.cik_for_ticker(&ticker).await?)),
        (None, Some(cik)) => Ok(Some(cik)),
        (None, None) => Ok(None),
        (Some(_), Some(_)) => Err(anyhow!("provide either ticker or cik, not both")),
    }
}

fn forms(args: &Value) -> Vec<String> {
    if let Some(values) = args.get("forms").and_then(Value::as_array) {
        return values
            .iter()
            .filter_map(Value::as_str)
            .flat_map(|value| parse_forms(&[value.to_string()]))
            .collect();
    }
    optional_string(args, "form")
        .map(|value| parse_forms(&[value]))
        .unwrap_or_default()
}

fn period_form(period: Option<&str>) -> Option<String> {
    match period.unwrap_or("annual").to_ascii_lowercase().as_str() {
        "annual" => Some("10-K".to_string()),
        "quarterly" => Some("10-Q".to_string()),
        "all" => None,
        other => Some(other.to_string()),
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

fn optional_date(args: &Value, key: &str) -> Result<Option<NaiveDate>> {
    optional_string(args, key)
        .map(|value| {
            NaiveDate::parse_from_str(&value, "%Y-%m-%d")
                .map_err(|error| anyhow!("invalid {key} date '{value}': {error}"))
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn maps_statement_periods() {
        assert_eq!(period_form(Some("annual")).as_deref(), Some("10-K"));
        assert_eq!(period_form(Some("quarterly")).as_deref(), Some("10-Q"));
        assert_eq!(period_form(Some("all")), None);
    }

    #[test]
    fn parses_form_arrays_and_comma_strings() {
        assert_eq!(forms(&json!({"form": "10-K,8-K"})), vec!["10-K", "8-K"]);
        assert_eq!(
            forms(&json!({"forms": ["10-K", "8-K"]})),
            vec!["10-K", "8-K"]
        );
    }
}
