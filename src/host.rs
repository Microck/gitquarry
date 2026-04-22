use crate::error::{AppError, AppResult};
use serde::Serialize;
use url::Url;

#[derive(Debug, Clone, Serialize)]
pub struct HostContext {
    pub web_host: String,
    pub api_base: String,
    pub raw_input: String,
}

pub fn normalize_host(input: Option<&str>) -> AppResult<HostContext> {
    let raw_input = input.unwrap_or("github.com").trim();

    if raw_input.is_empty() {
        return Err(AppError::new("E_HOST_INVALID", "host must not be empty"));
    }

    let parsed = if raw_input.contains("://") {
        Url::parse(raw_input).map_err(|err| {
            AppError::with_detail("E_HOST_INVALID", "invalid host URL", err.to_string())
        })?
    } else {
        Url::parse(&format!("https://{raw_input}")).map_err(|err| {
            AppError::with_detail("E_HOST_INVALID", "invalid host value", err.to_string())
        })?
    };

    let host = parsed
        .host_str()
        .ok_or_else(|| AppError::new("E_HOST_INVALID", "host must include a hostname"))?;

    let host_with_port = match parsed.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_string(),
    };

    let scheme = parsed.scheme();

    if matches!(host, "github.com" | "www.github.com" | "api.github.com") {
        return Ok(HostContext {
            web_host: "github.com".to_string(),
            api_base: "https://api.github.com".to_string(),
            raw_input: raw_input.to_string(),
        });
    }

    let api_base = format!("{scheme}://{host_with_port}/api/v3");

    Ok(HostContext {
        web_host: host_with_port,
        api_base,
        raw_input: raw_input.to_string(),
    })
}

pub fn token_env_var_for_host(host: &str) -> String {
    let suffix = host
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>();

    format!("GITQUARRY_TOKEN_{suffix}")
}

#[cfg(test)]
mod tests {
    use super::{normalize_host, token_env_var_for_host};

    #[test]
    fn normalizes_github_dot_com() {
        let host = normalize_host(Some("github.com")).unwrap();
        assert_eq!(host.web_host, "github.com");
        assert_eq!(host.api_base, "https://api.github.com");
    }

    #[test]
    fn normalizes_custom_ghe_url() {
        let host = normalize_host(Some("https://git.example.com/api/v3")).unwrap();
        assert_eq!(host.web_host, "git.example.com");
        assert_eq!(host.api_base, "https://git.example.com/api/v3");
    }

    #[test]
    fn normalizes_localhost_for_tests() {
        let host = normalize_host(Some("http://127.0.0.1:8787/api/v3")).unwrap();
        assert_eq!(host.web_host, "127.0.0.1:8787");
        assert_eq!(host.api_base, "http://127.0.0.1:8787/api/v3");
    }

    #[test]
    fn preserves_scheme_for_full_url_hosts() {
        let host = normalize_host(Some("http://git.example.com")).unwrap();
        assert_eq!(host.web_host, "git.example.com");
        assert_eq!(host.api_base, "http://git.example.com/api/v3");
    }

    #[test]
    fn derives_host_specific_env_var() {
        assert_eq!(
            token_env_var_for_host("github.com"),
            "GITQUARRY_TOKEN_GITHUB_COM"
        );
        assert_eq!(
            token_env_var_for_host("127.0.0.1:8787"),
            "GITQUARRY_TOKEN_127_0_0_1_8787"
        );
    }
}
