use std::{net::SocketAddr, sync::Arc};

mod errors;
mod helpers;
mod params;

use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use serde_json::json;

use errors::ApiResult;
use helpers::*;
use params::*;

use crate::sec::{
    CompanyReportQuery, DailyIndexQuery, DocumentQuery, EftsSearchQuery, EightKExhibitQuery,
    EightKQuery, FactQuery, FilingQuery, ForeignIssuerQuery, FundDisclosureQuery, InlineXbrlQuery,
    MetricsQuery, ParseQuery, ProspectusQuery, ProxyQuery, Schedule13Query, SecClient,
    SectionQuery, StatementQuery,
    daily::latest_sec_index_date,
    efts::{parse_forms, require_query},
    supported_parsers,
};

#[derive(Clone)]
struct AppState {
    client: Arc<SecClient>,
}

pub async fn serve(client: SecClient, host: String, port: u16) -> Result<()> {
    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    let state = AppState {
        client: Arc::new(client),
    };
    let app = router().with_state(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    eprintln!("sec-cli HTTP API listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/v1/forms", get(forms))
        .route("/v1/filings", get(filings))
        .route("/v1/daily", get(daily))
        .route("/v1/efts", get(efts))
        .route("/v1/facts", get(facts))
        .route("/v1/statements", get(statements))
        .route("/v1/metrics", get(metrics))
        .route("/v1/company-report", get(company_report))
        .route("/v1/ixbrl", get(ixbrl))
        .route("/v1/sections", get(sections))
        .route("/v1/docs", get(docs))
        .route("/v1/form4", get(form4))
        .route("/v1/form4-summary", get(form4_summary))
        .route("/v1/8k", get(eightk))
        .route("/v1/8k-exhibits", get(eightk_exhibits))
        .route("/v1/schedule13", get(schedule13))
        .route("/v1/13f", get(thirteenf))
        .route("/v1/13f-summary", get(thirteenf_summary))
        .route("/v1/13f-diff", get(thirteenf_diff))
        .route("/v1/proxy", get(proxy))
        .route("/v1/prospectus", get(prospectus))
        .route("/v1/foreign", get(foreign))
        .route("/v1/fund", get(fund))
        .route("/v1/parse", get(parse))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({"status": "ok", "service": "sec-cli"}))
}

async fn forms() -> Json<serde_json::Value> {
    Json(json!(supported_parsers()))
}

