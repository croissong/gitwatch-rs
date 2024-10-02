use std::{sync::Once, thread, time::Duration};

use assert_cmd::Command;
use gitwatch_rs::{app::App, app_config::AppConfig, cli::LogLevel, logger::setup_logger};
use regex::Regex;
use support::{
    AppRunner, TestRepo, IGNORED_FILE_NAME, TEST_COMMIT_MESSAGE, TEST_COMMIT_MESSAGE_SCRIPT,
    TEST_FILE_CONTENT, TEST_FILE_NAME, TEST_REMOTE,
};
use testresult::TestResult;

mod support;

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        setup_logger(LogLevel::Debug).unwrap();
    });
}

#[test]
fn test_commit_on_start() -> TestResult {
    setup();

    let test_repo = TestRepo::new()?;

    let config = test_repo.default_app_config();
    let app = App::new(config)?;

    test_repo.write_file(TEST_FILE_NAME, TEST_FILE_CONTENT)?;

    app.run(None)?;

    assert!(
        test_repo.verify_commit(TEST_COMMIT_MESSAGE)?,
        "commit not found in git log"
    );
    Ok(())
}

#[test]
fn test_watch() -> TestResult {
    setup();

    let test_repo = TestRepo::new()?;
    let config = AppConfig {
        watch: true,
        ..test_repo.default_app_config()
    };
    let app = App::new(config)?;
    let runner = AppRunner::run(app);

    test_repo.write_file(TEST_FILE_NAME, TEST_FILE_CONTENT)?;
    assert!(test_repo.wait_for_commit(TEST_COMMIT_MESSAGE, 1)?,);

    test_repo.delete_file(TEST_FILE_NAME)?;
    assert!(test_repo.wait_for_commit_count(TEST_COMMIT_MESSAGE, 2, 2)?,);

    runner.shutdown()?;
    Ok(())
}

#[test]
fn test_commit_message_script() -> TestResult {
    setup();

    let test_repo = TestRepo::new()?;
    test_repo.create_test_script()?;
    let config = AppConfig {
        commit_message: None,
        commit_message_script: Some(test_repo.dir.path().join(TEST_COMMIT_MESSAGE_SCRIPT)),
        ..test_repo.default_app_config()
    };
    let app = App::new(config)?;

    test_repo.write_file(TEST_FILE_NAME, TEST_FILE_CONTENT)?;

    app.run(None)?;
    assert!(test_repo.verify_commit("generated commit message")?);
    Ok(())
}

#[test]
fn test_debounce() -> TestResult {
    setup();

    let test_repo = TestRepo::new()?;
    let config = AppConfig {
        watch: true,
        commit_on_start: false,
        debounce_seconds: 1,
        ..test_repo.default_app_config()
    };
    let app = App::new(config)?;
    let runner = AppRunner::run(app);

    test_repo.write_file(TEST_FILE_NAME, "first change")?;
    thread::sleep(Duration::from_millis(500));
    test_repo.write_file(TEST_FILE_NAME, "second change")?;
    let result = test_repo.wait_for_commit_count(TEST_COMMIT_MESSAGE, 1, 2)?;

    runner.shutdown()?;
    assert!(result, "Expected only one debounced commit");
    Ok(())
}

#[test]
fn test_gitignore() -> TestResult {
    setup();
    let test_repo = TestRepo::new()?;
    let config = AppConfig {
        watch: true,
        ..test_repo.default_app_config()
    };
    let app = App::new(config)?;
    test_repo.write_file(IGNORED_FILE_NAME, TEST_FILE_CONTENT)?;
    let runner = AppRunner::run(app);

    thread::sleep(Duration::from_millis(500));
    test_repo.write_file(IGNORED_FILE_NAME, TEST_FILE_CONTENT)?;
    thread::sleep(Duration::from_millis(500));
    runner.shutdown()?;

    assert_eq!(
        test_repo.count_commits_with_message(TEST_COMMIT_MESSAGE)?,
        0,
        "Expected no commit"
    );
    Ok(())
}

