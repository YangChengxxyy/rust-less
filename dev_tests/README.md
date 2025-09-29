# Development Tests Directory

This directory contains temporary test files created during development and experimentation with the LESS compiler.

## Purpose

These files were created to test specific features and scenarios during the development process, particularly for:

- Media query nesting functionality
- Feature compatibility testing
- Debugging specific LESS compilation scenarios

## File Organization

The test files in this directory are primarily focused on media query nesting capabilities:

- `test_*_media*.rs` - Various media query nesting test scenarios
- `test_current_features.rs` - Testing existing compiler features
- `test_final_verification.rs` - Final verification tests

## Usage

These are standalone Rust test files that can be compiled and run individually:

```bash
# Compile and run a specific test
rustc test_file_name.rs && ./test_file_name

# Or use cargo to run if they follow cargo test conventions
cargo test --bin test_file_name
```

## Note

These are development/experimental files and may not follow the same standards as the main test suite in `/tests`. They serve as a sandbox for testing specific functionality during feature development.

For official tests, see the `/tests` directory which contains the main test suite.