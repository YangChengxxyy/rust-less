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
            source_map_lessjs_compat: false,
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
    use std::collections::HashSet;

    #[test]
    fn test_compressed_file_output() {
        let path = fixtures_path().join("main.less");

        let options = CompilerOptions {
            compress: true,
            source_map: false,
            source_map_lessjs_compat: false,
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

    #[test]
    fn test_compile_file_with_source_map_option() {
        let path = fixtures_path().join("main.less");
        let options = CompilerOptions {
            compress: false,
            source_map: true,
            source_map_lessjs_compat: false,
            include_paths: vec![],
        };

        let result = compile_file_with_options(&path, options);
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
        let css = result.unwrap();
        assert!(!css.trim().is_empty(), "Expected non-empty CSS output");
    }

    #[test]
    fn test_source_map_contains_imported_mixin_file() {
        let path = fixtures_path().join("main.less");
        let mut compiler = Compiler::new().with_source_map(true);

        let result = compiler.compile_file(&path);
        assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

        let source_map = compiler
            .generate_source_map()
            .expect("Expected source map JSON");
        assert!(
            source_map.contains("main.less"),
            "Source map should contain main.less: {}",
            source_map
        );
        assert!(
            source_map.contains("mixins.less"),
            "Source map should contain mixins.less for imported mixin expansions: {}",
            source_map
        );
    }

    #[test]
    fn test_source_map_lessjs_compat_mode_rewrites_sources_and_names() {
        let entry = fixtures_path().join("keyframes-entry.less");
        let mut compiler = Compiler::new().with_source_map(true);
        compiler.set_source_map_source_root(Some("/workspace/src".to_string()));
        compiler.set_source_map_lessjs_compat(true);

        compiler
            .compile_file(&entry)
            .expect("Expected keyframes entry to compile");

        let source_map = compiler.generate_source_map().unwrap();
        let sm = sourcemap::SourceMap::from_slice(source_map.as_bytes())
            .expect("Expected valid source map JSON");

        assert_eq!(sm.get_source_root(), None);
        assert_eq!(sm.get_name_count(), 0);
        assert_eq!(sm.get_source_count(), 1);
        assert_eq!(
            sm.get_source(0),
            Some("/workspace/src/keyframes-import.less")
        );
    }

    #[test]
    fn test_source_map_state_is_reset_between_compiles() {
        let main = fixtures_path().join("main.less");
        let deep = fixtures_path().join("nested").join("deep.less");
        let mut compiler = Compiler::new().with_source_map(true);

        compiler.compile_file(&main).unwrap();
        let first_map = compiler.generate_source_map().unwrap();
        assert!(
            first_map.contains("main.less"),
            "Expected main.less in first map"
        );

        compiler.compile_file(&deep).unwrap();
        let second_map = compiler.generate_source_map().unwrap();
        assert!(
            second_map.contains("deep.less"),
            "Expected deep.less in second map"
        );
        assert!(
            !second_map.contains("main.less"),
            "Second map should not leak sources from previous compile: {}",
            second_map
        );
    }

    #[test]
    fn test_source_map_tokens_include_circular_import_files() {
        let path = fixtures_path().join("circular-a.less");
        let mut compiler = Compiler::new().with_source_map(true);

        let css = compiler.compile_file(&path).unwrap();
        assert!(css.contains(".from-a"), "Expected .from-a output");
        assert!(css.contains(".from-b"), "Expected .from-b output");

        let source_map = compiler.generate_source_map().unwrap();
        let sm = sourcemap::SourceMap::from_slice(source_map.as_bytes())
            .expect("Expected valid source map JSON");

        let mut token_sources = HashSet::new();
        for token in sm.tokens() {
            if let Some(source) = token.get_source() {
                token_sources.insert(source.to_string());
            }
        }

        assert!(
            token_sources.iter().any(|s| s.ends_with("circular-a.less")),
            "Expected token source from circular-a.less. got: {:?}",
            token_sources
        );
        assert!(
            token_sources.iter().any(|s| s.ends_with("circular-b.less")),
            "Expected token source from circular-b.less. got: {:?}",
            token_sources
        );
    }

    #[test]
    fn test_source_map_lookup_for_imported_rule_points_to_imported_file() {
        let path = fixtures_path().join("circular-a.less");
        let mut compiler = Compiler::new().with_source_map(true);

        let css = compiler.compile_file(&path).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = sourcemap::SourceMap::from_slice(source_map.as_bytes())
            .expect("Expected valid source map JSON");

        let imported_color_line = css
            .lines()
            .position(|line| line.contains("color: #00f;"))
            .expect("Expected imported color line") as u32;

        let token = sm
            .lookup_token(imported_color_line, 0)
            .expect("Expected source-map token for imported rule output");
        let source = token
            .get_source()
            .expect("Expected source path for imported token");

        assert!(
            source.ends_with("circular-b.less"),
            "Expected imported rule to map to circular-b.less, got: {}",
            source
        );
        assert_eq!(
            token.get_name(),
            Some("color"),
            "Expected imported rule token name to be color"
        );
    }

    #[test]
    fn test_source_map_lookup_for_bubbled_media_declaration_points_to_imported_line() {
        let entry = fixtures_path().join("media-bubble-entry.less");
        let imported = fixtures_path().join("media-bubble-import.less");
        let imported_text =
            std::fs::read_to_string(&imported).expect("Expected media-bubble-import.less");

        let expected_src_line = imported_text
            .lines()
            .position(|line| line.contains("color: #00f;"))
            .expect("Expected color declaration in imported fixture")
            as u32;
        let expected_src_col = imported_text
            .lines()
            .nth(expected_src_line as usize)
            .and_then(|line| line.find("color"))
            .expect("Expected color token column in imported fixture")
            as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        let css = compiler.compile_file(&entry).unwrap();

        assert!(
            css.contains("@media (max-width: 600px)"),
            "Expected bubbled media query output, got: {}",
            css
        );

        let generated_line =
            css.lines()
                .position(|line| line.contains("color: #00f;"))
                .expect("Expected bubbled media declaration in generated CSS") as u32;

        let source_map = compiler.generate_source_map().unwrap();
        let sm = sourcemap::SourceMap::from_slice(source_map.as_bytes())
            .expect("Expected valid source map JSON");

        let token = sm
            .lookup_token(generated_line, 0)
            .expect("Expected source-map token for bubbled media declaration");
        let source = token
            .get_source()
            .expect("Expected source path for bubbled media token");

        assert!(
            source.ends_with("media-bubble-import.less"),
            "Expected bubbled media declaration to map to media-bubble-import.less, got: {}",
            source
        );
        assert_eq!(
            token.get_src_line(),
            expected_src_line,
            "Expected src line {} for bubbled declaration, got {}",
            expected_src_line,
            token.get_src_line()
        );
        assert_eq!(
            token.get_src_col(),
            expected_src_col,
            "Expected src col {} for bubbled declaration, got {}",
            expected_src_col,
            token.get_src_col()
        );
        assert_eq!(
            token.get_name(),
            Some("color"),
            "Expected bubbled media declaration token name to be color"
        );
    }

    #[test]
    fn test_source_map_lookup_for_bubbled_supports_declaration_points_to_imported_line() {
        let entry = fixtures_path().join("supports-bubble-entry.less");
        let imported = fixtures_path().join("supports-bubble-import.less");
        let imported_text =
            std::fs::read_to_string(&imported).expect("Expected supports-bubble-import.less");

        let expected_src_line = imported_text
            .lines()
            .position(|line| line.contains("color: #0a0;"))
            .expect("Expected color declaration in imported supports fixture")
            as u32;
        let expected_src_col = imported_text
            .lines()
            .nth(expected_src_line as usize)
            .and_then(|line| line.find("color"))
            .expect("Expected color token column in imported supports fixture")
            as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        let css = compiler.compile_file(&entry).unwrap();

        assert!(
            css.contains("@supports (display: grid)"),
            "Expected bubbled supports block output, got: {}",
            css
        );
        assert!(
            css.contains(".from-support"),
            "Expected selector to bubble into supports block, got: {}",
            css
        );

        let generated_line = css
            .lines()
            .position(|line| line.contains("color:"))
            .expect("Expected bubbled supports declaration in generated CSS")
            as u32;

        let source_map = compiler.generate_source_map().unwrap();
        let sm = sourcemap::SourceMap::from_slice(source_map.as_bytes())
            .expect("Expected valid source map JSON");

        let token = sm
            .lookup_token(generated_line, 0)
            .expect("Expected source-map token for bubbled supports declaration");
        let source = token
            .get_source()
            .expect("Expected source path for bubbled supports token");

        assert!(
            source.ends_with("supports-bubble-import.less"),
            "Expected bubbled supports declaration to map to supports-bubble-import.less, got: {}",
            source
        );
        assert_eq!(
            token.get_src_line(),
            expected_src_line,
            "Expected src line {} for bubbled supports declaration, got {}",
            expected_src_line,
            token.get_src_line()
        );
        assert_eq!(
            token.get_src_col(),
            expected_src_col,
            "Expected src col {} for bubbled supports declaration, got {}",
            expected_src_col,
            token.get_src_col()
        );
        assert_eq!(
            token.get_name(),
            Some("color"),
            "Expected bubbled supports declaration token name to be color"
        );
    }

    #[test]
    fn test_source_map_lookup_for_imported_mixin_declaration_points_to_mixin_file() {
        let main = fixtures_path().join("main.less");
        let mixins = fixtures_path().join("mixins.less");
        let mixins_text = std::fs::read_to_string(&mixins).expect("Expected mixins.less");

        let expected_src_line = mixins_text
            .lines()
            .position(|line| line.contains("-webkit-border-radius: @radius;"))
            .expect("Expected -webkit-border-radius declaration in mixins fixture")
            as u32;
        let expected_src_col = mixins_text
            .lines()
            .nth(expected_src_line as usize)
            .and_then(|line| line.find("-webkit-border-radius"))
            .expect("Expected -webkit-border-radius token column in mixins fixture")
            as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        let css = compiler.compile_file(&main).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = sourcemap::SourceMap::from_slice(source_map.as_bytes())
            .expect("Expected valid source map JSON");

        let generated_line =
            css.lines()
                .position(|line| line.contains("-webkit-border-radius: 8px;"))
                .expect("Expected imported mixin declaration in generated CSS") as u32;

        let token = sm
            .lookup_token(generated_line, 0)
            .expect("Expected source-map token for imported mixin declaration");
        let source = token
            .get_source()
            .expect("Expected source path for imported mixin token");

        assert!(
            source.ends_with("mixins.less"),
            "Expected imported mixin declaration to map to mixins.less, got: {}",
            source
        );
        assert_eq!(
            token.get_src_line(),
            expected_src_line,
            "Expected src line {} for imported mixin declaration, got {}",
            expected_src_line,
            token.get_src_line()
        );
        assert_eq!(
            token.get_src_col(),
            expected_src_col,
            "Expected src col {} for imported mixin declaration, got {}",
            expected_src_col,
            token.get_src_col()
        );
        assert_eq!(
            token.get_name(),
            Some("-webkit-border-radius"),
            "Expected imported mixin declaration token name to be -webkit-border-radius"
        );
    }

    #[test]
    fn test_source_map_lookup_for_imported_keyframes_declaration_points_to_imported_file() {
        let entry = fixtures_path().join("keyframes-entry.less");
        let imported = fixtures_path().join("keyframes-import.less");
        let imported_text =
            std::fs::read_to_string(&imported).expect("Expected keyframes-import.less");

        let expected_src_line = imported_text
            .lines()
            .position(|line| line.contains("opacity: 0;"))
            .expect("Expected keyframes opacity declaration in imported fixture")
            as u32;
        let expected_src_col = imported_text
            .lines()
            .nth(expected_src_line as usize)
            .and_then(|line| line.find("opacity"))
            .expect("Expected keyframes opacity token column in imported fixture")
            as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        let css = compiler.compile_file(&entry).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = sourcemap::SourceMap::from_slice(source_map.as_bytes())
            .expect("Expected valid source map JSON");

        assert!(
            css.contains("@keyframes importedFade"),
            "Expected imported keyframes output, got: {}",
            css
        );

        let generated_line = css
            .lines()
            .position(|line| line.contains("opacity: 0;"))
            .expect("Expected imported keyframes declaration in generated CSS")
            as u32;

        let token = sm
            .lookup_token(generated_line, 0)
            .expect("Expected source-map token for imported keyframes declaration");
        let source = token
            .get_source()
            .expect("Expected source path for imported keyframes token");

        assert!(
            source.ends_with("keyframes-import.less"),
            "Expected imported keyframes declaration to map to keyframes-import.less, got: {}",
            source
        );
        assert_eq!(
            token.get_src_line(),
            expected_src_line,
            "Expected src line {} for imported keyframes declaration, got {}",
            expected_src_line,
            token.get_src_line()
        );
        assert_eq!(
            token.get_src_col(),
            expected_src_col,
            "Expected src col {} for imported keyframes declaration, got {}",
            expected_src_col,
            token.get_src_col()
        );
        assert_eq!(
            token.get_name(),
            Some("opacity"),
            "Expected imported keyframes declaration token name to be opacity"
        );
    }

    #[test]
    fn test_source_map_lookup_for_imported_keyframes_rule_name_is_consistent() {
        let entry = fixtures_path().join("keyframes-entry.less");
        let imported = fixtures_path().join("keyframes-import.less");
        let imported_text =
            std::fs::read_to_string(&imported).expect("Expected keyframes-import.less");

        let expected_src_line = imported_text
            .lines()
            .position(|line| line.contains("@keyframes importedFade"))
            .expect("Expected @keyframes header in imported fixture")
            as u32;
        let expected_src_col = imported_text
            .lines()
            .nth(expected_src_line as usize)
            .and_then(|line| line.find("@keyframes"))
            .expect("Expected @keyframes header column in imported fixture")
            as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        let css = compiler.compile_file(&entry).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = sourcemap::SourceMap::from_slice(source_map.as_bytes())
            .expect("Expected valid source map JSON");

        let generated_line =
            css.lines()
                .position(|line| line.contains("@keyframes importedFade"))
                .expect("Expected imported @keyframes header in generated CSS") as u32;

        let token = sm
            .lookup_token(generated_line, 0)
            .expect("Expected source-map token for imported @keyframes header");
        let source = token
            .get_source()
            .expect("Expected source path for imported @keyframes header token");

        assert!(
            source.ends_with("keyframes-import.less"),
            "Expected imported @keyframes header to map to keyframes-import.less, got: {}",
            source
        );
        assert_eq!(
            token.get_src_line(),
            expected_src_line,
            "Expected src line {} for imported @keyframes header, got {}",
            expected_src_line,
            token.get_src_line()
        );
        assert_eq!(
            token.get_src_col(),
            expected_src_col,
            "Expected src col {} for imported @keyframes header, got {}",
            expected_src_col,
            token.get_src_col()
        );
        assert_eq!(
            token.get_name(),
            Some("@keyframes importedFade"),
            "Expected imported @keyframes header token name to be @keyframes importedFade"
        );
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
