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

use super::{IGNORED_FILE_NAME, TEST_COMMIT_MESSAGE, TEST_COMMIT_MESSAGE_SCRIPT, TEST_FILE_NAME};

pub struct TestRepo {
    pub dir: TempDir,
    pub remote: Repository,
    pub repo: Repository,
}

impl TestRepo {
    pub fn new() -> Result<Self> {
        let dir = tempfile::tempdir()?;
        let repo = Repository::init(dir.path())?;
        let remote = Self::create_remote(&repo)?;

        // keep references to temp dir to avoid auto cleanup
        let test_repo = TestRepo { dir, repo, remote };
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

    pub fn wait_for_commit(&self, expected_message: &str, timeout_secs: u64) -> Result<bool> {
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        while start.elapsed() < timeout {
            if self.verify_commit(expected_message)? {
                return Ok(true);
            }
            thread::sleep(Duration::from_millis(100));
        }

        Ok(false)
    }

    pub fn wait_for_commit_count(
        &self,
        message: &str,
        expected_count: usize,
        timeout_secs: u64,
    ) -> Result<bool> {
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        while start.elapsed() < timeout {
            let count = self.count_commits_with_message(message)?;
            if count == expected_count {
                return Ok(true);
            }
            thread::sleep(Duration::from_millis(100));
        }

        let final_count = self.count_commits_with_message(message)?;
        println!(
            "Timeout reached. Expected {} commits, found {}",
            expected_count, final_count
        );
        Ok(false)
    }

    pub fn count_commits_with_message(&self, message: &str) -> Result<usize> {
        let head = self.repo.head()?.peel_to_commit()?;

        let count = head
            .clone()
            .parents()
            .chain(std::iter::once(head))
            .filter(|commit| commit.message().unwrap_or("").contains(message))
            .count();
        Ok(count)
    }

    pub fn verify_commit(&self, expected_message: &str) -> Result<bool> {
        let head = self.repo.head()?.peel_to_commit()?;

        // check commit message
        if !head.message().unwrap_or("").contains(expected_message) {
            return Ok(false);
        }

        // check that commit has changes
        let parent = head.parent(0).ok();
        let parent_tree = parent.as_ref().and_then(|c| c.tree().ok());

        let diff = self
            .repo
            .diff_tree_to_tree(parent_tree.as_ref(), Some(&head.tree()?), None)?;

        Ok(diff.deltas().len() > 0)
    }

    fn create_initial_commit(&self) -> Result<()> {
        let mut index = self.repo.index()?;

        self.write_file(TEST_FILE_NAME, "initial content")?;
        index.add_path(Path::new(TEST_FILE_NAME))?;

        index.add_path(self.create_gitignore()?)?;
        index.write()?;

        let sig = Signature::now("Test User", "test@example.com")?;
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

    fn create_remote(repo: &Repository) -> Result<Repository> {
        let remote_dir = tempfile::tempdir()?;
        let remote_path = remote_dir.into_path();
        let remote_repo = Repository::init_bare(&remote_path)?;
        repo.remote("origin", &remote_path.to_string_lossy())?;
        Ok(remote_repo)
    }

    pub fn create_test_script(&self) -> Result<std::path::PathBuf> {
        let script_path = self.dir.path().join(TEST_COMMIT_MESSAGE_SCRIPT);
        let content = "echo 'generated commit message'";
        OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o755)
            .open(&script_path)?
            .write_all(format!("#!/bin/sh\n{content}").as_bytes())?;
        Ok(script_path)
    }
}
