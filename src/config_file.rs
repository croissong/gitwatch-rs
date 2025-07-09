use std::path::{Path, PathBuf};

use anyhow::Result;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};

use log::debug;
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct ConfigFile {
    pub commit_message: Option<String>,
    pub commit_message_script: Option<PathBuf>,
    pub commit_on_start: Option<bool>,
    pub debounce_seconds: Option<u64>,
    pub dry_run: Option<bool>,
    #[serde(default, with = "serde_regex")]
    pub ignore_regex: Option<Regex>,
    pub remote: Option<String>,
    pub retries: Option<i32>,
    pub watch: Option<bool>,
}

impl ConfigFile {
    pub fn load(repo_path: &Path) -> Result<Self> {
        let config_path = repo_path.join("gitwatch.yaml");
        if config_path.exists() {
            debug!("Using config file '{}'", config_path.display());
            Ok(Figment::new()
                .merge(Yaml::file(config_path))
                .merge(Env::prefixed("GITWATCH_"))
                .extract()?)
        } else {
            Ok(ConfigFile::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};
    use tempfile::TempDir;
    use testresult::TestResult;

    impl PartialEq for ConfigFile {
        fn eq(&self, other: &Self) -> bool {
            self.ignore_regex.as_ref().map(|r| r.as_str())
                == other.ignore_regex.as_ref().map(|r| r.as_str())
                && self.commit_message == other.commit_message
                && self.commit_message_script == other.commit_message_script
                && self.debounce_seconds == other.debounce_seconds
                && self.dry_run == other.dry_run
                && self.retries == other.retries
                && self.commit_on_start == other.commit_on_start
                && self.watch == other.watch
        }
    }

    fn create_config_file(dir: &TempDir, content: &str) -> Result<()> {
        fs::write(dir.path().join("gitwatch.yaml"), content)?;
        Ok(())
    }

    #[test]
    fn test_load_config_file() -> TestResult {
        let temp_dir = TempDir::new()?;

        let config_content = r#"
        commit_message: "test commit"
        commit_message_script: "script.sh"
        commit_on_start: true
        debounce_seconds: 5
        dry_run: true
        ignore_regex: "test.*"
        remote: "origin"
        retries: 3
        watch: true
        "#;

        create_config_file(&temp_dir, config_content)?;

        let config = ConfigFile::load(temp_dir.path())?;

        assert_eq!(config.commit_message, Some("test commit".to_string()));
        assert_eq!(
            config.commit_message_script,
            Some(PathBuf::from("script.sh"))
        );
        assert_eq!(config.commit_on_start, Some(true));
        assert_eq!(config.debounce_seconds, Some(5));
        assert_eq!(config.dry_run, Some(true));
        assert_eq!(config.ignore_regex.as_ref().unwrap().as_str(), "test.*");
        assert_eq!(config.remote, Some("origin".to_string()));
        assert_eq!(config.retries, Some(3));
        assert_eq!(config.watch, Some(true));

        Ok(())
    }

    #[test]
    fn test_load_empty_config() -> TestResult {
        let temp_dir = TempDir::new()?;
        let config = ConfigFile::load(temp_dir.path())?;
        assert_eq!(config, ConfigFile::default());
        Ok(())
    }

    #[test]
    fn test_load_invalid_config() -> TestResult {
        let temp_dir = TempDir::new()?;

        let invalid_content = r#"
        commit_message: 42  # should be string
        "#;

        create_config_file(&temp_dir, invalid_content)?;

        let result = ConfigFile::load(temp_dir.path());
        assert!(result.is_err());
        let err = format!("{:#}", result.unwrap_err());
        assert!(
            err.contains("invalid type"),
            "Unexpected error message: {err}"
        );

        Ok(())
    }
}
