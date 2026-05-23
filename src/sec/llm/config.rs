use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    OpenAi,
    Anthropic,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LlmConfig {
    pub provider: Option<LlmProvider>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub api_key_env: Option<String>,
    pub max_tokens: Option<u32>,
}

impl LlmConfig {
    pub fn load_with_overrides(overrides: Option<LlmConfig>) -> Result<Self> {
        let mut config = Self::load_file()?.unwrap_or_default();
        config.apply_env();
        if let Some(overrides) = overrides {
            config.merge(overrides);
        }
        Ok(config)
    }

    pub fn resolved_api_key(&self) -> Option<String> {
        if let Some(key) = self.api_key.as_ref().filter(|key| !key.trim().is_empty()) {
            return Some(key.clone());
        }
        let env_name = self
            .api_key_env
            .as_deref()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or("SEC_CLI_LLM_API_KEY");
        env::var(env_name).ok().filter(|key| !key.trim().is_empty())
    }

    pub fn merge(&mut self, other: LlmConfig) {
        self.provider = other.provider.or(self.provider);
        self.base_url = other.base_url.or_else(|| self.base_url.take());
        self.model = other.model.or_else(|| self.model.take());
        self.api_key = other.api_key.or_else(|| self.api_key.take());
        self.api_key_env = other.api_key_env.or_else(|| self.api_key_env.take());
        self.max_tokens = other.max_tokens.or(self.max_tokens);
    }

    fn load_file() -> Result<Option<Self>> {
        let path = config_path();
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(&path)
            .with_context(|| format!("failed to read LLM config {}", path.display()))?;
        let config = serde_json::from_slice(&bytes)
            .with_context(|| format!("failed to parse LLM config {}", path.display()))?;
        Ok(Some(config))
    }

    fn apply_env(&mut self) {
        if let Ok(provider) = env::var("SEC_CLI_LLM_PROVIDER") {
            self.provider = parse_provider(&provider);
        }
        set_from_env(&mut self.base_url, "SEC_CLI_LLM_BASE_URL");
        set_from_env(&mut self.model, "SEC_CLI_LLM_MODEL");
        set_from_env(&mut self.api_key, "SEC_CLI_LLM_API_KEY");
        set_from_env(&mut self.api_key_env, "SEC_CLI_LLM_API_KEY_ENV");
    }
}

fn config_path() -> PathBuf {
    env::var("SEC_CLI_LLM_CONFIG")
        .map(PathBuf::from)
        .ok()
        .or_else(|| dirs::config_dir().map(|dir| dir.join("sec-cli").join("llm.json")))
        .unwrap_or_else(|| PathBuf::from("llm.json"))
}

fn set_from_env(target: &mut Option<String>, name: &str) {
    if let Ok(value) = env::var(name)
        && !value.trim().is_empty()
    {
        *target = Some(value);
    }
}

fn parse_provider(value: &str) -> Option<LlmProvider> {
    match value.trim().to_ascii_lowercase().as_str() {
        "openai" | "openai-compatible" => Some(LlmProvider::OpenAi),
        "anthropic" | "anthropic-compatible" => Some(LlmProvider::Anthropic),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_provider_names() {
        assert_eq!(parse_provider("openai"), Some(LlmProvider::OpenAi));
        assert_eq!(parse_provider("anthropic"), Some(LlmProvider::Anthropic));
        assert_eq!(parse_provider("other"), None);
    }
}
