use crate::{app_config::AppConfig, filter::PathFilter};
use std::{path::PathBuf, sync::mpsc::Receiver};

use anyhow::{Context, Result};
use log::{debug, warn};

use crate::{repo::GitwatchRepo, watcher::FileWatcher};

pub struct App {
    commit_on_start: bool,
    path_filter: PathFilter,
    repo: GitwatchRepo,
    repo_path: PathBuf,
    watch: bool,
    watcher: FileWatcher,
}

impl App {
    pub fn new(config: AppConfig) -> Result<Self> {
        let repo_path = &config.repository;
        let repo = GitwatchRepo::new(
            repo_path,
            config.commit_message,
            config.commit_message_script,
            config.ignore_regex.clone(),
            config.dry_run,
            config.remote,
        )?;
        let watcher = FileWatcher::new(config.debounce_seconds, config.retries);
        let path_filter = PathFilter::new(repo_path, config.ignore_regex)?;

        Ok(Self {
            commit_on_start: config.commit_on_start,
            path_filter,
            repo,
            repo_path: config.repository,
            watch: config.watch,
            watcher,
        })
    }

    pub fn run(&self, shutdown_rx: Option<Receiver<()>>) -> Result<()> {
        if self.commit_on_start {
            self.repo.process_changes().context(format!(
                "Failed to create initial commit in repo '{}'",
                self.repo
            ))?;
        }

        if !self.watch {
            warn!("Watch is disabled");
            return Ok(());
        }

        self.watcher.watch(
            &self.repo_path,
            |paths| {
                self.log_changed_paths(paths);
                self.repo.process_changes()
            },
            |path| self.path_filter.is_path_ignored(path),
            shutdown_rx,
        )
    }

    fn log_changed_paths(&self, paths: &[PathBuf]) {
        let formatted_paths = paths
            .iter()
            .map(|p| p.strip_prefix(&self.repo_path).unwrap_or(p))
            .map(|p| format!("  {}", p.display()))
            .collect::<Vec<_>>()
            .join("\n");

        debug!("Detected changes:\n{}", formatted_paths);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use git2::Repository;

    use super::*;

    #[test]
    fn test_watch_false() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let repository = temp_dir.path().to_path_buf();
        let git_repo = Repository::init(&repository)?;

        // create unstaged change
        fs::write(repository.join("foo.txt"), "bar")?;

        let config = AppConfig {
            repository,
            commit_message: Some("test message".to_string()),
            commit_on_start: false,
            watch: false,
            ..AppConfig::default()
        };
        let app = App::new(config)?;
        app.run(None)?;

        assert!(!git_repo.statuses(None)?.is_empty());
        Ok(())
    }

    #[test]
    fn test_commit_on_start() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let repository = temp_dir.path().to_path_buf();
        let git_repo = Repository::init(&repository)?;

        // create unstaged change
        fs::write(repository.join("foo.txt"), "bar")?;

        let config = AppConfig {
            repository,
            commit_message: Some("test message".to_string()),
            commit_on_start: true,
            watch: false,
            ..AppConfig::default()
        };
        let app = App::new(config)?;
        let result = app.run(None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(
            format!(
                "Failed to create initial commit in repo '{}'",
                temp_dir.path().display()
            )
            .as_str()
        ));

        assert!(!git_repo.statuses(None)?.is_empty());
        Ok(())
    }
}
