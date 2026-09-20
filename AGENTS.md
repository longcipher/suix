# Suix — Agent Instructions

This document guides AI agents working in this repository. It is adapted from the
`rust-workspace-template` conventions and tailored to the Suix project.

## Scope

- Rust **workspace** with `[workspace]` in the root `Cargo.toml`.
- `bin/suix/` is the single CLI binary crate (`suix`).
- `crates/vanity/`, `crates/rpc/`, `crates/grpc/` are reusable library crates.
- No frontend/web-framework assumptions.

## Project Overview

Suix is a CLI tool for Sui blockchain operations:

- `vanity` — multi-threaded vanity address generation (hexspeak, hex, regex).
- `rpc` — Sui JSON-RPC client (generic calls + quick common methods).
- `grpc` — native Sui gRPC client (`sui-rpc-api`) with raw calls and streaming.
- `suix` — the binary that wires the above into CLI subcommands via `clap`.

## Execution Strategy

- Maximize parallelism by dispatching subagents aggressively and consuming tokens
  freely to complete tasks faster, while respecting the cargo constraint below.

## Tool Usage & Commands (Critical)

- **NEVER execute `cargo` commands in parallel.** Cargo uses strict file locks on
  the `target/` directory.
- ALWAYS run `cargo check`, `cargo build`, or `cargo test` sequentially. Wait for
  one to finish before starting the next.
- When fixing errors, execute the file `Write`/`Edit` tool FIRST, wait for it to
  succeed, and only THEN run `cargo` commands to verify. Do not parallelize file
  edits with cargo builds.

## Build Configuration

- Optionally wrap `rustc` with [kache](https://github.com/nicholasgasior/kache)
  via `.cargo/config.toml` to cache compilation artifacts:

  ```toml
  [build]
  rustc-wrapper = "kache"
  ```

  Only enable this if `kache` is installed; otherwise builds will fail.

## Cargo Workspace Rules (Critical)

1. Never manually type dependency versions in `Cargo.toml`; use `cargo add`.
2. Add workspace-level dependencies with:

   ```bash
   cargo add <crate> --workspace
   ```

3. Add sub-crate dependencies with:

   ```bash
   cargo add <crate> -p <crate-name> --workspace
   ```

4. Root `[workspace.dependencies]` must use numeric versions only (or git deps).
5. Root `[workspace.dependencies]` must not carry features by default; features are
   enabled per crate.
6. Sub-crates must use `workspace = true` for `version`, `edition`, `license`, and
   `repository`, plus `workspace = true` for shared dependencies.
7. `sui-*` dependencies (`sui-keys`, `sui-types`, `sui-rpc-api`) come from the
   MystenLabs/sui git repo and are pinned to a `rev`. Keep them on the same rev.

## Dependencies & Conventions

This project intentionally uses the following crates that are otherwise discouraged
by the generic template, because they are already established here:

- `reqwest` for JSON-RPC HTTP calls (with `json` feature).
- `anyhow` in the `grpc` crate for ergonomic error handling.
- `color-eyre` for top-level error reporting in the binary.

When adding *new* dependencies, prefer the workspace's existing choices and keep
shared crates in `[workspace.dependencies]`. Prefer `tokio`, `tracing`, `serde`,
`clap`, `eyre`, `thiserror`, and `rayon` for new code.

## Engineering Principles

### Error Handling

- Binary / application layer: `eyre` / `color-eyre`.
- Library layer: `eyre` is currently used in `vanity` and `rpc`; keep error types
  explicit and ergonomic. Prefer `thiserror` for new library error types.

### Concurrency

- `vanity` uses `rayon` for CPU-bound parallel address generation.
- `grpc` uses `tokio` for async and streaming.
- Release synchronous locks before `.await` points.

### Observability

- Logging uses `tracing` in new code; the binary may use `println!` for user output.
- Avoid `dbg!` and `println!` for library diagnostics.

### Safety

- `unsafe` is only acceptable when strictly necessary; document safety invariants.

## Testing Requirements

- Unit tests: colocate with implementation (`#[cfg(test)]`).
- Use `cargo test` as the TDD inner loop; write a failing test first, then implement
  the smallest change that makes it pass.
- Cover boundary conditions (empty inputs, zero values, maximum sizes).
- Integration tests: place in crate-level `tests/`.
- Add tests for behavioral changes and public API changes.
- Mutation testing: run `just mutation` (cargo-mutants) when available.

## Development Workflow

When fixing failures, identify the root cause first, then apply idiomatic fixes
instead of suppressing warnings or patching symptoms.

After each feature or bug fix, run:

```bash
just format
just lint
just test
```

If any command fails, report the failure and do not claim completion.

## Git Restrictions

- NEVER use `git worktree`. All code modifications MUST be made directly on the
  current branch in the existing working directory.
- Keep one logical change per commit; do not mix unrelated changes.

## Language Requirement

- Documentation, comments, and commit messages must be English only.
