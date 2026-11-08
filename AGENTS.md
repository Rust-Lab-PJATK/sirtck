# Repository Guidelines

## Project Structure & Module Organization

SIRTCK is a Rust application that checks GitHub pull requests against task-specific tests. Application entry and orchestration live in `src/main.rs`. Keep GitHub API work in `src/pr_task_manager.rs`, task-file loading in `src/file_task_manager.rs`, and checkout/test execution in `src/check_branch_manager.rs`. Domain types belong in `src/entity/` (for example, `task.rs` and `score.rs`).

Task definitions are stored in `tasks/tasks.json`; task test fixtures are organized under `tasks/<task-id>/tests/`. Add a new task using the same layout, such as `tasks/zad2/tests/`.

## Build, Test, and Development Commands

- `cargo build` — compile the checker in debug mode.
- `cargo run --release` — run the checker with production optimizations; it requires `.env` values described below.
- `cargo test` — run the repository test suite.
- `cargo fmt --check` — verify Rust formatting; run `cargo fmt` to apply it.
- `cargo clippy -- -D warnings` — reject lint warnings, matching the command SIRTCK runs against submissions.

## Coding Style & Naming Conventions

Use rustfmt defaults (four-space indentation) and idiomatic Rust: `snake_case` for functions, variables, modules, and fields; `CamelCase` for types and traits; `SCREAMING_SNAKE_CASE` for constants. Keep modules focused on a single responsibility and place reusable domain structures in `entity`. Prefer `Result` and `?` for fallible work; reserve `expect` for invariants with an informative message. Document public items with `///` when their behavior is not obvious.

## Testing Guidelines

Write unit tests beside the code they cover or integration tests in `tests/`. Use descriptive names such as `parse_test_summary_counts_measured_tests`. Task fixtures use Rust's built-in `#[test]` framework. Run `cargo test` and the strict Clippy command before opening a pull request; there is no configured coverage threshold.

## Commit & Pull Request Guidelines

Existing history uses short, imperative summaries such as `Update Cargo.toml`; follow that style and keep each commit scoped. Pull requests should explain the behavior change, reference the relevant task or issue, and list the commands run. Include sample console or PR-comment output when changing scoring or feedback formatting.

## Configuration & Security

Create an untracked `.env` with `GITHUB_TOKEN`, `GITHUB_OWNER`, and `GITHUB_REPO_NAME`. Never commit tokens or other credentials. Treat task JSON paths and cloned pull-request code as untrusted input.
