clippy:
  cargo clippy --tests --fix --allow-dirty --allow-staged

test:
  cargo nextest run --no-capture

test-coverage:
  # https://github.com/xd009642/tarpaulin/issues/1076
  cargo tarpaulin --skip-clean --engine llvm  --target-dir target-tarpaulin

generate-manpage:
  cargo xtask man

release: generate-manpage

update:
  # requires switching to nightly in rust-toolchain.toml
  cargo update --breaking -Z unstable-options
  cargo udeps
