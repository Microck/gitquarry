use crate::config::ConfigBundle;
use crate::error::{AppError, AppResult};
use crate::host::{HostContext, token_env_var_for_host};
use crate::model::CredentialSource;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[derive(Debug, Clone)]
pub struct CredentialResolution {
    pub token: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CredentialFile {
    hosts: BTreeMap<String, String>,
}

pub fn resolve_token(host: &HostContext, config: &ConfigBundle) -> AppResult<CredentialResolution> {
    if let Ok(token) = std::env::var(token_env_var_for_host(&host.web_host))
        && !token.trim().is_empty()
    {
        return Ok(CredentialResolution { token });
    }

    if let Ok(token) = std::env::var("GITQUARRY_TOKEN")
        && !token.trim().is_empty()
    {
        return Ok(CredentialResolution { token });
    }

    match try_read_keyring(host) {
        Ok(Some(token)) => return Ok(CredentialResolution { token }),
        Ok(None) => {}
        Err(err) if allow_insecure_storage() && keyring_read_is_unavailable(&err) => {
            if allow_insecure_storage()
                && let Some(token) = read_insecure_file(host, config)?
            {
                return Ok(CredentialResolution { token });
            }
        }
        Err(err) => {
            return Err(keyring_storage_error(
                "failed to read token from keyring",
                err,
            ));
        }
    }

    if allow_insecure_storage()
        && let Some(token) = read_insecure_file(host, config)?
    {
        return Ok(CredentialResolution { token });
    }

    Err(AppError::new(
        "E_AUTH_REQUIRED",
        format!(
            "auth required for host {}; run gitquarry auth login --host {}",
            host.web_host, host.web_host
        ),
    ))
}

pub fn saved_credential_source(
    host: &HostContext,
    config: &ConfigBundle,
) -> AppResult<Option<CredentialSource>> {
    match try_read_keyring(host) {
        Ok(Some(_)) => return Ok(Some(CredentialSource::Keyring)),
        Ok(None) => {}
        Err(err) if allow_insecure_storage() && keyring_read_is_unavailable(&err) => {
            if allow_insecure_storage() && read_insecure_file(host, config)?.is_some() {
                return Ok(Some(CredentialSource::InsecureFile));
            }
        }
        Err(err) => {
            return Err(keyring_storage_error(
                "failed to read token from keyring",
                err,
            ));
        }
    }

    if allow_insecure_storage() && read_insecure_file(host, config)?.is_some() {
        return Ok(Some(CredentialSource::InsecureFile));
    }

    Ok(None)
}

pub fn env_credential_source(host: &HostContext) -> Option<CredentialSource> {
    if std::env::var(token_env_var_for_host(&host.web_host))
        .ok()
        .filter(|value| !value.trim().is_empty())
        .is_some()
    {
        return Some(CredentialSource::EnvHost);
    }

    if std::env::var("GITQUARRY_TOKEN")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .is_some()
    {
        return Some(CredentialSource::EnvGlobal);
    }

    None
}

pub fn save_token(
    host: &HostContext,
    token: &str,
    config: &ConfigBundle,
) -> AppResult<CredentialSource> {
    save_token_with_attempt(
        host,
        token,
        config,
        allow_insecure_storage(),
        write_keyring,
        read_keyring,
    )
}

fn save_token_with_attempt<F, G>(
    host: &HostContext,
    token: &str,
    config: &ConfigBundle,
    allow_insecure_fallback: bool,
    write_secure: F,
    read_secure: G,
) -> AppResult<CredentialSource>
where
    F: FnOnce(&HostContext, &str) -> Result<(), keyring::Error>,
    G: Fn(&HostContext) -> AppResult<Option<String>>,
{
    if token.trim().is_empty() {
        return Err(AppError::new("E_AUTH_INVALID", "token must not be empty"));
    }

    match write_secure(host, token) {
        Err(err) if keyring_write_is_unavailable(&err) => {
            if allow_insecure_fallback {
                write_insecure_file(host, token, config)?;
                Ok(CredentialSource::InsecureFile)
            } else {
                Err(AppError::with_detail(
                    "E_AUTH_STORAGE",
                    "secure credential storage is unavailable; opt in to insecure file fallback with GITQUARRY_ALLOW_INSECURE_STORAGE=1",
                    err.to_string(),
                ))
            }
        }
        Ok(()) => match read_secure(host)? {
            Some(saved) if saved == token => Ok(CredentialSource::Keyring),
            Some(_) => Err(AppError::new(
                "E_AUTH_STORAGE",
                "failed to verify saved token in secure storage",
            )),
            None => {
                if allow_insecure_fallback {
                    write_insecure_file(host, token, config)?;
                    Ok(CredentialSource::InsecureFile)
                } else {
                    Err(AppError::new(
                        "E_AUTH_STORAGE",
                        "secure credential storage is unavailable; opt in to insecure file fallback with GITQUARRY_ALLOW_INSECURE_STORAGE=1",
                    ))
                }
            }
        },
        Err(err) => Err(AppError::with_detail(
            "E_AUTH_STORAGE",
            "failed to save token to keyring",
            err.to_string(),
        )),
    }
}

pub fn delete_token(host: &HostContext, config: &ConfigBundle) -> AppResult<bool> {
    let deleted_insecure = delete_insecure_file(host, config)?;
    match delete_keyring(host) {
        Ok(deleted_keyring) => Ok(deleted_keyring || deleted_insecure),
        Err(_) if deleted_insecure => Ok(true),
        Err(error) => Err(error),
    }
}

fn read_keyring(host: &HostContext) -> AppResult<Option<String>> {
    try_read_keyring(host)
        .map_err(|err| keyring_storage_error("failed to read token from keyring", err))
}

fn try_read_keyring(host: &HostContext) -> Result<Option<String>, keyring::Error> {
    let entry = match keyring::Entry::new("gitquarry", &host.web_host) {
        Ok(entry) => entry,
        Err(keyring::Error::NoEntry) => return Ok(None),
        Err(err) => return Err(err),
    };

    match entry.get_password() {
        Ok(value) if !value.trim().is_empty() => Ok(Some(value)),
        Ok(_) => Ok(None),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(err),
    }
}

fn write_keyring(host: &HostContext, token: &str) -> Result<(), keyring::Error> {
    let entry = keyring::Entry::new("gitquarry", &host.web_host)?;
    entry.set_password(token)
}

fn delete_keyring(host: &HostContext) -> AppResult<bool> {
    let entry = match keyring::Entry::new("gitquarry", &host.web_host) {
        Ok(entry) => entry,
        Err(keyring::Error::NoEntry) => return Ok(false),
        Err(err) => {
            return Err(AppError::with_detail(
                "E_AUTH_STORAGE",
                "failed to initialize credential entry",
                err.to_string(),
            ));
        }
    };

    match entry.delete_credential() {
        Ok(()) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(err) => Err(AppError::with_detail(
            "E_AUTH_STORAGE",
            "failed to delete token from keyring",
            err.to_string(),
        )),
    }
}

fn read_insecure_file(host: &HostContext, config: &ConfigBundle) -> AppResult<Option<String>> {
    if !config.paths.credentials_file.exists() {
        return Ok(None);
    }

    let file = load_insecure_file(config)?;
    Ok(file.hosts.get(&host.web_host).cloned())
}

fn write_insecure_file(host: &HostContext, token: &str, config: &ConfigBundle) -> AppResult<()> {
    config.ensure_parent_dirs()?;

    let mut file = load_insecure_file(config)?;

    file.hosts.insert(host.web_host.clone(), token.to_string());
    let raw = toml::to_string_pretty(&file).map_err(|err| {
        AppError::with_detail(
            "E_AUTH_STORAGE",
            "failed to serialize insecure credential file",
            err.to_string(),
        )
    })?;
    fs::write(&config.paths.credentials_file, raw).map_err(|err| {
        AppError::with_detail(
            "E_AUTH_STORAGE",
            "failed to write insecure credential file",
            err.to_string(),
        )
    })?;
    set_private_file_permissions(&config.paths.credentials_file)
}

fn delete_insecure_file(host: &HostContext, config: &ConfigBundle) -> AppResult<bool> {
    if !config.paths.credentials_file.exists() {
        return Ok(false);
    }

    let mut file = load_insecure_file(config)?;
    let removed = file.hosts.remove(&host.web_host).is_some();
    let raw = toml::to_string_pretty(&file).map_err(|err| {
        AppError::with_detail(
            "E_AUTH_STORAGE",
            "failed to serialize insecure credential file",
            err.to_string(),
        )
    })?;
    fs::write(&config.paths.credentials_file, raw).map_err(|err| {
        AppError::with_detail(
            "E_AUTH_STORAGE",
            "failed to write insecure credential file",
            err.to_string(),
        )
    })?;
    set_private_file_permissions(&config.paths.credentials_file)?;
    Ok(removed)
}

fn allow_insecure_storage() -> bool {
    std::env::var("GITQUARRY_ALLOW_INSECURE_STORAGE")
        .ok()
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}

fn keyring_write_is_unavailable(err: &keyring::Error) -> bool {
    matches!(
        err,
        keyring::Error::NoStorageAccess(_) | keyring::Error::NoEntry
    )
}

fn keyring_read_is_unavailable(err: &keyring::Error) -> bool {
    matches!(err, keyring::Error::NoStorageAccess(_))
}

fn keyring_storage_error(message: &str, err: keyring::Error) -> AppError {
    let detail = if matches!(err, keyring::Error::NoEntry) {
        format!("failed to initialize credential entry: {err}")
    } else {
        err.to_string()
    };
    AppError::with_detail("E_AUTH_STORAGE", message, detail)
}

fn load_insecure_file(config: &ConfigBundle) -> AppResult<CredentialFile> {
    if !config.paths.credentials_file.exists() {
        return Ok(CredentialFile::default());
    }

    let raw = fs::read_to_string(&config.paths.credentials_file).map_err(|err| {
        AppError::with_detail(
            "E_AUTH_STORAGE",
            "failed to read insecure credential file",
            err.to_string(),
        )
    })?;
    toml::from_str(&raw).map_err(|err| {
        AppError::with_detail(
            "E_AUTH_STORAGE",
            "failed to parse insecure credential file",
            err.to_string(),
        )
    })
}

#[cfg(unix)]
fn set_private_file_permissions(path: &std::path::Path) -> AppResult<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|err| {
        AppError::with_detail(
            "E_AUTH_STORAGE",
            "failed to restrict insecure credential file permissions",
            err.to_string(),
        )
    })
}

