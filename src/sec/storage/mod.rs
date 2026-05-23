use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct FileStore {
    root: PathBuf,
}

impl FileStore {
    pub fn new(root: Option<PathBuf>) -> Result<Self> {
        let root = root.unwrap_or_else(default_cache_dir);
        fs::create_dir_all(&root)
            .with_context(|| format!("failed to create cache directory {}", root.display()))?;
        Ok(Self { root })
    }

    pub fn read_url(&self, url: &str, ext: &str, ttl: Option<Duration>) -> Result<Option<Vec<u8>>> {
        let path = self.url_path(url, ext);
        if !path.exists() {
            return Ok(None);
        }
        if ttl.is_some_and(|ttl| is_expired(&path, ttl)) {
            return Ok(None);
        }
        fs::read(&path)
            .map(Some)
            .with_context(|| format!("failed to read cache file {}", path.display()))
    }

    pub fn write_url(&self, url: &str, ext: &str, bytes: &[u8]) -> Result<()> {
        atomic_write(&self.url_path(url, ext), bytes)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn url_path(&self, url: &str, ext: &str) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let digest = hasher.finalize();
        self.root.join(format!("{digest:x}.{ext}"))
    }
}

fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sec-cli")
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(tmp, path)?;
    Ok(())
}

fn is_expired(path: &Path, ttl: Duration) -> bool {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age > ttl)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_cache_dir() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("sec-cli-storage-test-{nonce}"))
    }

    #[test]
    fn writes_and_reads_url_cache_entries() {
        let dir = temp_cache_dir();
        let store = FileStore::new(Some(dir.clone())).unwrap();
        store
            .write_url("https://example.com/a", "txt", b"hello")
            .unwrap();

        let bytes = store
            .read_url(
                "https://example.com/a",
                "txt",
                Some(Duration::from_secs(60)),
            )
            .unwrap()
            .unwrap();
        assert_eq!(bytes, b"hello");

        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn expired_ttl_returns_cache_miss() {
        let dir = temp_cache_dir();
        let store = FileStore::new(Some(dir.clone())).unwrap();
        store
            .write_url("https://example.com/a", "txt", b"hello")
            .unwrap();

        let miss = store
            .read_url("https://example.com/a", "txt", Some(Duration::ZERO))
            .unwrap();
        assert!(miss.is_none());

        fs::remove_dir_all(dir).ok();
    }
}