#[test]
fn test_ignore_regex() -> TestResult {
    setup();
    let test_repo = TestRepo::new()?;
    let config = AppConfig {
        watch: true,
        ignore_regex: Some(Regex::new("bar.txt.*")?),
        ..test_repo.default_app_config()
    };
    let app = App::new(config)?;

    test_repo.write_file("bar.txt", TEST_FILE_CONTENT)?;
    let runner = AppRunner::run(app);

    thread::sleep(Duration::from_millis(500));
    test_repo.write_file("bar.txt", TEST_FILE_CONTENT)?;
    thread::sleep(Duration::from_millis(500));
    runner.shutdown()?;

    assert_eq!(
        test_repo.count_commits_with_message(TEST_COMMIT_MESSAGE)?,
        0,
        "Expected no commit"
    );
    Ok(())
}

#[test]
fn test_dry_run() -> TestResult {
    setup();
    let test_repo = TestRepo::new()?;
    let config = AppConfig {
        dry_run: true,
        ..test_repo.default_app_config()
    };
    let app = App::new(config)?;

    test_repo.write_file(TEST_FILE_NAME, TEST_FILE_CONTENT)?;
    app.run(None)?;

    assert_eq!(
        test_repo.count_commits_with_message(TEST_COMMIT_MESSAGE)?,
        0,
        "Expected no commit"
    );
    Ok(())
}

#[test]
fn test_main_valid_args() -> TestResult {
    let test_repo = TestRepo::new()?;
    let repo_path = test_repo.dir.path();

    test_repo.write_file(TEST_FILE_NAME, TEST_FILE_CONTENT)?;

    let mut cmd = Command::cargo_bin("gitwatch")?;
    cmd.arg("watch")
        .arg(repo_path)
        .arg("--commit-message")
        .arg("test commit")
        .arg("--watch=false");

    cmd.assert().success();

    assert!(test_repo.verify_commit("test commit")?);
    Ok(())
}

#[test]
fn test_main_invalid_path() -> TestResult {
    let mut cmd = Command::cargo_bin("gitwatch")?;
    cmd.arg("/nonexistent/path")
        .arg("--commit-message")
        .arg("test commit")
        .arg("--watch=false");

    cmd.assert().failure();
    Ok(())
}

#[test]
fn test_push() -> TestResult {
    setup();

    let test_repo = TestRepo::new()?;
    let config = AppConfig {
        remote: Some(TEST_REMOTE.to_string()),
        ..test_repo.default_app_config()
    };
    let app = App::new(config)?;

    test_repo.write_file(TEST_FILE_NAME, TEST_FILE_CONTENT)?;
    app.run(None)?;

    // verify changes were pushed to remote
    let remote_head = test_repo.remote.head()?;
    let remote_commit = remote_head.peel_to_commit()?;
    assert_eq!(
        remote_commit.message().unwrap_or(""),
        TEST_COMMIT_MESSAGE,
        "Remote commit message doesn't match"
    );

    Ok(())
}

#[test]
fn test_push_invalid_remote() -> TestResult {
    setup();

    let test_repo = TestRepo::new()?;
    let config = AppConfig {
        remote: Some(TEST_REMOTE.to_string()),
        ..test_repo.default_app_config()
    };

    let app = App::new(config)?;

    test_repo.repo.remote_delete(TEST_REMOTE)?;

    test_repo.write_file(TEST_FILE_NAME, TEST_FILE_CONTENT)?;
    let result = app.run(None);

    assert!(result.is_err());
    let err = format!("{:#}", result.unwrap_err());
    assert!(
        err.contains(&format!("Failed to push to remote '{TEST_REMOTE}'")),
        "Unexpected error message: {}",
        err
    );

    Ok(())
}

#[test]
fn test_completion_command() -> TestResult {
    let mut cmd = Command::cargo_bin("gitwatch")?;
    cmd.arg("completion").arg("bash");
    cmd.assert().success();
    Ok(())
}

#[test]
fn test_invalid_command() -> TestResult {
    let mut cmd = Command::cargo_bin("gitwatch")?;
    // Pass invalid args to trigger error path
    cmd.arg("watch").arg("--invalid-flag");
    cmd.assert().failure();
    Ok(())
}
