use std::process;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use gitwatch_rs::{
    app::App,
    app_config::AppConfig,
    cli::{Cli, Commands},
    logger::setup_logger,
};
use log::error;

fn main() {
    if let Err(e) = run() {
        error!("{e:?}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Watch(cli_opts) => {
            setup_logger(cli_opts.log_level)?;
            let config = AppConfig::new(cli_opts)?;
            let app = App::new(config)?;
            app.run(None)
        }
        Commands::Completion { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut std::io::stdout());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::str::contains;
    use testresult::TestResult;

    #[test]
    fn test_main_error_handling() -> TestResult {
        let mut cmd = Command::cargo_bin("gitwatch")?;
        cmd.args(["watch", "--commit-message", "test", "/nonexistent-path"]);

        cmd.assert()
            .failure()
            .stderr(contains("Invalid repository path"));
        Ok(())
    }
}
