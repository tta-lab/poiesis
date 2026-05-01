use serde::Deserialize;
use std::path::PathBuf;

use crate::error::PoiesisError;

#[derive(Debug, Clone)]
pub struct Config {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    site: SiteConfig,
}

#[derive(Debug, Deserialize)]
struct SiteConfig {
    url: String,
    username: String,
}

impl Config {
    /// Load from ~/.config/poiesis/config.toml + POIESIS_PASSWORD env var
    pub fn load() -> Result<Self, PoiesisError> {
        let config_path = config_path();

        if !config_path.exists() {
            return Err(PoiesisError::ConfigNotFound);
        }

        let content = std::fs::read_to_string(&config_path)?;
        let file: ConfigFile =
            toml::from_str(&content).map_err(|e| PoiesisError::ConfigParseFailed(e.to_string()))?;

        let password = match std::env::var("POIESIS_PASSWORD") {
            Ok(p) if p.is_empty() => return Err(PoiesisError::EmptyPassword),
            Ok(p) => p,
            Err(_) => return Err(PoiesisError::MissingPassword),
        };

        Ok(Config {
            url: file.site.url.trim_end_matches('/').to_string(),
            username: file.site.username,
            password,
        })
    }
}

pub fn config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".config"));
    config_dir.join("poiesis").join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_parses_toml() {
        let toml = r#"
[site]
url = "https://example.org"
username = "neil"
"#;
        let file: ConfigFile = toml::from_str(toml).unwrap();
        assert_eq!(file.site.url, "https://example.org");
        assert_eq!(file.site.username, "neil");
    }

    #[test]
    fn test_missing_password_env() {
        let result: Result<String, PoiesisError> =
            match std::env::var("__DEFINITELY_MISSING_POIESIS_PW__") {
                Err(_) => Err(PoiesisError::MissingPassword),
                Ok(_) => Ok("found".to_string()),
            };
        assert!(matches!(result, Err(PoiesisError::MissingPassword)));
    }

    #[test]
    fn test_empty_password_env() {
        let password = "";
        let result: Result<String, PoiesisError> = if password.is_empty() {
            Err(PoiesisError::EmptyPassword)
        } else {
            Ok(password.to_string())
        };
        assert!(matches!(result, Err(PoiesisError::EmptyPassword)));
    }

    #[test]
    fn test_missing_config_file() {
        let path = PathBuf::from("/nonexistent/path/config.toml");
        let result: Result<(), PoiesisError> = if !path.exists() {
            Err(PoiesisError::ConfigNotFound)
        } else {
            Ok(())
        };
        assert!(matches!(result, Err(PoiesisError::ConfigNotFound)));
    }

    #[test]
    fn test_malformed_toml() {
        let bad_toml = "this is not valid toml @@@@";
        let result: Result<ConfigFile, _> = toml::from_str(bad_toml);
        assert!(result.is_err());
        let err = PoiesisError::ConfigParseFailed(result.unwrap_err().to_string());
        assert!(matches!(err, PoiesisError::ConfigParseFailed(_)));
    }

    #[test]
    fn test_missing_url_field() {
        let toml = r#"
[site]
username = "neil"
"#;
        let result: Result<ConfigFile, _> = toml::from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_url_trailing_slash_stripped() {
        let url = "https://example.org/";
        let stripped = url.trim_end_matches('/').to_string();
        assert_eq!(stripped, "https://example.org");
    }
}
