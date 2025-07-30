use std::{
    fs::{self, OpenOptions},
    os::unix::fs::OpenOptionsExt,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use std::io::Write;

use anyhow::Result;
use git2::{Repository, Signature};

use gitwatch_rs::app_config::AppConfig;
use tempfile::TempDir;

use super::{IGNORED_FILE_NAME, TEST_COMMIT_MESSAGE, TEST_FILE_NAME};

pub struct TestRepo {
    pub dir: TempDir,
    pub _remote_dir: TempDir,
    pub remote: Repository,
    pub repo: Repository,
}

impl TestRepo {
    pub fn new() -> Result<Self> {
        let dir = tempfile::tempdir()?;
        let remote_dir = tempfile::tempdir()?;
        let repo = Repository::init(dir.path())?;
        let remote = Self::create_remote(&repo, &remote_dir)?;

        // keep references to temp dir to avoid auto cleanup
        let test_repo = TestRepo {
            dir,
            _remote_dir: remote_dir,
            repo,
            remote,
        };
        test_repo.setup_git_config()?;
        test_repo.create_initial_commit()?;
        Ok(test_repo)
    }

    pub fn default_app_config(&self) -> AppConfig {
        AppConfig {
            repository: self.dir.path().to_path_buf(),
            commit_message: Some(TEST_COMMIT_MESSAGE.to_string()),
            commit_message_script: None,
            debounce_seconds: 0,
            ignore_regex: None,
            dry_run: false,
            retries: 0,
            commit_on_start: true,
            watch: false,
            remote: None,
        }
    }

    pub fn write_file(&self, path: &str, content: &str) -> Result<()> {
        let file_path = self.dir.path().join(path);
        fs::write(file_path, content)?;
        Ok(())
    }

    pub fn delete_file(&self, path: &str) -> Result<()> {
        let file_path = self.dir.path().join(path);
        fs::remove_file(file_path)?;
        Ok(())
    }

    pub fn create_test_script(&self) -> Result<std::path::PathBuf> {
        let script_path = self.dir.path().join("commit-message-script.sh");
        let content = "echo 'generated commit message'";
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o755)
            .open(&script_path)?
            .write_all(format!("#!/bin/sh\n{content}").as_bytes())?;
        Ok(script_path)
    }

    const VERIFY_TIMEOUT: Duration = Duration::from_secs(5);
    const VERIFY_INTERVAL: Duration = Duration::from_millis(100);

    pub fn verify_commits(&self, expected_message: &str, expected_count: usize) -> Result<()> {
        let start = Instant::now();
        let mut last_commits = Vec::new();

        while start.elapsed() < Self::VERIFY_TIMEOUT {
            last_commits = self.get_commits_with_message(expected_message)?;

            if last_commits.len() == expected_count
                && last_commits
                    .iter()
                    .all(|commit| self.has_commit_changes(commit).unwrap_or(false))
            {
                return Ok(());
            }
            thread::sleep(Self::VERIFY_INTERVAL);
        }

        let last_count = last_commits.len();
        let empty_commits: Vec<_> = last_commits
            .iter()
            .enumerate()
            .filter(|(_, commit)| !self.has_commit_changes(commit).unwrap_or(false))
            .map(|(i, _)| i)
            .collect();

        panic!(
            "Timeout waiting for commits. Expected {expected_count} commit(s) with message '{expected_message}' and changes, found {last_count} commits (empty commits at indices: {empty_commits:?})"
        );
    }

    fn get_commits_with_message(&self, message: &str) -> Result<Vec<git2::Commit<'_>>> {
        let head = self.repo.head()?.peel_to_commit()?;

        // Collect matching commits into a Vec
        let commits: Vec<_> = head
            .clone()
            .parents()
            .chain(std::iter::once(head))
            .filter(|commit| commit.message().unwrap_or("").contains(message))
            .collect();

        Ok(commits)
    }

    fn has_commit_changes(&self, commit: &git2::Commit) -> Result<bool> {
        let parent = commit.parent(0).ok();
        let parent_tree = parent.as_ref().and_then(|c| c.tree().ok());

        let diff =
            self.repo
                .diff_tree_to_tree(parent_tree.as_ref(), Some(&commit.tree()?), None)?;

        Ok(diff.deltas().len() > 0)
    }

    fn create_initial_commit(&self) -> Result<()> {
        let mut index = self.repo.index()?;

        self.write_file(TEST_FILE_NAME, "initial content")?;
        index.add_path(Path::new(TEST_FILE_NAME))?;

        index.add_path(self.create_gitignore()?)?;
        index.write()?;

        let sig = self.repo.signature()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        self.repo
            .commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;
        Ok(())
    }

    fn setup_git_config(&self) -> Result<()> {
        let mut config = self.repo.config()?;
        config.set_str("user.name", "Test User")?;
        config.set_str("user.email", "test@example.com")?;
        Ok(())
    }

    fn create_gitignore(&self) -> Result<&Path> {
        self.write_file(".gitignore", IGNORED_FILE_NAME)?;
        Ok(Path::new(".gitignore"))
    }

    fn create_remote(repo: &Repository, remote_dir: &TempDir) -> Result<Repository> {
        let remote_path = remote_dir.path();
        let remote_repo = Repository::init_bare(remote_path)?;
        repo.remote("origin", &remote_path.to_string_lossy())?;
        Ok(remote_repo)
    }
}
