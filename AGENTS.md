# Repository Guidelines

## Project Structure & Module Organization
Centotype is a Rust workspace. Core gameplay logic lives in `core/`, with scoring and session orchestration handled by `engine/`. The terminal interface and CLI argument parsing sit in `cli/` and `centotype-bin/src/main.rs`. Persistence adapters are under `persistence/`, analytics pipelines in `analytics/`, content generators in `content/`, and platform integrations in `platform/`. Performance artifacts live in `benchmarks/`, design notes in `docs/`, and samples in `examples/`. Shared validation and release helpers reside in `scripts/` alongside crate-specific test harnesses (for example `engine/tests/`).

## Build, Test, and Development Commands
Run `cargo build --workspace` for a fast debug build and `cargo build --release --workspace` before benchmarking or packaging. Execute the CLI locally with `cargo run -p centotype-bin -- play --level 1`. `scripts/validate_local.sh` mirrors the CI pipeline; use `--quick` to skip long-running checks while iterating. Format code via `cargo fmt --all`, lint with `cargo clippy --workspace --all-targets -- -D warnings`, and generate docs with `cargo doc --workspace --no-deps` when needed.

## Coding Style & Naming Conventions
Crates target Rust 1.75+ and rely on default `rustfmt` rules (4-space indentation, trailing commas). Favor idiomatic Rust naming: snake_case for modules/functions, PascalCase for types, SCREAMING_SNAKE_CASE for constants. Keep command modules in `cli/` focused on orchestration; place reusable logic in `core/` or `engine/`. CI enforces `cargo fmt` and `cargo clippy`; run both before pushing.

## Testing Guidelines
`cargo test --workspace --lib` runs unit coverage across crates. Integration and property suites live in `engine/tests/` and other crate-specific `tests/` folders; run them with `cargo test --workspace --test '*'`. Slow performance and fuzz harnesses are marked `#[ignore]`; surface regressions locally with `cargo test --package centotype-engine -- --ignored`. Criterion baselines sit in `benchmarks/`; build them via `cargo bench --no-run` before comparison runs.

## Commit & Pull Request Guidelines
History currently contains only the bootstrap commit, so adopt conventional, imperative summaries (for example `feat(engine): tighten latency budget`). Reference related issues in the body, summarize validation steps (include `scripts/validate_local.sh` output or key `cargo` commands), and attach benchmark diffs or screenshots when relevant. PRs should flag performance-sensitive changes, security implications, and whether ignored tests were exercised.

## Security & Dependency Checks
Run `cargo audit` and `cargo deny check` (configured by `deny.toml`) at least once per feature branch, or allow the validation script to install and execute them. Review updates that touch `content_corpus.json` or persistence backends for data privacy and path handling.
