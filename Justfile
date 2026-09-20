# Default recipe to display help
default:
  @just --list

# Format all code
format:
  taplo fmt
  rumdl fmt .
  cargo sort -w -g
  cargo fmt --all

# Auto-fix linting issues
fix:
  taplo fmt
  rumdl check --fix .
  RUSTC_WRAPPER= cargo clippy --fix --all --allow-dirty
  cargo workspace-inheritance-check --fix

# Run all lints
lint:
  taplo fmt --check
  typos
  rumdl check .
  cargo sort -w -g -c
  cargo fmt --all -- --check
  RUSTC_WRAPPER= cargo clippy --all -- -D warnings
  cargo shear
  cargo workspace-inheritance-check

# Run tests
test:
  cargo test --workspace

# Run mutation tests with cargo-mutants
mutation:
  cargo mutants

# Run tests with coverage
test-coverage:
  cargo tarpaulin --all-features --workspace --timeout 300

# Build entire workspace
build:
  cargo build --workspace

# Check all targets compile
check:
  cargo check --all-targets --workspace

# Check for Chinese characters
check-cn:
  rg --line-number --column "\p{Han}"

# Full CI check
ci: lint test build

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

# Generate documentation for the workspace
docs:
  cargo doc --no-deps --open
