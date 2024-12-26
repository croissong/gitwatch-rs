use std::{
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context, Result};
use git2::{Oid, Repository, Status, StatusOptions};
use indoc::formatdoc;
use log::{debug, info, trace, warn};
use paris::formatter::colorize_string;
use regex::Regex;

use crate::commit_message::generate_commit_message;

pub struct GitwatchRepo {
    commit_message: Option<String>,
    commit_message_script: Option<PathBuf>,
    dry_run: bool,
    ignore_regex: Option<Regex>,
    remote: Option<String>,
    git_repo: Repository,
    repo_path: PathBuf,
}

impl GitwatchRepo {
    pub fn new(
        repo_path: &Path,
        commit_message: Option<String>,
        commit_message_script: Option<PathBuf>,
        ignore_regex: Option<Regex>,
        dry_run: bool,
        remote: Option<String>,
    ) -> Result<Self> {
        debug!("Using git repository '{}'", repo_path.display());
        let repo = Repository::open(repo_path)?;
        let gitwatch_repo = Self {
            git_repo: repo,
            repo_path: repo_path.to_path_buf(),
            commit_message,
            commit_message_script,
            dry_run,
            ignore_regex,
            remote,
        };
        gitwatch_repo.validate_commit_message_script()?;
        gitwatch_repo.validate_remote()?;

        gitwatch_repo.log_status().context(format!(
            "Failed to open git repository at path {}",
            repo_path.display()
        ))?;
        Ok(gitwatch_repo)
    }

    pub fn process_changes(&self) -> Result<()> {
        let has_staged_changes = self.stage_changes().context("Failed to stage changes")?;
        if !has_staged_changes {
            debug!("Working tree clean");
            return Ok(());
        }

        if self.dry_run {
            self.log_pending_commit()?;
        } else {
            self.commit_and_push()?;
        }
        Ok(())
    }

    // Returns true if the index contains any staged changes
    fn stage_changes(&self) -> Result<bool> {
        let mut index = self.git_repo.index()?;
        index.add_all(
            ["*"].iter(),
            git2::IndexAddOption::DEFAULT,
            Some(&mut |path, _matched_spec| {
                if self.is_path_ignored(path) {
                    1
                } else {
                    0
                }
            }),
        )?;
        index.write()?;
        let has_staged_changes = self.has_staged_changes()?;
        Ok(has_staged_changes)
    }

    fn has_staged_changes(&self) -> Result<bool> {
        let statuses = self.git_repo.statuses(None)?;
        Ok(statuses.iter().any(|entry| {
            let status = entry.status();
            status.is_index_new()
                || status.is_index_modified()
                || status.is_index_deleted()
                || status.is_index_renamed()
                || status.is_index_typechange()
        }))
    }

    fn log_pending_commit(&self) -> Result<()> {
        let commit_message = self.generate_commit_message()?;
        let staged_files = self.get_staged_file_paths()?;

        let log_message = colorize_string(formatdoc! {"
            <u>Commit message:</u>
            {}
            <u>Staged files:</u>
            {}
          ", commit_message, staged_files.join("\n")
        });
        info!("{}", log_message);
        warn!("Changes will not be commited (dry-run enabled)!");
        Ok(())
    }

    fn commit_and_push(&self) -> Result<()> {
        let index = self.git_repo.index()?;
        if index.is_empty() {
            debug!("Index is empty");
            return Ok(());
        }

        let commit_message = self
            .generate_commit_message()
            .context("Failed to generate commit message")?;
        let commit_id = self
            .create_git_commit(&commit_message)
            .context("Creating git commit failed")?;
        let commit_short_hash = &commit_id.to_string()[..7];
        info!(
            "Created commit: {} '{}'",
            commit_short_hash,
            commit_message.lines().next().unwrap()
        );

        if let Some(remote) = &self.remote {
            self.push_changes(remote)
                .context(format!("Failed to push to remote '{remote}'"))?;
        }

        Ok(())
    }