async fn filings(
    State(state): State<AppState>,
    Query(params): Query<FilingsParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let records = state
        .client
        .filings(FilingQuery {
            cik,
            form: params.form,
            latest: params.latest.unwrap_or(10),
            from: params.from,
            to: params.to,
            include_amends: params.include_amends.unwrap_or(false),
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn daily(
    State(state): State<AppState>,
    Query(params): Query<DailyParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let date = params
        .date
        .unwrap_or_else(|| latest_sec_index_date(chrono::Utc::now().date_naive()));
    let records = state
        .client
        .daily_filings(DailyIndexQuery {
            date,
            form: params.form,
            company: params.company,
            limit: params.limit.or(Some(100)),
            include_amends: params.include_amends.unwrap_or(false),
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn efts(
    State(state): State<AppState>,
    Query(params): Query<EftsParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_optional_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let forms = params.form.map(|form| vec![form]).unwrap_or_default();
    let records = state
        .client
        .efts_search(EftsSearchQuery {
            query: require_query(&params.query)?,
            ciks: cik.into_iter().collect(),
            forms: parse_forms(&forms),
            from: params.from,
            to: params.to,
            limit: params.limit.or(Some(20)),
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn facts(
    State(state): State<AppState>,
    Query(params): Query<FactsParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let records = state
        .client
        .facts(FactQuery {
            cik,
            concept: params.concept,
            form: params.form,
            unit: params.unit,
            latest: params.latest.unwrap_or(20),
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn statements(
    State(state): State<AppState>,
    Query(params): Query<StatementsParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let records = state
        .client
        .financial_statements(StatementQuery {
            cik,
            statement: params.statement.unwrap_or_else(|| "all".to_string()),
            form: period_form(params.period.as_deref()),
            unit: params.unit,
            latest: params.latest.unwrap_or(4),
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn metrics(
    State(state): State<AppState>,
    Query(params): Query<MetricsParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let records = state
        .client
        .financial_metrics(MetricsQuery {
            cik,
            form: period_form(params.period.as_deref()),
            unit: params.unit,
            latest: params.latest.unwrap_or(4),
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn company_report(
    State(state): State<AppState>,
    Query(params): Query<CompanyReportParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let records = state
        .client
        .company_reports(CompanyReportQuery {
            cik,
            form: Some(params.form.unwrap_or_else(|| "10-K".to_string())),
            latest: params.latest.unwrap_or(1),
            include_amends: params.include_amends.unwrap_or(false),
            topic: params.topic,
            limit_tables: params.limit_tables,
            limit_rows: params.limit_rows,
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn ixbrl(
    State(state): State<AppState>,
    Query(params): Query<IxbrlParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let records = state
        .client
        .inline_xbrl_facts(InlineXbrlQuery {
            cik,
            form: Some(params.form.unwrap_or_else(|| "10-K".to_string())),
            latest: params.latest.unwrap_or(1),
            include_amends: params.include_amends.unwrap_or(false),
            concept: params.concept,
            limit: params.limit,
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn sections(
    State(state): State<AppState>,
    Query(params): Query<SectionParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let records = state
        .client
        .sections(SectionQuery {
            cik,
            form: Some(params.form.unwrap_or_else(|| "10-K".to_string())),
            latest: params.latest.unwrap_or(1),
            include_amends: params.include_amends.unwrap_or(false),
            accession: params.accession,
            item: params.item,
            limit_bytes: params.limit_bytes,
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn docs(
    State(state): State<AppState>,
    Query(params): Query<DocumentParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let records = state
        .client
        .document_records(DocumentQuery {
            cik,
            form: params.form,
            latest: params.latest.unwrap_or(1),
            include_amends: params.include_amends.unwrap_or(false),
            limit: params.limit,
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn form4(
    State(state): State<AppState>,
    Query(params): Query<CompanyParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let query = form4_query(&state.client, &params).await?;
    Ok(Json(json!(state.client.form4_transactions(query).await?)))
}

async fn form4_summary(
    State(state): State<AppState>,
    Query(params): Query<CompanyParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let query = form4_query(&state.client, &params).await?;
    Ok(Json(json!(state.client.form4_reports(query).await?)))
}

async fn eightk(
    State(state): State<AppState>,
    Query(params): Query<EightKParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let records = state
        .client
        .eightk_events(EightKQuery {
            cik,
            latest: params.latest.unwrap_or(5),
            include_amends: params.include_amends.unwrap_or(false),
            item: params.item,
            limit_bytes: params.limit_bytes,
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn eightk_exhibits(
    State(state): State<AppState>,
    Query(params): Query<EightKExhibitParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let records = state
        .client
        .eightk_exhibits(EightKExhibitQuery {
            cik,
            latest: params.latest.unwrap_or(5),
            include_amends: params.include_amends.unwrap_or(false),
            category: params.category,
            limit_bytes: params.limit_bytes,
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn schedule13(
    State(state): State<AppState>,
    Query(params): Query<Schedule13Params>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let records = state
        .client
        .schedule13_reports(Schedule13Query {
            cik,
            form: params.form,
            latest: params.latest.unwrap_or(5),
            include_amends: params.include_amends.unwrap_or(false),
            limit_bytes: params.limit_bytes,
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn thirteenf(
    State(state): State<AppState>,
    Query(params): Query<CompanyParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let query = thirteenf_query(&state.client, &params).await?;
    Ok(Json(json!(state.client.thirteenf_holdings(query).await?)))
}

async fn thirteenf_summary(
    State(state): State<AppState>,
    Query(params): Query<CompanyParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let query = thirteenf_query(&state.client, &params).await?;
    Ok(Json(json!(state.client.thirteenf_reports(query).await?)))
}

async fn thirteenf_diff(
    State(state): State<AppState>,
    Query(params): Query<CompanyParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let query = thirteenf_query(&state.client, &params).await?;
    Ok(Json(json!(
        state.client.thirteenf_diff_holdings(query).await?
    )))
}

async fn proxy(
    State(state): State<AppState>,
    Query(params): Query<CompanyParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let records = state
        .client
        .proxy_statements(ProxyQuery {
            cik,
            latest: params.latest.unwrap_or(1),
            include_amends: params.include_amends.unwrap_or(false),
            limit_rows: params.limit,
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn prospectus(
    State(state): State<AppState>,
    Query(params): Query<DisclosureParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let records = state
        .client
        .prospectuses(ProspectusQuery {
            cik,
            form: Some(params.form.unwrap_or_else(|| "all".to_string())),
            latest: params.latest.unwrap_or(1),
            include_amends: params.include_amends.unwrap_or(false),
            limit_bytes: params.limit_bytes,
            limit_tables: params.limit_tables,
            limit_rows: params.limit_rows,
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn foreign(
    State(state): State<AppState>,
    Query(params): Query<DisclosureParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let records = state
        .client
        .foreign_issuer_reports(ForeignIssuerQuery {
            cik,
            form: Some(params.form.unwrap_or_else(|| "all".to_string())),
            latest: params.latest.unwrap_or(1),
            include_amends: params.include_amends.unwrap_or(false),
            limit_bytes: params.limit_bytes,
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn fund(
    State(state): State<AppState>,
    Query(params): Query<FundParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let records = state
        .client
        .fund_disclosures(FundDisclosureQuery {
            cik,
            form: Some(params.form.unwrap_or_else(|| "all".to_string())),
            latest: params.latest.unwrap_or(1),
            include_amends: params.include_amends.unwrap_or(false),
            limit_holdings: params.limit_holdings,
            limit_bytes: params.limit_bytes,
        })
        .await?;
    Ok(Json(json!(records)))
}

async fn parse(
    State(state): State<AppState>,
    Query(params): Query<ParseParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let records = state
        .client
        .parse_form(ParseQuery {
            cik,
            form: params.form,
            latest: params.latest.unwrap_or(1),
            include_amends: params.include_amends.unwrap_or(false),
            limit: params.limit,
        })
        .await?;
    Ok(Json(json!(records)))
}
