# Repository Guidelines

## Project Structure & Module Organization
- `src/` contains the Rust library (`src/lib.rs`), compiler pipeline (`src/lexer.rs`, `src/parser.rs`, `src/compiler/`), AST (`src/ast/`), built-in functions (`src/functions.rs`), and optional WASM bindings (`src/wasm/`).
- `src/bin/` hosts CLI and debug binaries (e.g., `src/bin/cli.rs`).
- `tests/` holds integration tests plus LESS fixtures under `tests/fixtures/`.
- `examples/` provides runnable usage samples (e.g., `cargo run --example basic_example`).
- `docs/`, `debug/`, and `dev_tests/` contain supplemental notes and experiments.

## Build, Test, and Development Commands
- `cargo build` compiles the library and binaries.
- `cargo test` runs unit + integration tests.
- `cargo test <pattern>` filters tests by name (e.g., `cargo test nesting`).
- `cargo run --example basic_example` runs a sample program from `examples/`.
- `cargo build --features cli` builds the CLI binary `rust-less`.
- `./build-wasm.sh` builds WASM packages (requires `wasm-pack`).

## Coding Style & Naming Conventions
- Use `rustfmt` formatting (4-space indent, standard Rust style).
- Prefer `clippy` clean code: `cargo clippy --all-targets --all-features`.
- Rust naming: `snake_case` for functions/modules, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for consts.

## Testing Guidelines
- Frameworks: Rust built-in test harness (`cargo test`), `insta` for snapshots, `criterion` for benches.
- Keep tests close to behavior: unit tests in module files, integration tests in `tests/`.
- Name tests descriptively, e.g., `test_imports`, `test_color_functions_implementation`.

## Commit & Pull Request Guidelines
- Recent commits use short prefixes like `feat:` and `fix:`; follow Conventional Commits when possible (per README), but expect some legacy deviations.
- PRs should include: a clear description, test coverage for changes, and updates to docs/examples when behavior changes.
- Favor TDD: add or update tests before implementation when feasible.

## Configuration & Features
- Optional features: `cli`, `wasm`, `serde` (see `Cargo.toml`).
- CLI usage examples live in `README.md`; keep them in sync with behavior.

## Current Status & Roadmap
- Current version: 0.2.3; test suite reports 140 passing tests with a few ignored advanced cases (see `README.md`).
- Near-term focus: finish string function parsing edge cases and tighten error reporting.
- Planned features: source maps, Maps data structure, and a plugin system; WASM build is supported via `./build-wasm.sh`.
## Milestone Checklist (Synced With README)
- v0.2.4 (1-2 weeks): fix string function parsing (`e()`, `replace()`), add edge-case tests, improve error details, run perf baselines.
- v0.3.0 (1-2 months): ship minimal source maps, refine `@import` options, align CLI flags with library config.
- v0.4.0 (2-3 months): Maps data structure, full source map outputs, draft plugin hook design.
- v1.0.0 (6-12 months): LSP, build-tool plugins, standardized WASM release flow, versioned plugin API.
