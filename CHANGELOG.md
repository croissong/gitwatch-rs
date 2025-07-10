# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/croissong/gitwatch-rs/releases/tag/v0.1.0) - 2025-07-10

### Added

- initial commit

### Fixed

- *(nix)* simplify package definition
- *(cli)* fix completion script bin name

### Other

- *(publish)* fix docker push job private repo access
- *(test_repo)* fix early cleanup of remote tmpdir
- add libgit2 to devshell; add note on cargo update nightly toolchain requirement
- fix clippy warnings
- update deps
- update flake.lock
- *(nix)* add nix pkg meta attributes & libgit2 dependency; mv files to pkg dir
- change package version to 0.1.0
- *(pkg)* add arch package config
- improve readme
- reduce integration test flakiness
- tweak log messages
Initial release
