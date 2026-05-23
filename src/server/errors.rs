use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

pub(super) type ApiResult<T> = std::result::Result<T, ApiError>;

pub(super) struct ApiError(anyhow::Error);

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            status_for_error(&self.0),
            Json(json!({"error": self.0.to_string()})),
        )
            .into_response()
    }
}

fn status_for_error(error: &anyhow::Error) -> StatusCode {
    let message = error.to_string().to_ascii_lowercase();
    if message.contains("provide ")
        || message.contains("missing required")
        || message.contains("unsupported")
        || message.contains("cannot be empty")
        || message.contains("not both")
    {
        StatusCode::BAD_REQUEST
    } else if message.contains("not found") || message.contains("no filing") {
        StatusCode::NOT_FOUND
    } else if message.contains("sec request failed")
        || message.contains("request failed")
        || message.contains("timed out")
        || error.chain().any(|cause| cause.is::<reqwest::Error>())
    {
        StatusCode::BAD_GATEWAY
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_api_errors_to_useful_status_codes() {
        assert_eq!(
            status_for_error(&anyhow::anyhow!("provide ticker or cik")),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            status_for_error(&anyhow::anyhow!(
                "ticker not found in SEC company_tickers.json"
            )),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status_for_error(&anyhow::anyhow!("SEC request failed with status 503")),
            StatusCode::BAD_GATEWAY
        );
        assert_eq!(
            status_for_error(&anyhow::anyhow!("unexpected parser bug")),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
