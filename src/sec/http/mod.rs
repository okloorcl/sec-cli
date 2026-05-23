use anyhow::{Result, anyhow};
use reqwest::header::{ACCEPT_ENCODING, HeaderMap, HeaderValue, USER_AGENT};

#[derive(Clone)]
pub struct EdgarTransport {
    client: reqwest::Client,
}

impl EdgarTransport {
    pub fn new(identity: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_str(&identity)?);
        headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .use_rustls_tls()
            .build()?;

        Ok(Self { client })
    }

    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let response = self.client.get(url).send().await?;
        let status = response.status();
        if !status.is_success() {
            return Err(anyhow!(
                "SEC request failed with status {} for {}",
                status,
                url
            ));
        }
        Ok(response.bytes().await?.to_vec())
    }
}
