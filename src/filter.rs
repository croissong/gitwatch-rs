use std::path::{absolute, Path};

use anyhow::Result;
use ignore::{
    gitignore::{Gitignore, GitignoreBuilder},
    Match,
};
use log::debug;
use regex::Regex;

pub struct PathFilter {
    ignore_regex: Option<Regex>,
    gitignore: Gitignore,
    repo_path: std::path::PathBuf,
}

impl PathFilter {
    pub fn new(repo_path: &Path, ignore_regex: Option<Regex>) -> Result<Self> {
        Ok(Self {
            ignore_regex,
            gitignore: build_gitignore(repo_path)?,
            repo_path: repo_path.to_path_buf(),
        })
    }

    pub fn is_path_ignored(&self, path: &Path) -> bool {
        let normalized_path = match absolute(path) {
            Ok(path) => path,
            Err(_) => return false,
        };
        // path should always be repository-relative
        let relative_path = match normalized_path.strip_prefix(&self.repo_path) {
            Ok(path) => path,
            Err(_) => return false,
        };

        if relative_path.starts_with(".git") {
            return true;
        }

        if let Match::Ignore(_) = self
            .gitignore
            .matched_path_or_any_parents(relative_path, relative_path.is_dir())
        {
            debug!("Path {} ignored via .gitignore", path.display());
            return true;
        }

        if let Some(regex) = &self.ignore_regex {
            if regex.is_match(&relative_path.to_string_lossy()) {
                debug!(
                    "Path {} ignored via --ignore-regex",
                    relative_path.display()
                );
                return true;
            }
        }
        false
    }
}

fn build_gitignore(repo_path: &Path) -> Result<Gitignore> {
    let mut builder = GitignoreBuilder::new(repo_path);
    let gitignore_path = repo_path.join(".gitignore");
    if gitignore_path.exists() {
        log::trace!("Using gitignore {}", gitignore_path.display());
        builder.add(gitignore_path);
    }

    let gitignore = builder.build()?;
    Ok(gitignore)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_gitignore() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // create .gitignore file
        fs::write(repo_path.join(".gitignore"), "*.ignored\nignored_dir/")?;

        let path_filter = PathFilter::new(repo_path, None)?;

        // test ignored files
        assert!(path_filter.is_path_ignored(&repo_path.join(".git/config")));
        assert!(path_filter.is_path_ignored(&repo_path.join("test.ignored")));
        assert!(path_filter.is_path_ignored(&repo_path.join("ignored_dir/file.txt")));

        // test non-ignored files
        assert!(!path_filter.is_path_ignored(&repo_path.join("test.txt")));
        assert!(!path_filter.is_path_ignored(&repo_path.join("allowed_dir/file.txt")));

        Ok(())
    }

    #[test]
    fn test_ignore_regex() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        let ignore_regex = Some(Regex::new(".*\\.temp$")?);
        let path_filter = PathFilter::new(repo_path, ignore_regex)?;

        // test ignored files
        assert!(path_filter.is_path_ignored(&repo_path.join("test.temp")));
        assert!(path_filter.is_path_ignored(&repo_path.join("subdir/another.temp")));

        // test non-ignored files
        assert!(!path_filter.is_path_ignored(&repo_path.join("test.txt")));
        assert!(!path_filter.is_path_ignored(&repo_path.join("temp.txt")));

        Ok(())
    }

    #[test]
    fn test_path_absolute_error() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();
        let path_filter = PathFilter::new(repo_path, None)?;

        // create an invalid path containing a null byte which will fail absolute()
        let invalid_path = Path::new("\0invalid");

        assert!(!path_filter.is_path_ignored(invalid_path));
        Ok(())
    }

    #[test]
    fn test_strip_prefix_error() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();
        let path_filter = PathFilter::new(repo_path, None)?;

        // create a path outside the repo directory that will fail strip_prefix()
        let outside_path = temp_dir.path().parent().unwrap().join("outside.txt");

        assert!(!path_filter.is_path_ignored(&outside_path));
        Ok(())
    }
}
