# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/croissong/gitwatch-rs/compare/v0.1.0...v0.1.1) - 2025-07-30

### Fixed

- *(repo)* fix git remote auth

### Other

- *(nix)* simplify LD_LIBRARY_PATH in devshell
- bump deps; use cargo-edit to update deps
- *(just)* add comments for running single tests
- *(repo)* increase coverage
- tweak error logs
- *(repo)* tweak open repo log msg
- add systemd service example & nix service module

## [0.1.0](https://github.com/croissong/gitwatch-rs/releases/tag/v0.1.0) - 2025-07-14

Initial commit

### Features
- Watch a local Git repository and automatically commit changes
- Optionally push to a remote
- Use a custom commit message or generate one via a script
- Configure a debounce time to limit commit frequency
