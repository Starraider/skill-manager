use crate::error::AppError;
use crate::models::config::AppConfig;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Expands a leading `~` or `~/` to the user's home directory.
pub fn expand_tilde(path: &Path) -> PathBuf {
    if let Ok(stripped) = path.strip_prefix("~") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    let path_str = path.to_string_lossy();
    if path_str == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    } else if let Some(stripped) = path_str.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    } else if let Some(stripped) = path_str.strip_prefix("~\\") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    path.to_path_buf()
}

/// Returns the default path to `config.json`.
pub fn default_config_path() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("skill-manager").join("config.json")
    } else if let Some(home) = dirs::home_dir() {
        home.join(".config")
            .join("skill-manager")
            .join("config.json")
    } else {
        PathBuf::from(".config/skill-manager/config.json")
    }
}

/// Loads application configuration from the standard config file path,
/// returning the default seeded config if the file is missing or corrupted.
pub fn load_config() -> Result<AppConfig, AppError> {
    load_config_from_path(&default_config_path())
}

/// Loads configuration from a specific file path.
pub fn load_config_from_path(path: &Path) -> Result<AppConfig, AppError> {
    if !path.exists() {
        return Ok(AppConfig::default_seeded());
    }

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return Err(AppError::Io(e)),
    };

    match serde_json::from_str::<AppConfig>(&content) {
        Ok(config) => Ok(config),
        Err(_) => {
            // Corrupt config recovers gracefully to default seeded
            Ok(AppConfig::default_seeded())
        }
    }
}

/// Saves application configuration atomically to the standard config file path.
pub fn save_config(config: &AppConfig) -> Result<(), AppError> {
    save_config_to_path(config, &default_config_path())
}

/// Saves application configuration atomically to a specific file path.
pub fn save_config_to_path(config: &AppConfig, path: &Path) -> Result<(), AppError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    if !parent.exists() {
        std::fs::create_dir_all(parent)?;
    }

    let json_bytes = serde_json::to_string_pretty(config)?;

    let mut temp_file = tempfile::NamedTempFile::new_in(parent)?;
    temp_file.write_all(json_bytes.as_bytes())?;
    temp_file.flush()?;

    temp_file.persist(path).map_err(|e| AppError::Io(e.error))?;
    Ok(())
}
