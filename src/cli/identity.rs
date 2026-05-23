use std::env;

use anyhow::Result;

use super::config::configured_identity;

pub(super) fn resolve_identity(cli_identity: Option<String>) -> Result<String> {
    let configured_identity = configured_identity()?;
    resolve_identity_from(
        cli_identity,
        env::var("EDGAR_IDENTITY").ok(),
        env::var("SEC_IDENTITY").ok(),
        configured_identity,
    )
}

fn resolve_identity_from(
    cli_identity: Option<String>,
    edgar_identity: Option<String>,
    sec_identity: Option<String>,
    configured_identity: Option<String>,
) -> Result<String> {
    cli_identity
        .or(edgar_identity)
        .or(sec_identity)
        .or(configured_identity)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "SEC identity is required. Run `sec config set-identity \"Your Name your.email@example.com\"`, set SEC_IDENTITY, or pass --identity."
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_identity_takes_precedence() {
        assert_eq!(
            resolve_identity_from(
                Some("Alice alice@example.com".to_string()),
                Some("Bob bob@example.com".to_string()),
                None,
                Some("Config config@example.com".to_string()),
            )
            .unwrap(),
            "Alice alice@example.com"
        );
    }

    #[test]
    fn rejects_missing_identity() {
        assert!(resolve_identity_from(None, None, None, None).is_err());
    }

    #[test]
    fn uses_configured_identity_after_environment() {
        assert_eq!(
            resolve_identity_from(
                None,
                None,
                None,
                Some("Config config@example.com".to_string())
            )
            .unwrap(),
            "Config config@example.com"
        );
    }
}
