# Repository Guidelines

## Project Structure & Module Organization

The workspace is a multi-crate Rust backend. `src/main.rs` bootstraps the HTTP gateway and delegates to the `star-river` crate. Infrastructure crates live in `database/`, `entity/`, and `migration/` (SeaORM models and schema), while `engine/`, `virtual-trading/`, and `strategy-stats/` host the strategy runtime. `event-center/` and `exchange-client/` provide messaging and the MetaTrader5 bridge; keep `.venv/` inside the latter untouched. Examples and sandboxes stay under `examples/`.

## Build, Test & Development Commands

Use `cargo check --workspace` for fast dependency validation. Format with `cargo fmt --all` and lint via `cargo clippy --workspace --all-targets -- -D warnings`. Run the full suite with `cargo test --workspace`. Start the service using `RUST_LOG=info cargo run`, which binds to `SERVER_ADDR` (default `0.0.0.0:3100`). Schema maintenance lives in the `migration` crate: `cargo run -- generate <name>` produces a migration, followed by `cargo run -- up` or `down`. To reset local SQLite state, run `sea-orm-cli migrate refresh -d ./migration` and regenerate entities using `sea-orm-cli generate entity -o ./entity/src --with-serde both`.

## Coding Style & Naming Conventions

Rust code follows four-space indentation and idiomatic naming (snake_case functions, CamelCase types). Respect workspace boundaries: request/response glue stays inside `star-river`, heavy orchestration belongs in `engine` or `star-river-core`. Always run `cargo fmt` (configured for the 2024 edition through `rustfmt.toml`). Python utilities under `exchange-client/.../script` should stay synced with existing folder names; prefer `black` defaults if formatting is required.

## Testing Guidelines

Collocate unit tests in module-level `mod tests` blocks and put cross-crate scenarios in `tests/`. Name tests after behavior (`engine_order_flow_handles_cancel`). Async cases should use `#[tokio::test(flavor = "multi_thread")]`. When interacting with storage, use temporary SQLite files or in-memory connections so CI remains deterministic.

## Commit & Pull Request Guidelines

Commits in this repo follow Conventional Commit syntax (`type(scope): summary`), e.g., `fix(backtest): guard playback end`. Keep each commit focused and squash noisy fixups before pushing. PRs must describe the problem, the solution, and any API or schema impact;


## Commit Comment Format
When I'm asking to summary a commit comment, you should review uncommitted change, and provide a submission document using Chinese.
THe commit format is like:
feat/refactor/fix(module): commit title
- detail 1
- detail 2
- detail 3
important: Don't list the file path.

## Security & Configuration Tips

Keep secrets confined to your ignored `.env`; share values via secure channels rather than commits. Windows machines running the MetaTrader bridge should monitor cleanup logs under `logs/` after each run. When adding new external endpoints, document expected credentials in the private runbook and audit tracing spans to ensure sensitive fields are masked.
