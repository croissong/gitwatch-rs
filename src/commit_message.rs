use anyhow::{bail, Context, Result};
use log::debug;
use std::path::Path;
use std::process::Command;

pub fn generate_commit_message(script_path: &Path, repo_path: &Path) -> Result<String> {
    let file_name = script_path
        .file_name()
        .context("Failed to get script name")?
        .to_string_lossy();
    debug!("Executing commit message script {}", file_name);

    let output = Command::new(script_path)
        .current_dir(repo_path)
        .output()
        .with_context(|| {
            format!(
                "Failed to execute commit message script '{}'",
                script_path.display()
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!(
            "Commit message script '{}' failed with exit code {}.\nError: {}",
            script_path.display(),
            output.status,
            stderr
        );
    }

    let commit_message = String::from_utf8(output.stdout)
        .context("Commit message script output is not valid UTF-8")?;

    let trimmed_message = commit_message.trim();
    if trimmed_message.is_empty() {
        bail!("Commit message script output is empty");
    }

    if let Some(first_line) = trimmed_message.lines().next() {
        debug!("Generated commit message: '{first_line}'");
    }

    Ok(commit_message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;
    use testresult::TestResult;

    #[test]
    #[cfg(unix)]
    fn test_successful_script_execution() -> TestResult {
        let temp_dir = TempDir::new()?;
        let script_content = "echo 'Test commit message'";
        let script_path = create_test_script(&temp_dir, script_content)?;

        let result = generate_commit_message(&script_path, temp_dir.path())?;
        assert_eq!(result.trim(), "Test commit message");
        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn test_working_directory() -> TestResult {
        let temp_dir = TempDir::new()?;
        let script_content = "echo $PWD";
        let script_path = create_test_script(&temp_dir, script_content)?;

        let result = generate_commit_message(&script_path, temp_dir.path())?;
        assert_eq!(
            result.trim(),
            temp_dir.path().canonicalize()?.display().to_string()
        );
        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn test_script_execution_failure() -> TestResult {
        let temp_dir = TempDir::new()?;
        let script_content = "exit 1";
        let script_path = create_test_script(&temp_dir, script_content)?;

        let result = generate_commit_message(&script_path, temp_dir.path());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("failed with exit code"));
        Ok(())
    }

    #[test]
    fn test_nonexistent_script() {
        let result = generate_commit_message(Path::new("/nonexistent/script/path"), Path::new(""));
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to execute commit message script"));
    }

    #[test]
    #[cfg(unix)]
    fn test_empty_output() -> TestResult {
        let temp_dir = TempDir::new()?;
        let script_content = "echo ''";
        let script_path = create_test_script(&temp_dir, script_content)?;

        let result = generate_commit_message(&script_path, temp_dir.path());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Commit message script output is empty"));
        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn test_whitespace_only_output() -> TestResult {
        let temp_dir = TempDir::new()?;
        let script_content = "echo '   \n  \t  '";
        let script_path = create_test_script(&temp_dir, script_content)?;

        let result = generate_commit_message(&script_path, temp_dir.path());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Commit message script output is empty"));
        Ok(())
    }

    fn create_test_script(dir: &TempDir, content: &str) -> Result<std::path::PathBuf> {
        let script_path = dir.path().join("test_script.sh");
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o755)
            .open(&script_path)?
            .write_all(format!("#!/bin/sh\n{content}").as_bytes())?;
        thread::sleep(Duration::from_millis(50));
        Ok(script_path)
    }
}
