use std::{sync::LazyLock, time::Duration};

use anyhow::{Context, Result, anyhow, bail};
use reqwest::Client;
use serde_json::{Value, json};

use super::config::{LlmConfig, LlmProvider};

pub struct LlmClient {
    http: Client,
    config: LlmConfig,
}

impl LlmClient {
    pub fn new(config: LlmConfig) -> Self {
        Self {
            http: shared_http_client(),
            config,
        }
    }

    pub async fn complete(&self, system: &str, user: &str) -> Result<String> {
        match self.provider()? {
            LlmProvider::OpenAi => self.openai_complete(system, user).await,
            LlmProvider::Anthropic => self.anthropic_complete(system, user).await,
        }
    }

    fn provider(&self) -> Result<LlmProvider> {
        self.config.provider.ok_or_else(|| {
            anyhow!("LLM provider is not configured; set SEC_CLI_LLM_PROVIDER or llm.json provider")
        })
    }

    fn api_key(&self) -> Result<String> {
        self.config.resolved_api_key().ok_or_else(|| {
            anyhow!(
                "LLM API key is not configured; set api_key_env in llm.json or SEC_CLI_LLM_API_KEY"
            )
        })
    }

    fn model(&self) -> Result<&str> {
        self.config
            .model
            .as_deref()
            .filter(|model| !model.trim().is_empty())
            .ok_or_else(|| anyhow!("LLM model is not configured"))
    }

    async fn openai_complete(&self, system: &str, user: &str) -> Result<String> {
        let url = endpoint(
            self.config.base_url.as_deref(),
            "https://api.openai.com/v1",
            "/chat/completions",
        );
        let body = json!({
            "model": self.model()?,
            "temperature": 0,
            "max_tokens": self.config.max_tokens.unwrap_or(800),
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ]
        });
        let value = self
            .http
            .post(url)
            .bearer_auth(self.api_key()?)
            .json(&body)
            .send()
            .await
            .context("failed to call OpenAI-compatible LLM")?;
        parse_response(value).await.and_then(parse_openai_text)
    }

    async fn anthropic_complete(&self, system: &str, user: &str) -> Result<String> {
        let url = endpoint(
            self.config.base_url.as_deref(),
            "https://api.anthropic.com",
            "/v1/messages",
        );
        let body = json!({
            "model": self.model()?,
            "temperature": 0,
            "max_tokens": self.config.max_tokens.unwrap_or(800),
            "system": system,
            "messages": [{"role": "user", "content": user}]
        });
        let value = self
            .http
            .post(url)
            .header("x-api-key", self.api_key()?)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .context("failed to call Anthropic-compatible LLM")?;
        parse_response(value).await.and_then(parse_anthropic_text)
    }
}

fn shared_http_client() -> Client {
    static CLIENT: LazyLock<Client> = LazyLock::new(|| {
        Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(60))
            .build()
            .expect("valid LLM reqwest client")
    });
    CLIENT.clone()
}

fn endpoint(base_url: Option<&str>, default_base: &str, suffix: &str) -> String {
    let base = base_url
        .filter(|url| !url.trim().is_empty())
        .unwrap_or(default_base)
        .trim_end_matches('/');
    if base.ends_with(suffix.trim_start_matches('/')) {
        base.to_string()
    } else {
        format!("{base}{suffix}")
    }
}

async fn parse_response(response: reqwest::Response) -> Result<Value> {
    let status = response.status();
    let text = response
        .text()
        .await
        .context("failed to read LLM response")?;
    if !status.is_success() {
        bail!(
            "LLM request failed with HTTP {}: {}",
            status,
            truncate(&text)
        );
    }
    serde_json::from_str(&text).context("LLM response was not JSON")
}

fn parse_openai_text(value: Value) -> Result<String> {
    value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("OpenAI-compatible response missing choices[0].message.content"))
}

fn parse_anthropic_text(value: Value) -> Result<String> {
    value
        .get("content")
        .and_then(Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .find(|item| item.get("type").and_then(Value::as_str) == Some("text"))
        })
        .and_then(|item| item.get("text"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("Anthropic-compatible response missing text content"))
}

fn truncate(text: &str) -> String {
    const MAX: usize = 500;
    if text.chars().count() <= MAX {
        text.to_string()
    } else {
        format!("{}...", text.chars().take(MAX).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_compatible_endpoints() {
        assert_eq!(
            endpoint(
                Some("https://open.bigmodel.cn/api/coding/paas/v4"),
                "",
                "/chat/completions"
            ),
            "https://open.bigmodel.cn/api/coding/paas/v4/chat/completions"
        );
        assert_eq!(
            endpoint(
                Some("https://open.bigmodel.cn/api/anthropic"),
                "",
                "/v1/messages"
            ),
            "https://open.bigmodel.cn/api/anthropic/v1/messages"
        );
    }
}
