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
        create_dir_all_private(&root)
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
        create_dir_all_private(parent)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes)?;
    restrict_file_permissions(&tmp)?;
    fs::rename(tmp, path)?;
    restrict_file_permissions(path)?;
    Ok(())
}

fn create_dir_all_private(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    restrict_dir_permissions(path)
}

#[cfg(unix)]
fn restrict_dir_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .with_context(|| format!("failed to restrict cache directory {}", path.display()))
}

#[cfg(not(unix))]
fn restrict_dir_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn restrict_file_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("failed to restrict cache file {}", path.display()))
}

#[cfg(not(unix))]
fn restrict_file_permissions(_path: &Path) -> Result<()> {
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
    use std::sync::atomic::{AtomicU64, Ordering};

    fn temp_cache_dir() -> PathBuf {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sec-cli-storage-test-{}-{id}", std::process::id()))
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

    #[cfg(unix)]
    #[test]
    fn cache_files_and_dirs_are_private() {
        use std::os::unix::fs::PermissionsExt;

        let dir = temp_cache_dir();
        let store = FileStore::new(Some(dir.clone())).unwrap();
        store
            .write_url("https://example.com/private", "json", b"{}")
            .unwrap();

        let dir_mode = fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(dir_mode, 0o700);

        let entry = fs::read_dir(&dir).unwrap().next().unwrap().unwrap().path();
        let file_mode = fs::metadata(entry).unwrap().permissions().mode() & 0o777;
        assert_eq!(file_mode, 0o600);

        fs::remove_dir_all(dir).ok();
    }
}
