use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub fn normalize_path(path: &Path) -> Result<PathBuf> {
    let path_str = path.to_str().context("Invalid path")?;
    let expanded = shellexpand::full(path_str)?;
    Ok(PathBuf::from(expanded.as_ref()).canonicalize()?)
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use testresult::TestResult;

    use super::*;

    #[test]
    fn test_normalize_path() -> TestResult {
        let temp_dir = tempfile::tempdir()?;
        let home = temp_dir.path();
        env::set_var("HOME", home.to_str().unwrap());

        let test_path = home.join("path");
        fs::create_dir_all(&test_path)?;

        let test_cases = vec![
            // absolute path (should succeed)
            (home.to_path_buf(), Ok(home.canonicalize()?)),
            // environment variable expansions (should succeed)
            (PathBuf::from("$HOME/path"), Ok(test_path.canonicalize()?)),
            (PathBuf::from("${HOME}/path"), Ok(test_path.canonicalize()?)),
            // non-existent paths (should fail)
            (PathBuf::from("~/test"), Err(())),
            // invalid paths (should fail)
            (PathBuf::from("\0invalid"), Err(())),
        ];

        for (input, expected) in test_cases {
            let result = normalize_path(&input);
            match expected {
                Ok(expected_path) => {
                    assert!(result.is_ok());
                    assert_eq!(result.unwrap(), expected_path);
                }
                Err(_) => {
                    assert!(result.is_err());
                }
            }
        }

        env::remove_var("HOME");
        Ok(())
    }
}
