[working-directory: 'example']
run:
  cargo run -- watch . --log-level=debug

clippy:
  cargo clippy --tests --fix --allow-dirty --allow-staged

test:
  cargo nextest run --no-capture
  ## test single module:
  # cargo nextest run --no-capture repo

test-coverage:
  # https://github.com/xd009642/tarpaulin/issues/1076
  cargo tarpaulin --skip-clean --engine llvm  --target-dir target-tarpaulin
  ## test single module:
  # cargo tarpaulin --skip-clean --engine llvm  --target-dir target-tarpaulin -- repo

generate-manpage:
  cargo xtask man

release: generate-manpage

update:
  # requires switching to nightly in rust-toolchain.toml
  cargo update --breaking -Z unstable-options
  cargo udeps