    fn is_path_ignored(&self, path: &Path) -> bool {
        if let Some(regex) = &self.ignore_regex {
            return regex.is_match(&path.to_string_lossy());
        }
        false
    }

    fn generate_commit_message(&self) -> Result<String> {
        if let Some(message) = &self.commit_message {
            Ok(message.clone())
        } else {
            // can unwrap safely, because it has been validated that either message or message script is set
            let script_path = self.commit_message_script.as_ref().unwrap();
            let commit_message = generate_commit_message(script_path, &self.repo_path)?;
            Ok(commit_message)
        }
    }

    fn get_staged_file_paths(&self) -> Result<Vec<String>> {
        let statuses = self.get_statuses()?;
        let mut staged_file_paths = Vec::new();
        for entry in statuses.iter() {
            if entry.status().is_index_new() || entry.status().is_index_modified() {
                if let Some(path) = entry.path() {
                    staged_file_paths.push(path.to_string());
                }
            }
        }
        Ok(staged_file_paths)
    }

    fn create_git_commit(&self, commit_message: &str) -> Result<Oid> {
        let mut index = self.git_repo.index()?;
        let tree_oid = index.write_tree()?;
        let tree = self.git_repo.find_tree(tree_oid)?;

        let signature = self.git_repo.signature()?;
        let parent_commit = self
            .git_repo
            .head()?
            .peel_to_commit()
            .context("Head commit not found")?;

        let oid = self.git_repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            commit_message,
            &tree,
            &[&parent_commit],
        )?;
        Ok(oid)
    }

    fn get_statuses(&self) -> Result<git2::Statuses> {
        let mut options = StatusOptions::new();
        options.include_ignored(false);
        options.include_untracked(true);
        options.recurse_untracked_dirs(true);
        self.git_repo
            .statuses(Some(&mut options))
            .context("Failed to read git status")
    }

    fn log_status(&self) -> Result<()> {
        if let Ok(head) = self.git_repo.head() {
            if let Ok(commit) = head.peel_to_commit() {
                let commit_short_hash = &commit.id().to_string()[..7];
                info!(
                    "Opened repository at: {} ({})",
                    commit_short_hash,
                    commit.summary().unwrap_or("No commit message")
                );
            }
        }

        let statuses = self.get_statuses()?;
        let is_dirty = statuses.iter().any(|s| s.status() != Status::CURRENT);

        if is_dirty {
            debug!("Repository has uncommitted changes)");
            debug!("Modified files:");
            for entry in statuses.iter() {
                if entry.status() != Status::CURRENT {
                    debug!("  {}", entry.path().unwrap_or("Unknown path"));
                }
            }
        }

        Ok(())
    }

    fn push_changes(&self, remote_name: &str) -> Result<()> {
        debug!("Pushing to remote {}", remote_name);
        let mut remote = self.git_repo.find_remote(remote_name)?;

        const BRANCH_ERROR: &str = "Failed to get current branch name";
        let branch_name = self
            .git_repo
            .head()
            .with_context(|| BRANCH_ERROR)?
            .shorthand()
            .ok_or_else(|| anyhow!(BRANCH_ERROR))?
            .to_string();

        let refspec = format!("HEAD:refs/heads/{}", branch_name);
        trace!("Pushing refspec: {}", refspec);

        // push current branch
        remote.push(&[&refspec], None)?;
        info!("Pushed changes to {}", remote_name);
        Ok(())
    }

    fn validate_commit_message_script(&self) -> Result<()> {
        if let Some(script_path) = &self.commit_message_script {
            if !script_path.exists() {
                bail!("Commit message script not found: {}", script_path.display());
            }
        }
        Ok(())
    }

    fn validate_remote(&self) -> Result<()> {
        if let Some(remote_name) = &self.remote {
            if self.git_repo.find_remote(remote_name).is_err() {
                bail!("Remote '{}' not found in repository", remote_name);
            }
        }
        Ok(())
    }
}

