use axum::{
    Json,
    extract::{Query, State},
};
use serde_json::json;

use super::{
    AppState,
    errors::ApiResult,
    helpers::resolve_cik,
    params::{AgentPackParams, sections_list},
};
use crate::sec::AgentPackQuery;

pub(super) async fn agent_pack(
    State(state): State<AppState>,
    Query(params): Query<AgentPackParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let cik = resolve_cik(&state.client, params.ticker.as_deref(), params.cik).await?;
    let record = state
        .client
        .agent_pack(AgentPackQuery {
            cik,
            form: params.form.unwrap_or_else(|| "10-K".to_string()),
            latest: params.latest.unwrap_or(1),
            sections: sections_list(params.sections),
            section_limit_bytes: params.section_limit_bytes.or(Some(20_000)),
            metrics_latest: params.metrics_latest.unwrap_or(4),
        })
        .await?;
    Ok(Json(json!(record)))
}
