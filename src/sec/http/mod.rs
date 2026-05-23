use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use reqwest::{
    StatusCode,
    header::{ACCEPT_ENCODING, HeaderMap, HeaderValue, USER_AGENT},
};
use tokio::sync::Mutex;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(200);
const MAX_ATTEMPTS: usize = 4;

#[derive(Clone)]
pub struct EdgarTransport {
    client: reqwest::Client,
    limiter: Arc<Mutex<Instant>>,
}

impl EdgarTransport {
    pub fn new(identity: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_str(&identity)?);
        headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .use_rustls_tls()
            .build()?;

        Ok(Self {
            client,
            limiter: Arc::new(Mutex::new(Instant::now() - MIN_REQUEST_INTERVAL)),
        })
    }

    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let mut last_error = None;
        for attempt in 1..=MAX_ATTEMPTS {
            self.wait_for_slot().await;
            match self.client.get(url).send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        return Ok(response.bytes().await?.to_vec());
                    }
                    if !is_retryable(status) || attempt == MAX_ATTEMPTS {
                        return Err(anyhow!(
                            "SEC request failed with status {} for {}",
                            status,
                            url
                        ));
                    }
                    last_error = Some(anyhow!("SEC request returned retryable status {status}"));
                }
                Err(error) => {
                    if attempt == MAX_ATTEMPTS {
                        return Err(error).map_err(Into::into);
                    }
                    last_error = Some(error.into());
                }
            }
            tokio::time::sleep(backoff(attempt)).await;
        }
        Err(last_error.unwrap_or_else(|| anyhow!("SEC request failed for {url}")))
    }

    async fn wait_for_slot(&self) {
        let mut last_request = self.limiter.lock().await;
        let elapsed = last_request.elapsed();
        if elapsed < MIN_REQUEST_INTERVAL {
            tokio::time::sleep(MIN_REQUEST_INTERVAL - elapsed).await;
        }
        *last_request = Instant::now();
    }
}

fn is_retryable(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn backoff(attempt: usize) -> Duration {
    let millis = 250_u64.saturating_mul(1 << attempt.saturating_sub(1));
    Duration::from_millis(millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retries_rate_limit_and_server_errors() {
        assert!(is_retryable(StatusCode::TOO_MANY_REQUESTS));
        assert!(is_retryable(StatusCode::BAD_GATEWAY));
        assert!(!is_retryable(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn backoff_increases_by_attempt() {
        assert_eq!(backoff(1), Duration::from_millis(250));
        assert_eq!(backoff(2), Duration::from_millis(500));
        assert_eq!(backoff(3), Duration::from_millis(1000));
    }
}
