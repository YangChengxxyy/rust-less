# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rust-less is a high-performance LESS to CSS compiler written in Rust. It supports variables, nesting, mixins with guards, color/math functions, `:extend()`, file imports with cycle detection, and variable interpolation.

## Build and Test Commands

```bash
cargo build                          # Build library
cargo build --features cli           # Build CLI binary
cargo test                           # Run all tests
cargo test <pattern>                 # Run tests matching pattern (e.g., cargo test nesting)
cargo test --test test_imports       # Run specific integration test file
cargo run --example basic_example    # Run example from examples/
cargo clippy --all-targets --all-features  # Lint
./build-wasm.sh                      # Build WASM package (requires wasm-pack)
```

## Architecture

The compiler follows a traditional pipeline: Lexer → Parser → AST → Compiler → CSS output.

```
src/
├── lib.rs              # Public API: compile(), compile_file(), Compiler
├── lexer.rs            # Tokenizer
├── parser.rs           # Recursive descent parser, builds AST
├── ast/
│   ├── mod.rs          # Core AST node definitions (Statement, Rule, MixinDefinition)
│   ├── expressions.rs  # Expression types and operations
│   ├── selectors.rs    # Selector handling
│   └── values.rs       # Value types (colors, dimensions, strings)
├── compiler/
│   ├── mod.rs          # Main Compiler struct and orchestration
│   ├── expression.rs   # Expression evaluation
│   ├── mixin.rs        # Mixin expansion and guards
│   ├── rule.rs         # CSS rule generation
│   ├── import.rs       # @import handling with file resolution
│   ├── at_rule.rs      # @media, @keyframes, etc.
│   └── sourcemap.rs    # Source map generation (basic)
├── wasm/
│   ├── mod.rs          # WASM module entry
│   └── bindings.rs     # wasm-bindgen bindings
├── functions.rs        # Built-in functions (color, math, string)
├── extend.rs           # :extend() processing
└── error.rs            # Error types with position info
```

Key design points:
- Parser produces `ast::Stylesheet` containing `Vec<Statement>`
- Compiler maintains variable/mixin scopes and expands mixins inline
- Two output modes: `Compiler::new()` (pretty) and `Compiler::compressed()`

## Features

Optional Cargo features: `cli`, `wasm`, `serde`, `functions` (default enabled).

## Testing

- Unit tests in module files, integration tests in `tests/`
- Test fixtures in `tests/fixtures/`
- Uses `insta` for snapshots, `criterion` for benchmarks
- Follow TDD: write failing test first, then implement

## Code Style

- Use `rustfmt` and `clippy`
- Conventional Commits for commit messages (`feat:`, `fix:`, etc.)
