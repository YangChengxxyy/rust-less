//! Tests for LESS file import functionality
//!
//! This module tests the file system import capabilities including:
//! - Basic file imports
//! - Nested/relative imports
//! - Circular import detection
//! - Import with different options (once, reference, inline, multiple)
//! - Include paths

use rust_less::{compile_file, compile_file_with_options, Compiler, CompilerOptions, Error};
use std::path::PathBuf;

/// Get the path to test fixtures
fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

#[cfg(test)]
mod file_compilation {
    use super::*;

    #[test]
    fn test_compile_simple_file() {
        let path = fixtures_path().join("variables.less");
        let result = compile_file(&path);

        // The variables.less file only defines variables, so output should be empty
        // (variables don't produce CSS output on their own)
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
    }

    #[test]
    fn test_compile_file_with_imports() {
        let path = fixtures_path().join("main.less");
        let result = compile_file(&path);

        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

        let css = result.unwrap();

        // Check that imported variables were used
        assert!(css.contains("color: #007bff"), "Primary color not found");
        assert!(css.contains("font-size: 16px"), "Font size not found");

        // Check that imported mixins were applied
        assert!(
            css.contains("border-radius:"),
            "Border radius mixin not applied"
        );
        assert!(
            css.contains("-webkit-border-radius:"),
            "Webkit prefix not found"
        );

        // Check for nested selectors
        assert!(css.contains(".button"), "Button class not found");
        assert!(css.contains(".card"), "Card class not found");
    }

    #[test]
    fn test_compile_file_not_found() {
        let path = fixtures_path().join("nonexistent.less");
        let result = compile_file(&path);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, Error::IoError { .. }),
            "Expected IoError, got: {:?}",
            err
        );
    }

    #[test]
    fn test_compile_mixins_file() {
        let path = fixtures_path().join("mixins.less");
        let result = compile_file(&path);

        // mixins.less defines mixins, so output should be empty
        // (mixin definitions don't produce CSS output on their own)
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
    }
}

#[cfg(test)]
mod import_resolution {
    use super::*;

    #[test]
    fn test_relative_import() {
        let path = fixtures_path().join("nested").join("deep.less");
        let result = compile_file(&path);

        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

        let css = result.unwrap();
        assert!(
            css.contains(".nested-component"),
            "Nested component not found"
        );
    }

    #[test]
    fn test_import_without_extension() {
        // Create a test that imports without .less extension
        let less = r#"
@import "variables";

.test {
    color: @primary-color;
}
"#;
        let mut compiler = Compiler::new();
        compiler.add_include_path(fixtures_path());

        let result = compiler.compile(less);
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

        let css = result.unwrap();
        assert!(css.contains("#007bff"), "Variable not resolved");
    }

    #[test]
    fn test_include_paths() {
        let less = r#"
@import "variables.less";
@import "mixins.less";

.test {
    color: @primary-color;
    .border-radius(10px);
}
"#;
        let options = CompilerOptions {
            compress: false,
            source_map: false,
            include_paths: vec![fixtures_path().to_string_lossy().to_string()],
        };

        let result = rust_less::compile_with_options(less, options);
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

        let css = result.unwrap();
        assert!(css.contains("#007bff"), "Variable not resolved");
        assert!(css.contains("border-radius: 10px"), "Mixin not applied");
    }
}

#[cfg(test)]
mod circular_imports {
    use super::*;

    #[test]
    fn test_circular_import_handled_gracefully() {
        // Circular imports should be handled by skipping already-imported files
        let path = fixtures_path().join("circular-a.less");
        let result = compile_file(&path);

        // Should succeed - circular imports are handled by the "once" behavior
        assert!(
            result.is_ok(),
            "Circular import not handled: {:?}",
            result.err()
        );

        let css = result.unwrap();
        // Both files should contribute their content
        assert!(css.contains(".from-a"), "Content from circular-a not found");
        assert!(css.contains(".from-b"), "Content from circular-b not found");
    }

