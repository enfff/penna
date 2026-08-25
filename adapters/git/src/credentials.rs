use penna_core::ports::RepositoryError;

pub const KEYCHAIN_SERVICE: &str = "penna";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedCredential {
    SshAgent,
    Token(String),
    NoCredential,
}

pub fn is_https_remote(remote_url: &str) -> bool {
    remote_url.starts_with("https://") || remote_url.starts_with("http://")
}

pub fn is_ssh_remote(remote_url: &str) -> bool {
    !is_https_remote(remote_url)
        && (remote_url.starts_with("ssh://")
            || (remote_url.contains('@') && !remote_url.starts_with("file://")))
}

pub fn resolve_https_token(
    env_token: Option<&str>,
    keychain_token: Option<String>,
    remote_url: &str,
) -> Result<ResolvedCredential, RepositoryError> {
    if let Some(token) = env_token.map(str::trim).filter(|t| !t.is_empty()) {
        return Ok(ResolvedCredential::Token(token.to_string()));
    }

    if let Some(token) = keychain_token {
        return Ok(ResolvedCredential::Token(token));
    }

    Err(RepositoryError::AuthRequired(remote_url.to_string()))
}

pub fn resolve_credentials(
    remote_url: &str,
    env_token: Option<&str>,
    keychain_token: Option<String>,
) -> Result<ResolvedCredential, RepositoryError> {
    if is_ssh_remote(remote_url) {
        return Ok(ResolvedCredential::SshAgent);
    }

    if !is_https_remote(remote_url) {
        return Ok(ResolvedCredential::NoCredential);
    }

    resolve_https_token(env_token, keychain_token, remote_url)
}

pub fn lookup_keychain_token(remote_url: &str) -> Option<String> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, remote_url).ok()?;
    entry.get_password().ok()
}

pub fn store_keychain_token(remote_url: &str, token: &str) -> Result<(), RepositoryError> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, remote_url)
        .map_err(|e| RepositoryError::Storage(format!("Failed to open keychain entry: {e}")))?;

    entry
        .set_password(token)
        .map_err(|e| RepositoryError::Storage(format!("Failed to store token in keychain: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_url_forms_route_to_ssh_agent() {
        for url in [
            "git@git.example.com:user/journal.git",
            "ssh://git@host.example.com/journal.git",
            "user@server:/path/to/journal",
        ] {
            let resolved = resolve_credentials(url, None, None).unwrap();
            assert_eq!(resolved, ResolvedCredential::SshAgent, "url: {}", url);
        }
    }

    #[test]
    fn local_paths_and_file_urls_need_no_credentials() {
        for url in [
            "/tmp/bare-remote",
            "file:///tmp/bare-remote",
            "./sibling-journal.git",
            "git://host.example.com/journal.git",
        ] {
            let resolved = resolve_credentials(url, None, None).unwrap();
            assert_eq!(
                resolved,
                ResolvedCredential::NoCredential,
                "url: {}",
                url
            );
        }
    }

    #[test]
    fn https_urls_do_not_route_to_ssh_agent() {
        assert!(!is_ssh_remote("https://git.example.com/user/journal.git"));
        assert!(!is_ssh_remote("http://host/journal.git"));
    }

    #[test]
    fn env_var_wins_over_keychain() {
        let resolved = resolve_https_token(
            Some("  env-token  "),
            Some("stored-token".to_string()),
            "https://git.example.com/u/journal.git",
        )
        .unwrap();

        assert_eq!(
            resolved,
            ResolvedCredential::Token("env-token".to_string()),
            "trimmed env token must take precedence"
        );
    }

    #[test]
    fn blank_env_falls_through_to_keychain() {
        let resolved = resolve_https_token(
            Some("   "),
            Some("stored-token".to_string()),
            "https://git.example.com/u/journal.git",
        )
        .unwrap();

        assert_eq!(resolved, ResolvedCredential::Token("stored-token".to_string()));
    }

    #[test]
    fn missing_everything_yields_auth_required_with_remote_url() {
        let err = resolve_https_token(None, None, "https://git.example.com/u/journal.git").unwrap_err();

        match err {
            RepositoryError::AuthRequired(url) => {
                assert_eq!(url, "https://git.example.com/u/journal.git")
            }
            other => panic!("expected AuthRequired, got {:?}", other),
        }
    }
}