impl Display for GitwatchRepo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repo_path.display(),)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, sync::Once};

    use tempfile::TempDir;
    use testresult::TestResult;

    use crate::{cli::LogLevel, logger::setup_logger};

    use super::*;

    static INIT: Once = Once::new();

    fn init_test_repo() -> Result<TempDir> {
        INIT.call_once(|| {
            setup_logger(LogLevel::Debug).unwrap();
        });
        let temp_dir = tempfile::tempdir()?;
        let repo = Repository::init(&temp_dir)?;
        let remote_dir = tempfile::tempdir()?;
        let remote_path = remote_dir.into_path();
        let _ = Repository::init_bare(&remote_path)?;
        repo.remote("origin", &remote_path.to_string_lossy())?;
        Ok(temp_dir)
    }

    #[test]
    fn test_empty_repo() -> TestResult {
        let temp_dir = tempfile::tempdir()?;
        let result = GitwatchRepo::new(
            temp_dir.path(),
            Some("test".to_string()),
            None,
            None,
            false,
            None,
        );
        assert!(result.is_err());
        let err_str = result.err().unwrap().to_string();
        assert!(
            err_str.contains("could not find repository"),
            "Expected error about missing repository, got: {}",
            err_str
        );
        Ok(())
    }

    #[test]
    fn test_invalid_commit_message_script() -> TestResult {
        let temp_dir = init_test_repo()?;

        let result = GitwatchRepo::new(
            temp_dir.path(),
            None,
            Some(PathBuf::from("/nonexistent/script")),
            None,
            false,
            None,
        );
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("Commit message script not found"));
        Ok(())
    }

    #[test]
    fn test_invalid_remote() -> TestResult {
        let temp_dir = init_test_repo()?;

        let result = GitwatchRepo::new(
            temp_dir.path(),
            Some("test".to_string()),
            None,
            None,
            false,
            Some("nonexistent-remote".to_string()),
        );
        assert!(result.is_err());
        let err_str = result.err().unwrap().to_string();
        assert!(
            err_str.contains("Remote 'nonexistent-remote' not found"),
            "Expected error about missing remote, got: {}",
            err_str
        );
        Ok(())
    }

    #[test]
    fn test_commit_with_empty_index() -> TestResult {
        let temp_dir = init_test_repo()?;

        let repo = GitwatchRepo::new(
            temp_dir.path(),
            Some("test".to_string()),
            None,
            None,
            false,
            None,
        )?;

        // commit with empty index
        repo.commit_and_push()?;

        // verify no commit was created
        let head = repo.git_repo.head();
        assert!(
            head.is_err(),
            "Head should not exist as no commit should have been created"
        );

        Ok(())
    }

    #[test]
    fn test_commit_and_push_invalid_branch() -> TestResult {
        let temp_dir = init_test_repo()?;
        let repo = GitwatchRepo::new(
            temp_dir.path(),
            Some("test".to_string()),
            None,
            None,
            false,
            Some("origin".to_string()),
        )?;

        // test error getting branch name by deleting the HEAD file
        std::fs::remove_file(temp_dir.path().join(".git/HEAD")).unwrap();
        let result = repo.push_changes("origin");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Failed to get current branch name"),
            "Unexpected error message: {}",
            err
        );

        Ok(())
    }

    #[test]
    fn test_ignore_regex() -> Result<()> {
        let temp_dir = init_test_repo()?;
        let repo = GitwatchRepo::new(
            temp_dir.path(),
            Some("test".to_string()),
            None,
            Some(Regex::new(".*foo.txt.*")?),
            false,
            None,
        )?;

        fs::write(temp_dir.path().join("foo.txt"), "test content")?;
        let has_staged_changes = repo.stage_changes()?;
        assert!(
            !has_staged_changes,
            "Index should be empty when file is ignored"
        );
        Ok(())
    }
}
