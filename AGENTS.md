# Suix Agent Instructions

## Non-negotiables

- Surgical edits only: every changed line must trace to the request. No drive-by refactors, no reformatting untouched code, no deleting pre-existing dead code (mention it instead).
- Fix root causes. Never silence a warning, weaken a check, or patch a symptom to make a gate pass.
- Never report completion while any `just` gate below fails. Report the failure verbatim and stop.
- Stop and ask when a request is ambiguous, conflicts with existing conventions, or requires credentials, publishing, or destructive operations.
- When a correction reveals a missing rule, add one specific rule here ("always X for Y", never "be careful with Y"); delete rules that never catch real errors.

## Scope

- Rust workspace for a Sui blockchain CLI. `bin/suix` is the only binary; `crates/vanity`, `crates/rpc`, `crates/grpc` and `crates/keystore` are reusable libraries.
- `vanity` — parallel vanity-address generation (rayon): prefix/suffix/contains patterns, hexspeak mapping, Ed25519 / Secp256k1 / Secp256r1.
- `rpc` — Sui JSON-RPC client: generic `rpc_call`, quick per-method helpers, and unsigned `tx` builders in `crates/rpc/src/tx.rs`.
- `grpc` — native Sui gRPC client over `sui-rpc-api` + `tonic`: raw method calls, checkpoint streaming, balances, stakes, simulate, execute.
- `keystore` — local key management: generate, import, export, list, sign, multisig. Default file is `<data dir>/suix/sui.keystore`.
- `suix` — the `clap` CLI that wires the four crates together. `bin/suix/src/commands/mod.rs` holds the command tree and argument structs; `bin/suix/src/main.rs` holds the handlers. English only in code, comments, docs, and commit messages.

## Execution

- Parallelize aggressively: dispatch subagents for independent research, exploration, and implementation.
- NEVER run two `cargo` commands concurrently, and never overlap file edits with a running `cargo` command (`target/` file lock): finish edits, wait, then run.
- NEVER use `git worktree`. Edit the current branch in place.
- TDD for behavior changes: failing test first, smallest change to pass, then run `just mutation` and fix surviving mutants.

## Verification loop

After each feature or bug fix, in order; any failure means the task is not done:

```bash
just format
just lint
just test
just mutation
```

This repo has no CI workflow, so `just lint` is the gate: typos, rumdl, `taplo fmt --check`, `cargo sort -c`, `cargo fmt --check`, `cargo clippy -D warnings`, `cargo shear`, workspace-inheritance check. Run `just setup` once to install the tools it needs. Every test is an in-crate `#[cfg(test)]` module; there is no database or network fixture.

## Rust rules

- Toolchain is pinned to stable `1.96.1` in `rust-toolchain.toml`; the Justfile uses plain `cargo fmt` / `cargo clippy`. Do not introduce `+nightly` or `+1.96.1` variants.
- The `sui-*` git dependencies (`sui-keys`, `sui-types`, `sui-rpc-api`, `move-core-types`, `shared-crypto`) come from `MystenLabs/sui` and must all pin the SAME `rev` — two revs compile two copies of `sui-types` and produce "expected `sui_types::…`, found `sui_types::…`" mismatches. `fastcrypto` is a separate git dep with its own rev; do not fold it into the Sui rev.
- Never hand-type dependency versions. Shared deps live in root `[workspace.dependencies]` and are referenced with `dep.workspace = true`; add them with `cargo add <crate> --workspace` and enable features in the member crate.
- Members inherit `version`/`edition`/`license`/`repository`/`authors`/`readme`/`homepage`/`keywords`/`categories` via `workspace = true`; `cargo workspace-inheritance-check` is part of `just lint` and `just fix` fixes the failures.
- This workspace defines NO `[workspace.lints]` set, so clippy runs with default lints only. Do not assume `unwrap`/`expect`/`println!` are denied — but keep them out of library code anyway.

## Dependency choices

- Errors: `eyre` in `vanity`, `rpc` and `keystore`; `anyhow` in `grpc` (the tonic surface is `anyhow`-shaped); `color-eyre` in `bin/suix` for the top-level report. `thiserror` is not a dependency yet — add it with `cargo add thiserror --workspace` before introducing typed library error enums.
- HTTP: `reqwest` with the `json` feature for JSON-RPC and the GraphQL command. gRPC goes through `tonic` via `sui-rpc-api`. Do not add a third client.
- Concurrency: `rayon` for the CPU-bound vanity search, `tokio` for I/O and gRPC streaming. There is no lock-heavy shared state to speak of — keep it that way.
- There is NO logging framework in this workspace (no `tracing` dependency). The CLI prints user-facing output with `println!`; libraries return results. If you add structured logging, wire `tracing` + `tracing-subscriber` into the binary in the same change.

## Code conventions

- Unit tests live in `#[cfg(test)]` modules next to the code. Add a test for every behavior or public-API change; cover boundary conditions (empty input, zero count, max addresses).
- `suix vanity` prints the address AND the base64 private key to the terminal when no `--save-path` is given; with `--save-path` it writes the keypair file into a directory created with mode `0700`. Keep that permission, and keep the key out of logs and error messages.
- Key material is loaded from `SUIX_KEY` (bech32 `suiprivkey...` or base64 `flag || privkey`) or from the keystore file. Never echo it into error messages.
- Network defaults come from the profile constants in `bin/suix/src/commands/mod.rs` and can be overridden with `SUIX_URL` / `SUIX_GRPC_URL` or the global `--profile` / `--config` flags. Change the constants, not the call sites.
- `bin/suix/src/main.rs` dispatches on `Commands`; every new subcommand needs an arm there and a variant in `commands/mod.rs`. Keep the two in sync.
