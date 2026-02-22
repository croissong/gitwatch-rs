# Changelog

## [0.1.2](https://github.com/croissong/gitwatch-rs/compare/v0.1.1...v0.1.2) - 2025-12-11

### Dependencies
- chore(deps): bump the cargo-deps group with 2 updates
- chore(deps): bump actions/checkout from 5 to 6
- chore(deps): bump the cargo-deps group with 2 updates
- chore(deps): bump the cargo-deps group with 4 updates
- chore(deps): bump rust from 1.90-slim to 1.91-slim
- chore(deps): bump the cargo-deps group with 2 updates
- chore(deps): bump the cargo-deps group with 3 updates
- chore(deps): bump regex from 1.11.3 to 1.12.2 in the cargo-deps group
- chore(deps): bump the cargo-deps group with 3 updates
- chore(deps): bump the cargo-deps group with 4 updates
- chore(deps): bump rust from 1.89-slim to 1.90-slim
- chore(deps): bump tempfile from 3.21.0 to 3.22.0
- chore(deps): bump serde from 1.0.219 to 1.0.223
- chore(deps): bump log from 0.4.27 to 0.4.28
- chore(deps): bump clap from 4.5.46 to 4.5.47
- chore(deps): bump clap from 4.5.45 to 4.5.46
- chore(deps): bump tempfile from 3.20.0 to 3.21.0
- chore(deps): bump regex from 1.11.1 to 1.11.2
- chore(deps): bump clap_complete from 4.5.55 to 4.5.57
- chore(deps): bump anyhow from 1.0.98 to 1.0.99
- chore(deps): bump actions/checkout from 4 to 5
- chore(deps): bump clap from 4.5.43 to 4.5.45
- chore(deps): bump rust from 1.88-slim to 1.89-slim
- chore(deps): bump notify-debouncer-full from 0.5.0 to 0.6.0
- chore(deps): bump clap from 4.5.42 to 4.5.43
- chore(deps): bump rust from 1.82-slim to 1.88-slim ([#25](https://github.com/croissong/gitwatch-rs/pull/25))
- chore(deps): update codecov-action to latest version

### Other
- docs: improve example readme
- test: fix flaky test (2)
- test: increase VERIFY_TIMEOUT to fix flaky test

hopefully
- build: include maintainer in arch pkg & docker image
- build(docker): improve dockerfile
- docs: tweak package description
- docs(readme): add repology badge
- docs: minor tweaks
- build(arch): update arch package & change to bin pkg
- docs: update example.png
- docs: add crate badge to readme

## [0.1.1](https://github.com/croissong/gitwatch-rs/compare/v0.1.0...v0.1.1) - 2025-07-31

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
