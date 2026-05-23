use std::env;

use anyhow::Result;

pub(super) fn resolve_identity(cli_identity: Option<String>) -> Result<String> {
    resolve_identity_from(
        cli_identity,
        env::var("EDGAR_IDENTITY").ok(),
        env::var("SEC_IDENTITY").ok(),
    )
}

fn resolve_identity_from(
    cli_identity: Option<String>,
    edgar_identity: Option<String>,
    sec_identity: Option<String>,
) -> Result<String> {
    cli_identity
        .or(edgar_identity)
        .or(sec_identity)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "SEC identity is required. Set SEC_IDENTITY=\"Your Name your.email@example.com\" or pass --identity."
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
            )
            .unwrap(),
            "Alice alice@example.com"
        );
    }

    #[test]
    fn rejects_missing_identity() {
        assert!(resolve_identity_from(None, None, None).is_err());
    }
}
