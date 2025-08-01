# Changelog

## [0.1.2](https://github.com/croissong/gitwatch-rs/compare/v0.1.1...v0.1.2) - 2025-08-01

### Other

- docs: update example.png
- minor: tweak log messages
- docs: add crate badge to readme
- ci(release-plz): tweak changelog format (2)## [0.1.1](https://github.com/croissong/gitwatch-rs/compare/v0.1.0...v0.1.1) - 2025-07-31

### Fixed

- fix(repo): fix git remote auth

### Other

- ci(release-plz): tweak changelog format

- ci(release): use personal access token for release workflows

- docs(readme): tweak wording

- minor(nix): simplify LD_LIBRARY_PATH in devshell

- chore: bump deps; use cargo-edit to update deps

- docs(just): add comments for running single tests

- test(repo): increase coverage

- minor: tweak error logs

- minor(repo): tweak open repo log msg

- docs: add systemd service example & nix service module

## [0.1.0](https://github.com/croissong/gitwatch-rs/releases/tag/v0.1.0) - 2025-07-14

Initial commit

### Features
- Watch a local Git repository and automatically commit changes
- Optionally push to a remote
- Use a custom commit message or generate one via a script
- Configure a debounce time to limit commit frequency
