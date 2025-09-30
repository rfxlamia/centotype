# Repository Guidelines

## Project Structure & Module Organization
- Workspace crates: `core/` (domain, scoring), `engine/` (event loop, input, render), `content/` (generation, cache), `analytics/`, `cli/`, `persistence/`, `platform/`, and binary in `centotype-bin/`.
- Tests: unit tests live next to sources; integration/E2E in `centotype-bin/tests/`. Benches in `benches/` and `benchmarks/examples/`.
- Docs: `docs/` (architecture, performance, contribution). Scripts in `scripts/`.

## Build, Test, and Development Commands
- Build all crates: `cargo build --workspace`
- Run binary: `cargo run -p centotype -- <args>` (package name `centotype` in `centotype-bin`)
- Test all crates: `cargo test --workspace`
- Test specific crate: `cargo test -p centotype-core`
- Test library only (skip examples): `cargo test -p centotype-content --lib`
- Fast CI-style run with timeout: `timeout 60s cargo test -q` (rerun with +15s if it times out)
- Lints: `cargo clippy --all-targets --all-features -D warnings`
- Format: `cargo fmt --all`

## Coding Style & Naming Conventions
- Rust 2021 edition; forbid `unsafe` (see `workspace.lints`). Avoid `unwrap()`/`panic!`; use `Result` with `anyhow`/`thiserror`.
- Names: modules/functions `snake_case`; types/traits `PascalCase`; constants `SCREAMING_SNAKE_CASE`.
- Keep public APIs documented; prefer small, composable functions.

## Testing Guidelines
- Framework: Rust `#[test]`, property tests via `proptest`, benches via `criterion`.
- Naming: `mod xyz/tests.rs` or `xyz.rs` with `#[cfg(test)]`. E2E tests under `centotype-bin/tests/`.
- Run focused tests: `cargo test -p centotype-core some_test_name`
- If examples in `content` block builds, run `--lib` or exclude that crate from workspace test and test dependents per-crate.

## Commit & Pull Request Guidelines
- Use Conventional Commit style where possible: `feat:`, `fix:`, `docs:`, `build:`, `refactor:`, `test:`.
- Subject: imperative, <= 72 chars; body explains what/why. Reference issues.
- PRs: clear description, linked issues, test evidence (commands/output), update docs when behavior changes. Include screenshots only if UI output is relevant.

## Security & Configuration Tips
- Input validation and panic safety are enforced (Clippy denies `unwrap_used`, `panic`).
- Prefer non-blocking I/O (`tokio`) and avoid long critical sections; use `parking_lot`/`Arc` patterns already present.
- Use scripts in `scripts/` for CI and release tasks.
