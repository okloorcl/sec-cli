use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

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
        if let Some(bytes) = self.store.read_url(url, "txt")? {
            return String::from_utf8(bytes).with_context(|| "cached text was not valid UTF-8");
        }

        let bytes = self.transport.get_bytes(url).await?;
        self.store.write_url(url, "txt", &bytes)?;
        String::from_utf8(bytes)
            .with_context(|| format!("text response was not valid UTF-8: {url}"))
    }

    pub(crate) async fn get_json<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T> {
        if let Some(bytes) = self.store.read_url(url, "json")? {
            return serde_json::from_slice(&bytes)
                .with_context(|| format!("failed to parse cached JSON for {}", url));
        }

        let bytes = self.transport.get_bytes(url).await?;
        self.store.write_url(url, "json", &bytes)?;
        serde_json::from_slice(&bytes).with_context(|| format!("failed to parse JSON from {}", url))
    }

    pub fn cache_dir(&self) -> &std::path::Path {
        self.store.root()
    }
}