#[cfg(not(unix))]
fn set_private_file_permissions(_path: &std::path::Path) -> AppResult<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        CredentialFile, delete_insecure_file, load_insecure_file, save_token_with_attempt,
        write_insecure_file,
    };
    use crate::config::{ConfigBundle, ConfigFile, ConfigPaths};
    use crate::host::HostContext;
    use std::fs;
    use std::io;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    fn fixture_host() -> HostContext {
        HostContext {
            web_host: "git.example.test".to_string(),
            api_base: "https://git.example.test/api/v3".to_string(),
            raw_input: "https://git.example.test".to_string(),
        }
    }

    fn fixture_config(temp: &TempDir) -> ConfigBundle {
        let dir = temp.path().to_path_buf();
        ConfigBundle {
            paths: ConfigPaths {
                dir: dir.clone(),
                config_file: dir.join("config.toml"),
                credentials_file: dir.join("credentials.toml"),
            },
            data: ConfigFile::default(),
        }
    }

    #[test]
    fn save_token_falls_back_only_when_secure_storage_is_unavailable() {
        let temp = TempDir::new().unwrap();
        let config = fixture_config(&temp);
        let host = fixture_host();

        let source = save_token_with_attempt(
            &host,
            "fixture-token",
            &config,
            true,
            |_, _| {
                Err(keyring::Error::NoStorageAccess(Box::new(io::Error::other(
                    "keychain locked",
                ))))
            },
            |_| Ok(None),
        )
        .unwrap();

        assert!(matches!(
            source,
            crate::model::CredentialSource::InsecureFile
        ));
        let raw = fs::read_to_string(&config.paths.credentials_file).unwrap();
        let file: CredentialFile = toml::from_str(&raw).unwrap();
        assert_eq!(file.hosts.get(&host.web_host).unwrap(), "fixture-token");
    }

    #[test]
    fn save_token_does_not_fallback_on_other_keyring_errors() {
        let temp = TempDir::new().unwrap();
        let config = fixture_config(&temp);
        let host = fixture_host();

        let err = save_token_with_attempt(
            &host,
            "fixture-token",
            &config,
            true,
            |_, _| {
                Err(keyring::Error::Invalid(
                    "service".to_string(),
                    "backend rejected write".to_string(),
                ))
            },
            |_| Ok(None),
        )
        .unwrap_err();

        assert_eq!(err.code, "E_AUTH_STORAGE");
        assert!(err.message.contains("failed to save token to keyring"));
        assert!(!config.paths.credentials_file.exists());
    }

    #[test]
    fn save_token_falls_back_when_secure_backend_does_not_persist_across_reads() {
        let temp = TempDir::new().unwrap();
        let config = fixture_config(&temp);
        let host = fixture_host();

        let source = save_token_with_attempt(
            &host,
            "fixture-token",
            &config,
            true,
            |_, _| Ok(()),
            |_| Ok(None),
        )
        .unwrap();

        assert!(matches!(
            source,
            crate::model::CredentialSource::InsecureFile
        ));
        let raw = fs::read_to_string(&config.paths.credentials_file).unwrap();
        let file: CredentialFile = toml::from_str(&raw).unwrap();
        assert_eq!(file.hosts.get(&host.web_host).unwrap(), "fixture-token");
    }

    #[test]
    fn save_token_errors_when_secure_storage_returns_different_value() {
        let temp = TempDir::new().unwrap();
        let config = fixture_config(&temp);
        let host = fixture_host();

        let err = save_token_with_attempt(
            &host,
            "fixture-token",
            &config,
            true,
            |_, _| Ok(()),
            |_| Ok(Some("different-token".to_string())),
        )
        .unwrap_err();

        assert_eq!(err.code, "E_AUTH_STORAGE");
        assert!(err.message.contains("failed to verify saved token"));
        assert!(!config.paths.credentials_file.exists());
    }

    #[test]
    fn write_insecure_file_rejects_malformed_existing_credentials() {
        let temp = TempDir::new().unwrap();
        let config = fixture_config(&temp);
        let host = fixture_host();
        fs::write(&config.paths.credentials_file, "[hosts\nbroken").unwrap();

        let err = write_insecure_file(&host, "fixture-token", &config).unwrap_err();

        assert_eq!(err.code, "E_AUTH_STORAGE");
        assert!(
            err.message
                .contains("failed to parse insecure credential file")
        );
    }

    #[test]
    fn delete_insecure_file_rejects_malformed_existing_credentials() {
        let temp = TempDir::new().unwrap();
        let config = fixture_config(&temp);
        let host = fixture_host();
        fs::write(&config.paths.credentials_file, "[hosts\nbroken").unwrap();

        let err = delete_insecure_file(&host, &config).unwrap_err();

        assert_eq!(err.code, "E_AUTH_STORAGE");
        assert!(
            err.message
                .contains("failed to parse insecure credential file")
        );
    }

    #[test]
    fn load_insecure_file_defaults_when_credentials_file_is_missing() {
        let temp = TempDir::new().unwrap();
        let config = fixture_config(&temp);

        let file = load_insecure_file(&config).unwrap();

        assert!(file.hosts.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn write_insecure_file_restricts_permissions() {
        let temp = TempDir::new().unwrap();
        let config = fixture_config(&temp);
        let host = fixture_host();

        write_insecure_file(&host, "fixture-token", &config).unwrap();

        assert_eq!(fixture_file_mode(&config.paths.credentials_file), 0o600);
    }

    #[cfg(unix)]
    fn fixture_file_mode(path: &std::path::Path) -> u32 {
        fs::metadata(path).unwrap().permissions().mode() & 0o777
    }
}
