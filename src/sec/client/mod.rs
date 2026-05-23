use std::{collections::HashMap, path::PathBuf, time::Duration};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, de::DeserializeOwned};

use super::{edgar::company_tickers_url, http::EdgarTransport, storage::FileStore};

#[derive(Clone)]
pub struct SecClient {
    transport: EdgarTransport,
    store: FileStore,
}

#[derive(Debug, Deserialize)]
struct TickerRecord {
    cik_str: u64,
    ticker: String,
    #[serde(rename = "title")]
    _title: String,
}

impl SecClient {
    pub fn new(identity: String, cache_dir: Option<PathBuf>) -> Result<Self> {
        Ok(Self {
            transport: EdgarTransport::new(identity)?,
            store: FileStore::new(cache_dir)?,
        })
    }

    pub async fn cik_for_ticker(&self, ticker: &str) -> Result<u64> {
        let data: HashMap<String, TickerRecord> = self.get_json(&company_tickers_url()).await?;
        let needle = ticker.trim().to_ascii_uppercase().replace('.', "-");

        data.values()
            .find(|record| record.ticker.eq_ignore_ascii_case(&needle))
            .map(|record| record.cik_str)
            .ok_or_else(|| anyhow!("ticker not found in SEC company_tickers.json"))
    }

    pub async fn get_text(&self, url: &str) -> Result<String> {
        self.cached_fetch(url, "txt", |bytes| {
            String::from_utf8(bytes.to_vec()).context("text response was not valid UTF-8")
        })
        .await
    }

    pub(crate) async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        self.cached_fetch(url, "json", |bytes| {
            serde_json::from_slice(bytes).context("failed to parse JSON response")
        })
        .await
    }

    pub fn cache_dir(&self) -> &std::path::Path {
        self.store.root()
    }

    async fn cached_fetch<T, F>(&self, url: &str, ext: &str, parse: F) -> Result<T>
    where
        F: Fn(&[u8]) -> Result<T>,
    {
        if let Some(bytes) = self.store.read_url(url, ext, cache_ttl(url))? {
            return parse(&bytes).with_context(|| format!("failed to parse cached {ext}: {url}"));
        }

        let bytes = self.transport.get_bytes(url).await?;
        self.store.write_url(url, ext, &bytes)?;
        parse(&bytes).with_context(|| format!("failed to parse {ext} response: {url}"))
    }
}

fn cache_ttl(url: &str) -> Option<Duration> {
    if url.contains("/Archives/edgar/data/") {
        None
    } else if url.ends_with("/company_tickers.json") {
        Some(Duration::from_secs(24 * 60 * 60))
    } else if url.contains("/submissions/") {
        Some(Duration::from_secs(60 * 60))
    } else {
        Some(Duration::from_secs(6 * 60 * 60))
    }
}
