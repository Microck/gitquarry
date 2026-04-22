use crate::error::{AppError, AppResult};
use crate::model::{ColorPreference, OutputFormat, ProgressMode};
use serde::{Deserialize, Serialize};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigFile {
    pub host: Option<String>,
    pub format: Option<OutputFormat>,
    pub limit: Option<usize>,
    pub progress: Option<ProgressMode>,
    pub color: Option<ColorPreference>,
}

#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub dir: PathBuf,
    pub config_file: PathBuf,
    pub credentials_file: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ConfigBundle {
    pub paths: ConfigPaths,
    pub data: ConfigFile,
}

impl ConfigBundle {
    pub fn load() -> AppResult<Self> {
        let paths = resolve_paths()?;
        let data = if paths.config_file.exists() {
            let raw = fs::read_to_string(&paths.config_file).map_err(|err| {
                AppError::with_detail("E_CONFIG_IO", "failed to read config", err.to_string())
            })?;
            toml::from_str(&raw).map_err(|err| {
                AppError::with_detail("E_CONFIG_PARSE", "failed to parse config", err.to_string())
            })?
        } else {
            ConfigFile::default()
        };

        Ok(Self { paths, data })
    }

    pub fn ensure_parent_dirs(&self) -> AppResult<()> {
        fs::create_dir_all(&self.paths.dir).map_err(|err| {
            AppError::with_detail(
                "E_CONFIG_IO",
                "failed to create config directory",
                err.to_string(),
            )
        })?;
        set_private_dir_permissions(&self.paths.dir)
    }
}

fn resolve_paths() -> AppResult<ConfigPaths> {
    let dir = if let Ok(path) = std::env::var("GITQUARRY_CONFIG_DIR") {
        PathBuf::from(path)
    } else {
        dirs::config_dir()
            .ok_or_else(|| {
                AppError::new("E_CONFIG_PATH", "could not determine user config directory")
            })?
            .join("gitquarry")
    };

    Ok(ConfigPaths {
        config_file: dir.join("config.toml"),
        credentials_file: dir.join("credentials.toml"),
        dir,
    })
}

#[cfg(unix)]
fn set_private_dir_permissions(path: &PathBuf) -> AppResult<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|err| {
        AppError::with_detail(
            "E_CONFIG_IO",
            "failed to restrict config directory permissions",
            err.to_string(),
        )
    })
}

#[cfg(not(unix))]
fn set_private_dir_permissions(_path: &PathBuf) -> AppResult<()> {
    Ok(())
}
