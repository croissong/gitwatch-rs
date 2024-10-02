pub mod app;
pub mod app_config;
pub mod cli;
pub mod commit_message;
pub mod config_file;
pub mod filter;
pub mod logger;
pub mod repo;
pub mod util;
pub mod watcher;

#[cfg(test)]
pub mod test_support {
    pub mod constants {
        include!("../tests/support/constants.rs");
    }
}
