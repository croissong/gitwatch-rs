use std::{fs, path::PathBuf, str::FromStr};

use clap::{CommandFactory, Parser};
use clap_mangen::Man;
use gitwatch_rs::cli::CliOptions;

#[derive(Parser)]
enum Cli {
    /// Generate man page
    Man {},
}

fn main() {
    let cli = Cli::parse();

    match cli {
        Cli::Man {} => generate_man_page(),
    }
}

fn generate_man_page() {
    let cmd = CliOptions::command();
    let man = Man::new(cmd);
    let mut buffer = Vec::new();
    man.render(&mut buffer).expect("Failed to render man page");

    let manpage_file = PathBuf::from_str("docs").unwrap().join("gitwatch.1");
    fs::write(&manpage_file, buffer).expect("Failed to write man page");

    println!("Generated man page {}", manpage_file.display());
}
