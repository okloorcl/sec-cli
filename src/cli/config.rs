use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub(super) struct CliConfig {
    pub(super) identity: Option<String>,
}

pub(super) fn configured_identity() -> Result<Option<String>> {
    Ok(read_config()?.identity)
}

pub(super) fn set_identity(identity: String) -> Result<CliConfig> {
    let mut config = read_config()?;
    config.identity = Some(identity.trim().to_string());
    write_config(&config)?;
    Ok(config)
}

pub(super) fn read_config() -> Result<CliConfig> {
    let path = config_path();
    if !path.exists() {
        return Ok(CliConfig::default());
    }
    let bytes =
        fs::read(&path).with_context(|| format!("failed to read config {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse config {}", path.display()))
}

pub(super) fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sec-cli")
        .join("config.json")
}

fn write_config(config: &CliConfig) -> Result<()> {
    let path = config_path();
    let parent = path
        .parent()
        .context("sec-cli config path has no parent directory")?;
    create_private_dir(parent)?;
    let bytes = serde_json::to_vec_pretty(config)?;
    fs::write(&path, bytes)
        .with_context(|| format!("failed to write config {}", path.display()))?;
    restrict_file_permissions(&path)
}

fn create_private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    restrict_dir_permissions(path)
}

#[cfg(unix)]
fn restrict_dir_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .with_context(|| format!("failed to restrict config directory {}", path.display()))
}

#[cfg(not(unix))]
fn restrict_dir_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn restrict_file_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("failed to restrict config file {}", path.display()))
}

#[cfg(not(unix))]
fn restrict_file_permissions(_path: &Path) -> Result<()> {
    Ok(())
}
