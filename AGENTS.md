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
- Optional features: `cli`, `wasm`, `serde`, `lsp` (see `Cargo.toml`).
- CLI usage examples live in `README.md`; keep them in sync with behavior.

## Current Status & Roadmap
- Current version: 1.0.0; test suite reports 400 passing tests (`--all-features` 415); less.js compat gate: 85 pass / 0 fail under strict-mappings (see `README.md`).
- 2026-09-09: Maps 高级语义与 LESS 4.x 边界对齐已完成（重复键 last-wins、单位/负数键、map 值惰性求值、插值/复合键、map 作属性值报错；A 类原生用例 +5）；变量系统完成 LESS 4.x 惰性对齐（定义域惰性求值、循环引用保护、块收尾悬空报错、DR/map 别名调用解引用）；性能基线经 `tools/perf-check/run-perf-check.sh` 随版本维护。
- v1.0.0 生态组件：`rust-less-lsp` 语言服务器（feature `lsp`）、构建工具插件（`packages/`）、WASM 发布流程（`./build-wasm.sh`）、版本化插件 API（`rust_less::plugin::PluginBundle`）。
## Milestone Checklist (Synced With README)
- v0.2.4 (1-2 weeks): fix string function parsing (`e()`, `replace()`), add edge-case tests, improve error details, run perf baselines.
- v0.3.0 ✅ done: minimal source maps, combined `@import` options (comma-separated list, unknown-option error, orthogonal optional/multiple), CLI flags aligned with `CompilerOptions`.
- v0.4.0 ✅ done: Maps data structure, deep source map alignment (strict-mappings), plugin hook design (`docs/PLUGIN_HOOKS_DESIGN.md`, `Compiler::register_function`).
- v1.0.0 ✅ done (2026-09-09): LSP server (`rust-less-lsp`, feature `lsp`), build-tool plugins (Webpack loader, Vite plugin, Rollup plugin in `packages/`), standardized WASM release flow (`./build-wasm.sh` + `docs/WASM_RELEASE.md` + CI workflow), versioned plugin API freeze + packaging/discovery convention (`PluginBundle`, `docs/PLUGIN_PACKAGING.md`).
