# Suix - development workflows.
# Install `just` (brew install just | cargo install just), then run `just setup`.
# There is no CI workflow in this repo: `just lint` is the gate, and every
# tool it needs is installed by `just setup`.

set shell := ["bash", "-cu"]

# Default recipe to display help
default:
  @just --list

# ============================================================
# Formatting
# ============================================================

# Format all code (markdown, TOML, Rust)
format:
  rumdl fmt .
  taplo fmt
  cargo sort -w -g
  cargo fmt --all

# Auto-fix what `format` cannot fix (markdown lint, clippy, workspace inheritance)
fix:
  rumdl check --fix .
  RUSTC_WRAPPER= cargo clippy --fix --all --allow-dirty
  cargo workspace-inheritance-check --fix

# ============================================================
# Lints
# ============================================================

# Run all lints (this is the gate)
lint:
  typos
  rumdl check .
  taplo fmt --check
  cargo sort -w -g -c
  cargo fmt --all -- --check
  RUSTC_WRAPPER= cargo clippy --all --all-targets --all-features --locked -- -D warnings
  cargo shear
  cargo workspace-inheritance-check

# Check for Chinese characters
check-cn:
  #!/usr/bin/env bash
  set -euo pipefail
  if rg --line-number --column "\p{Han}"; then
    echo "error: Chinese characters found (paths above)" >&2
    exit 1
  fi
  echo "ok: no Chinese characters found"

# Run clippy with the pedantic, nursery and cargo lint groups
clippy-all:
  RUSTC_WRAPPER= cargo clippy --all --all-targets --all-features -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo

# Check for unused dependencies
unused-deps:
  cargo shear

# ============================================================
# Tests
# ============================================================

# Run the workspace test suite
test:
  cargo test --workspace

# Run mutation tests with cargo-mutants
mutation:
  cargo mutants

# Run tests with coverage
test-coverage:
  cargo tarpaulin --all-features --workspace --timeout 300

# Run the workspace tests with cargo-nextest
test-nextest:
  cargo nextest run --all-features

# ============================================================
# Build & check
# ============================================================

# Build the entire workspace
build:
  cargo build --workspace

# Check all targets compile
check:
  cargo check --all-targets --workspace

# ============================================================
# CI
# ============================================================

# Full CI check
ci: lint test build

# Full check (lint + test + build + audit)
full-check: lint test build audit

# ============================================================
# Maintenance & Tools
# ============================================================

# Clean build artifacts
clean:
  cargo clean

# Install all required development tools
setup:
  cargo install cargo-mutants
  cargo install cargo-shear
  cargo install cargo-sort
  cargo install cargo-workspace-inheritance-check
  cargo install typos-cli
  cargo install rumdl
  cargo install taplo-cli
  cargo install cargo-tarpaulin
  cargo install cargo-nextest
  cargo install cargo-audit
  cargo install cargo-outdated

# Generate documentation for the workspace
docs:
  cargo doc --no-deps --open

# Run a security audit
audit:
  cargo audit

# Check for outdated dependencies
outdated:
  cargo outdated

# Run miri on unsafe code (needs a nightly toolchain)
miri:
  cargo +nightly miri test --all-features

# Update all dependencies
update-deps:
  cargo update
