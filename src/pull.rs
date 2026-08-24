use anyhow::{bail, Context, Result};
use auth_git2::GitAuthenticator;
use git2::{build::CheckoutBuilder, AnnotatedCommit, ErrorCode, Oid, Repository};
use log::{debug, info};

/// Fetch the current branch from the given remote and integrate it into the
/// local branch, rebasing local commits on top of the remote ones.
/// Equivalent to `git pull --rebase <remote>` in the original gitwatch.
pub fn pull_rebase(git_repo: &Repository, remote_name: &str) -> Result<()> {
    let branch = get_current_branch(git_repo)?;
    debug!("Pulling branch {branch} from remote {remote_name}");

    let mut remote = git_repo.find_remote(remote_name)?;
    GitAuthenticator::default()
        .fetch(git_repo, &mut remote, &[&branch], None)
        .context(format!("Failed to fetch branch '{branch}'"))?;

    let fetch_head = git_repo
        .find_reference("FETCH_HEAD")
        .context("FETCH_HEAD not found after fetch")?;
    let upstream = git_repo.reference_to_annotated_commit(&fetch_head)?;

    let (analysis, _) = git_repo.merge_analysis(&[&upstream])?;
    if analysis.is_up_to_date() {
        debug!("Already up to date with {remote_name}/{branch}");
        return Ok(());
    }

    if analysis.is_fast_forward() {
        return fast_forward(git_repo, &branch, &upstream);
    }

    rebase_onto(git_repo, &upstream)
}

fn fast_forward(git_repo: &Repository, branch: &str, upstream: &AnnotatedCommit) -> Result<()> {
    let refname = format!("refs/heads/{branch}");
    let target = git_repo.find_commit(upstream.id())?;
    git_repo.checkout_tree(target.as_object(), Some(&mut CheckoutBuilder::default()))?;
    git_repo
        .find_reference(&refname)?
        .set_target(upstream.id(), "gitwatch: fast-forward")?;
    info!("Fast-forwarded {branch} to {}", short_hash(&upstream.id()));
    Ok(())
}

fn rebase_onto(git_repo: &Repository, upstream: &AnnotatedCommit) -> Result<()> {
    let signature = git_repo.signature()?;
    // branch = None rebases the current HEAD onto the upstream commit
    let mut rebase = git_repo
        .rebase(None, Some(upstream), None, None)
        .context("Failed to start rebase")?;

    while let Some(operation) = rebase.next() {
        if let Err(e) = operation {
            rebase.abort().ok();
            return Err(e).context("Failed to apply commit during rebase");
        }

        let conflicts = get_conflicting_paths(git_repo)?;
        if !conflicts.is_empty() {
            rebase.abort().context("Failed to abort rebase")?;
            bail!(
                "Rebase stopped due to conflicts in: {}",
                conflicts.join(", ")
            );
        }

        match rebase.commit(None, &signature, None) {
            Ok(_) => {}
            // the changes are already part of the upstream commits, nothing to apply
            Err(e) if e.code() == ErrorCode::Applied => {
                debug!("Skipping commit that is already applied upstream");
            }
            Err(e) => {
                rebase.abort().ok();
                return Err(e).context("Failed to commit rebased changes");
            }
        }
    }

    rebase
        .finish(Some(&signature))
        .context("Failed to finish rebase")?;
    info!("Rebased local commits onto {}", short_hash(&upstream.id()));
    Ok(())
}

fn get_current_branch(git_repo: &Repository) -> Result<String> {
    let head = git_repo.head().context("Failed to read HEAD")?;
    if !head.is_branch() {
        bail!("Cannot pull with a detached HEAD");
    }
    let branch = head
        .shorthand()
        .context("Failed to read current branch name")?;
    Ok(branch.to_string())
}

fn get_conflicting_paths(git_repo: &Repository) -> Result<Vec<String>> {
    let index = git_repo.index()?;
    if !index.has_conflicts() {
        return Ok(Vec::new());
    }
    let paths = index
        .conflicts()?
        .filter_map(|conflict| conflict.ok())
        .filter_map(|conflict| conflict.our.or(conflict.their).or(conflict.ancestor))
        .map(|entry| String::from_utf8_lossy(&entry.path).to_string())
        .collect();
    Ok(paths)
}