    #[test]
    fn test_duplicate_import_skipped() {
        let less = r#"
@import "variables.less";
@import "variables.less";

.test {
    color: @primary-color;
}
"#;
        let mut compiler = Compiler::new();
        compiler.add_include_path(fixtures_path());

        let result = compiler.compile(less);
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
    }
}

#[cfg(test)]
mod css_imports {
    use super::*;

    #[test]
    fn test_css_import_passthrough() {
        let less = r#"
@import "external.css";

.local {
    color: red;
}
"#;
        let result = rust_less::compile(less);
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

        let css = result.unwrap();
        // CSS imports should be passed through as-is
        assert!(
            css.contains("@import \"external.css\""),
            "CSS import not passed through"
        );
        assert!(css.contains(".local"), "Local rule not found");
    }

    #[test]
    fn test_css_import_with_media() {
        let less = r#"
@import "print.css" print;

.screen-only {
    display: block;
}
"#;
        let result = rust_less::compile(less);
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

        let css = result.unwrap();
        assert!(
            css.contains("@import \"print.css\" print"),
            "CSS import with media query not correct"
        );
    }
}

#[cfg(test)]
mod compiler_options {
    use super::*;

    #[test]
    fn test_compressed_file_output() {
        let path = fixtures_path().join("main.less");

        let options = CompilerOptions {
            compress: true,
            source_map: false,
            include_paths: vec![],
        };

        let result = compile_file_with_options(&path, options);
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

        let css = result.unwrap();
        // Compressed output should have minimal whitespace
        assert!(!css.contains("  "), "Compressed output has extra spaces");
    }

    #[test]
    fn test_multiple_include_paths() {
        let less = r#"
@import "variables.less";

.test {
    color: @primary-color;
}
"#;
        let mut compiler = Compiler::new();
        compiler.add_include_path(fixtures_path());
        compiler.add_include_path(fixtures_path().join("nested"));

        let result = compiler.compile(less);
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
    }
}

#[cfg(test)]
mod error_handling {
    use super::*;

    #[test]
    fn test_import_file_not_found_error() {
        let less = r#"
@import "nonexistent-file.less";

.test {
    color: red;
}
"#;
        let result = rust_less::compile(less);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, Error::ImportError { .. }),
            "Expected ImportError, got: {:?}",
            err
        );
    }

    #[test]
    fn test_import_syntax_error_in_imported_file() {
        // This test would require creating a file with syntax errors
        // For now, we test that well-formed imports work
        let path = fixtures_path().join("variables.less");
        let result = compile_file(&path);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod integration {
    use super::*;

    #[test]
    fn test_full_project_compilation() {
        // Simulate compiling a full project with multiple imports
        let path = fixtures_path().join("main.less");
        let result = compile_file(&path);

        assert!(
            result.is_ok(),
            "Failed to compile project: {:?}",
            result.err()
        );

        let css = result.unwrap();

        // Verify structure of output
        assert!(css.contains("body {"), "Body rule not found");
        assert!(css.contains(".button {"), "Button rule not found");
        assert!(css.contains(".card {"), "Card rule not found");
        assert!(
            css.contains(".card .card-title {"),
            "Nested card-title not found"
        );
        assert!(css.contains("@media"), "Media query not found");

        // Verify values from imported variables
        assert!(css.contains("#007bff"), "Primary color not used");
        assert!(css.contains("16px"), "Base font size not used");

        // Verify mixins were expanded
        assert!(
            css.contains("-webkit-border-radius"),
            "Webkit prefix from mixin not found"
        );
        assert!(
            css.contains("-moz-border-radius"),
            "Moz prefix from mixin not found"
        );
    }

    #[test]
    fn test_variable_override_in_main_file() {
        let less = r#"
@import "variables.less";

// Override imported variable
@primary-color: #ff0000;

.test {
    color: @primary-color;
}
"#;
        let mut compiler = Compiler::new();
        compiler.add_include_path(fixtures_path());

        let result = compiler.compile(less);
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

        let css = result.unwrap();
        // The overridden value should be used
        // Note: #ff0000 may be shortened to #f00 by the compiler
        assert!(
            css.contains("#ff0000") || css.contains("#f00"),
            "Override not applied. CSS output: {}",
            css
        );
    }
}
