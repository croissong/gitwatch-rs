use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use clap_complete::Shell;
use log::LevelFilter;
use regex::Regex;
use serde::Deserialize;

#[derive(Parser)]
#[command(
    name = "gitwatch",
    about = "CLI to watch a git repo and automatically commit changes"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser)]
pub enum Commands {
    /// Watch a repository and commit changes
    Watch(CliOptions),

    /// Generate shell completion scripts
    Completion {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Parser)]
pub struct CliOptions {
    /// Path to the Git repository to monitor for changes
    #[clap(default_value = ".")]
    pub repository: PathBuf,

    #[clap(flatten)]
    pub commit_message: CommitMessageOptions,

    /// Automatically commit any existing changes on start
    #[clap(long = "commit-on-start", default_value = "true")]
    pub commit_on_start: std::primitive::bool,

    /// Number of seconds to wait before processing multiple changes to the same file.
    /// Higher values reduce commit frequency but group more changes together.
    #[clap(long = "debounce-seconds", default_value = "1", verbatim_doc_comment)]
    pub debounce_seconds: u64,

    /// Run without performing actual Git operations (staging, committing, etc.)
    #[clap(long = "dry-run", default_value = "false")]
    pub dry_run: bool,

    /// Regular expression pattern for files to exclude from watching.
    /// Matching is performed against repository-relative file paths.
    /// Note: the .git folder & gitignored files are ignored by default.
    /// Example: "\.tmp$" to ignore temporary files.
    #[clap(short = 'i', long = "ignore-regex", verbatim_doc_comment)]
    pub ignore_regex: Option<Regex>,

    /// Set the log level
    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,

    /// Name of the remote to push to (if specified).
    /// Example: "origin".
    #[clap(short = 'r', long = "remote", verbatim_doc_comment)]
    pub remote: Option<String>,

    /// Number of retry attempts when errors occur.
    /// Use -1 for infinite retries.
    #[clap(long = "retries", default_value = "3", verbatim_doc_comment)]
    pub retries: i32,

    /// Enable continuous monitoring of filesystem changes.
    /// Set to false for one-time commit of current changes.
    #[clap(
        short = 'w',
        long = "watch",
        default_value = "true",
        verbatim_doc_comment
    )]
    pub watch: std::primitive::bool,
}

#[derive(Clone, Debug, clap::Args)]
#[group(multiple = false)]
pub struct CommitMessageOptions {
    #[clap(short = 'm', long = "commit-message")]
    /// Static commit message to use for all commits
    pub message: Option<String>,

    /// Path to executable script that generates commit messages.
    /// The path can be absolute or relative to the repository.
    /// The script is executed with the repository as working directory
    /// and must output the message to stdout.
    #[clap(long = "commit-message-script", verbatim_doc_comment)]
    pub script: Option<PathBuf>,
}

#[derive(Copy, Clone, Debug, Default, Deserialize, ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loglevel_conversion() {
        let conversions = [
            (LogLevel::Trace, LevelFilter::Trace),
            (LogLevel::Debug, LevelFilter::Debug),
            (LogLevel::Info, LevelFilter::Info),
            (LogLevel::Warn, LevelFilter::Warn),
            (LogLevel::Error, LevelFilter::Error),
        ];

        for (input, expected) in conversions {
            assert_eq!(LevelFilter::from(input), expected);
        }
    }
}