fn short_hash(oid: &Oid) -> String {
    oid.to_string()[..7].to_string()
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use tempfile::TempDir;
    use testresult::TestResult;

    use super::*;

    struct TestSetup {
        _dir: TempDir,
        _remote_dir: TempDir,
        repo: Repository,
    }

    /// Creates a repo with one commit, pushed to a bare remote called 'origin'
    fn setup() -> Result<TestSetup> {
        let dir = tempfile::tempdir()?;
        let remote_dir = tempfile::tempdir()?;
        let repo = Repository::init(dir.path())?;
        Repository::init_bare(remote_dir.path())?;

        let mut config = repo.config()?;
        config.set_str("user.name", "Test User")?;
        config.set_str("user.email", "test@example.com")?;

        repo.remote("origin", &remote_dir.path().to_string_lossy())?;
        commit_file(&repo, "initial.txt", "initial", "initial commit")?;
        let branch = get_current_branch(&repo)?;
        repo.find_remote("origin")?
            .push(&[&format!("refs/heads/{branch}:refs/heads/{branch}")], None)?;

        Ok(TestSetup {
            _dir: dir,
            _remote_dir: remote_dir,
            repo,
        })
    }

    fn commit_file(repo: &Repository, name: &str, content: &str, message: &str) -> Result<Oid> {
        let workdir = repo.workdir().context("Repo has no workdir")?;
        fs::write(workdir.join(name), content)?;

        let mut index = repo.index()?;
        index.add_path(Path::new(name))?;
        index.write()?;
        let tree = repo.find_tree(index.write_tree()?)?;

        let signature = repo.signature()?;
        let parents = match repo.head().ok().and_then(|head| head.peel_to_commit().ok()) {
            Some(commit) => vec![commit],
            None => vec![],
        };
        let parents: Vec<_> = parents.iter().collect();
        let oid = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )?;
        Ok(oid)
    }

    /// Adds a commit to the remote, as another clone of the repo would
    fn commit_to_remote(setup: &TestSetup, name: &str, content: &str) -> Result<()> {
        let clone_dir = tempfile::tempdir()?;
        let remote_url = setup.repo.find_remote("origin")?.url().unwrap().to_string();
        let clone = Repository::clone(&remote_url, clone_dir.path())?;

        let mut config = clone.config()?;
        config.set_str("user.name", "Other User")?;
        config.set_str("user.email", "other@example.com")?;

        commit_file(&clone, name, content, "remote commit")?;
        let branch = get_current_branch(&clone)?;
        clone
            .find_remote("origin")?
            .push(&[&format!("refs/heads/{branch}:refs/heads/{branch}")], None)?;
        Ok(())
    }

    #[test]
    fn test_pull_up_to_date() -> TestResult {
        let setup = setup()?;
        let head_before = setup.repo.head()?.peel_to_commit()?.id();

        pull_rebase(&setup.repo, "origin")?;

        assert_eq!(setup.repo.head()?.peel_to_commit()?.id(), head_before);
        Ok(())
    }

    #[test]
    fn test_pull_fast_forward() -> TestResult {
        let setup = setup()?;
        commit_to_remote(&setup, "remote.txt", "remote content")?;

        pull_rebase(&setup.repo, "origin")?;

        let head = setup.repo.head()?.peel_to_commit()?;
        assert_eq!(head.message().unwrap_or(""), "remote commit");
        assert!(
            setup.repo.workdir().unwrap().join("remote.txt").exists(),
            "Fast-forwarded file should exist in the working directory"
        );
        Ok(())
    }

    #[test]
    fn test_pull_rebase_diverged() -> TestResult {
        let setup = setup()?;
        commit_to_remote(&setup, "remote.txt", "remote content")?;
        commit_file(&setup.repo, "local.txt", "local content", "local commit")?;

        pull_rebase(&setup.repo, "origin")?;

        let head = setup.repo.head()?.peel_to_commit()?;
        assert_eq!(head.message().unwrap_or(""), "local commit");
        assert_eq!(head.parent_count(), 1, "History should stay linear");
        assert_eq!(head.parent(0)?.message().unwrap_or(""), "remote commit");
        assert!(setup.repo.workdir().unwrap().join("remote.txt").exists());
        assert!(setup.repo.workdir().unwrap().join("local.txt").exists());
        assert_eq!(setup.repo.state(), git2::RepositoryState::Clean);
        Ok(())
    }

    #[test]
    fn test_pull_rebase_conflict_aborts() -> TestResult {
        let setup = setup()?;
        commit_to_remote(&setup, "conflict.txt", "remote content")?;
        let head_before =
            commit_file(&setup.repo, "conflict.txt", "local content", "local commit")?;

        let result = pull_rebase(&setup.repo, "origin");

        let err = format!("{:#}", result.unwrap_err());
        assert!(
            err.contains("conflicts in: conflict.txt"),
            "Unexpected error message: {err}"
        );
        assert_eq!(
            setup.repo.state(),
            git2::RepositoryState::Clean,
            "Aborted rebase must leave the repo in a clean state"
        );
        assert_eq!(setup.repo.head()?.peel_to_commit()?.id(), head_before);
        Ok(())
    }

    #[test]
    fn test_pull_detached_head() -> TestResult {
        let setup = setup()?;
        let head = setup.repo.head()?.peel_to_commit()?.id();
        setup.repo.set_head_detached(head)?;

        let err = pull_rebase(&setup.repo, "origin").unwrap_err().to_string();
        assert!(
            err.contains("detached HEAD"),
            "Unexpected error message: {err}"
        );
        Ok(())
    }

    #[test]
    fn test_pull_fast_forward_keeps_local_edits() -> TestResult {
        let setup = setup()?;
        commit_to_remote(&setup, "remote.txt", "remote content")?;
        let workdir = setup.repo.workdir().unwrap();
        fs::write(workdir.join("initial.txt"), "local edit")?;

        pull_rebase(&setup.repo, "origin")?;

        assert!(workdir.join("remote.txt").exists());
        assert_eq!(
            fs::read_to_string(workdir.join("initial.txt"))?,
            "local edit"
        );
        Ok(())
    }
}
