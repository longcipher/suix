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
  just check-mutation-scope

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

# Mutation testing is intentionally absent from the default loop and from CI.
#
# Whole-workspace mutation runs are PROHIBITED here: a `cargo mutants` run over
# the whole workspace takes hours and rarely finishes, so it is never a gate and
# never a completion check. Only the lines the current change touched are
# mutated, within the /pb-build budget: at most 3 files, 60 mutants, 300 s.
#
#   just mutation                            # every changed .rs file
#   just mutation crates/core/src/parser.rs  # one specific file
#
# The budget is verified before the run starts, so an over-large change fails
# immediately instead of grinding for hours. A surviving mutant is a test
# failure: strengthen the assertion, or record an exemption in
# `.cargo/mutants.toml` under `exclude_re` with a one-line reason.
#
# Mutate the lines this change touched, within the /pb-build budget.
mutation *args:
  #!/usr/bin/env bash
  set -euo pipefail
  max_files=3
  max_mutants=60
  max_seconds=300

  base=""
  upstream=$(git rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null || true)
  for ref in "$upstream" origin/main origin/master main master; do
    [ -n "$ref" ] || continue
    if git rev-parse --verify --quiet "$ref" >/dev/null 2>&1; then
      base=$(git merge-base HEAD "$ref" 2>/dev/null || true)
      [ -n "$base" ] && break
    fi
  done
  [ -n "$base" ] || base=HEAD

  work=$(mktemp -d)
  trap 'rm -rf "$work"' EXIT
  if [ "$#" -gt 0 ]; then
    printf '%s\n' "$@" | sed '/^[[:space:]]*$/d' > "$work/files"
  else
    git diff --name-only --diff-filter=ACMR "$base" -- '*.rs' | sort -u > "$work/files"
  fi
  file_count=$(grep -c . "$work/files" || true)
  if [ "$file_count" -eq 0 ]; then
    echo "error: no changed Rust files against $base; there is nothing to mutate." >&2
    echo "       Whole-workspace mutation is prohibited: it takes hours and rarely finishes." >&2
    exit 2
  fi
  if [ "$file_count" -gt "$max_files" ]; then
    echo "error: $file_count files exceed the ${max_files}-file mutation budget." >&2
    echo "       Mutate fewer files at a time; never raise the budget, never run the workspace." >&2
    exit 3
  fi

  : > "$work/diff"
  while IFS= read -r file; do
    [ -n "$file" ] || continue
    git diff --no-ext-diff --no-textconv "$base" -- "$file" >> "$work/diff" || true
  done < "$work/files"
  if ! grep -q '^diff --git ' "$work/diff"; then
    echo "error: the diff against $base is empty; there is nothing to mutate." >&2
    exit 2
  fi
  RUSTC_WRAPPER= cargo mutants --workspace --in-diff "$work/diff" --list --no-times > "$work/list" 2>&1 \
    || { cat "$work/list" >&2; echo "error: cargo-mutants could not list the mutants." >&2; exit 1; }
  mutant_count=$(grep -Ec ':[0-9]+:[0-9]+:' "$work/list" || true)
  if [ "$mutant_count" -gt "$max_mutants" ]; then
    echo "error: $mutant_count mutants exceed the ${max_mutants}-mutant budget." >&2
    echo "       Narrow the change; never raise the budget, never fall back to a workspace run." >&2
    exit 3
  fi
  if [ "$mutant_count" -eq 0 ]; then
    echo "ok: no mutants in the changed lines."
    exit 0
  fi

  echo "mutating $mutant_count mutant(s) in $file_count file(s) (budget: $max_mutants mutants / ${max_seconds}s)"
  if command -v timeout >/dev/null 2>&1; then
    timeout "$max_seconds" env RUSTC_WRAPPER= cargo mutants --workspace --in-diff "$work/diff" --timeout-multiplier 2 --no-times
  elif command -v gtimeout >/dev/null 2>&1; then
    gtimeout "$max_seconds" env RUSTC_WRAPPER= cargo mutants --workspace --in-diff "$work/diff" --timeout-multiplier 2 --no-times
  else
    echo "warning: neither 'timeout' nor 'gtimeout' is available; the ${max_seconds}s wall-clock cap is not enforced." >&2
    echo "         Install coreutils ('brew install coreutils') to get 'gtimeout'." >&2
    env RUSTC_WRAPPER= cargo mutants --workspace --in-diff "$work/diff" --timeout-multiplier 2 --no-times
  fi

# Fail if an unbounded mutation command reappears in a build or CI file.
# Whole-workspace mutation runs take hours and rarely finish, so they are
# prohibited: the only permitted forms are a diff-scoped (`--in-diff`), a
# file-scoped (`--file`), or a branch-scoped (`--since`) run.
#
# Check that no build or CI file regains an unbounded mutation command.
check-mutation-scope:
  #!/usr/bin/env bash
  set -euo pipefail
  status=0
  for file in Justfile justfile Makefile makefile .github/workflows/*.yml .github/workflows/*.yaml; do
    [ -f "$file" ] || continue
    offenders=$(grep -nE 'cargo[[:space:]]+mutants' "$file" \
      | grep -vE '^[0-9]+:[[:space:]]*#' \
      | grep -vE -- '--in-diff|--file|--list|--since|-f[[:space:]]' \
      || true)
    if [ -n "$offenders" ]; then
      echo "error: $file runs an unbounded mutation command:" >&2
      printf '%s\n' "$offenders" >&2
      echo "       Whole-workspace mutation runs are prohibited. Use the diff-scoped 'just mutation'." >&2
      status=1
    fi
  done
  exit "$status"

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
