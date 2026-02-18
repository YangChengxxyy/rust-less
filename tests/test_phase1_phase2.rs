use rust_less::Compiler;

// ========== Phase 1 Tests ==========

mod extend_fixes {
    use super::*;

    #[test]
    fn test_extend_b_does_not_match_button() {
        let input = r#"
        .button { color: red; }
        .b { color: blue; }
        .a:extend(.b all) { }
        "#;
        let mut compiler = Compiler::new();
        let result = compiler.compile(input).unwrap();
        // .b should get .a added, but .button should NOT
        assert!(
            result.contains(".b, .a"),
            "Expected .b, .a but got:\n{}",
            result
        );
        assert!(
            !result.contains(".button, .a"),
            "Should not match .button: {}",
            result
        );
    }

    #[test]
    fn test_extend_all_compound_selector() {
        let input = r#"
        .a:hover { color: red; }
        .b:extend(.a all) { }
        "#;
        let mut compiler = Compiler::new();
        let result = compiler.compile(input).unwrap();
        assert!(
            result.contains(".a:hover, .b:hover"),
            "Expected .a:hover, .b:hover but got:\n{}",
            result
        );
    }

    #[test]
    fn test_extend_all_descendant() {
        let input = r#"
        .container .a { color: red; }
        .b:extend(.a all) { }
        "#;
        let mut compiler = Compiler::new();
        let result = compiler.compile(input).unwrap();
        assert!(
            result.contains(".container .a, .container .b"),
            "Expected descendant extend: {}",
            result
        );
    }
}

mod media_query_order {
    use super::*;

    #[test]
    fn test_media_query_after_rule_not_end() {
        let input = r#"
        .first {
            width: 100px;
            @media (max-width: 768px) {
                width: 50px;
            }
        }
        .second {
            color: red;
        }
        "#;
        let mut compiler = Compiler::new();
        let result = compiler.compile(input).unwrap();
        // Media query should come AFTER .first but BEFORE .second
        let media_pos = result.find("@media").unwrap_or(usize::MAX);
        let second_pos = result.find(".second").unwrap_or(0);
        assert!(
            media_pos < second_pos,
            "Media query should appear before .second. Output:\n{}",
            result
        );
    }
}

// ========== Phase 2 Tests ==========

mod string_concatenation {
    use super::*;

    #[test]
    fn test_string_plus_string() {
        let mut compiler = Compiler::new();
        let result = compiler
            .compile(r#".test { content: "hello" + " world"; }"#)
            .unwrap();
        assert!(
            result.contains(r#"content: "hello world""#),
            "Got: {}",
            result
        );
    }

    #[test]
    fn test_string_plus_number() {
        let mut compiler = Compiler::new();
        let result = compiler
            .compile(r#".test { content: "size-" + 16; }"#)
            .unwrap();
        assert!(result.contains(r#"content: "size-16""#), "Got: {}", result);
    }
}

mod guard_operators {
    use super::*;

    #[test]
    fn test_guard_and() {
        let input = r#"
        .mixin(@a) when (@a > 0) and (@a < 10) {
            value: @a;
        }
        .test1 { .mixin(5); }
        "#;
        let mut compiler = Compiler::new();
        let result = compiler.compile(input).unwrap();
        assert!(
            result.contains("value: 5"),
            "Guard 'and' should pass for 5. Got: {}",
            result
        );
    }

    #[test]
    fn test_guard_and_fails() {
        let input = r#"
        .mixin(@a) when (@a > 0) and (@a < 10) {
            value: @a;
        }
        .test1 { .mixin(15); }
        "#;
        let mut compiler = Compiler::new();
        let result = compiler.compile(input);
        // Should fail because 15 > 10, guard doesn't match
        assert!(result.is_err() || !result.unwrap().contains("value: 15"));
    }

    #[test]
    fn test_guard_comma_or() {
        let input = r#"
        .mixin(@a) when (@a = 1), (@a = 2) {
            value: @a;
        }
        .test1 { .mixin(2); }
        "#;
        let mut compiler = Compiler::new();
        let result = compiler.compile(input).unwrap();
        assert!(
            result.contains("value: 2"),
            "Guard comma/or should pass for 2. Got: {}",
            result
        );
    }

    #[test]
    fn test_guard_not() {
        let input = r#"
        .mixin(@a) when not (@a = 0) {
            value: @a;
        }
        .test1 { .mixin(5); }
        "#;
        let mut compiler = Compiler::new();
        let result = compiler.compile(input).unwrap();
        assert!(
            result.contains("value: 5"),
            "Guard 'not' should pass for 5. Got: {}",
            result
        );
    }
}

mod variadic_mixins {
    use super::*;

    #[test]
    fn test_variadic_basic() {
        let input = r#"
        .mixin(@a, @rest...) {
            first: @a;
            rest: @rest;
        }
        .test { .mixin(1, 2, 3, 4); }
        "#;
        let mut compiler = Compiler::new();
        let result = compiler.compile(input).unwrap();
        assert!(result.contains("first: 1"), "Got: {}", result);
        assert!(result.contains("rest: 2 3 4"), "Got: {}", result);
    }

    #[test]
    fn test_arguments_variable() {
        let input = r#"
        .mixin(@a, @b) {
            all-args: @arguments;
        }
        .test { .mixin(1px, 2px); }
        "#;
        let mut compiler = Compiler::new();
        let result = compiler.compile(input).unwrap();
        assert!(result.contains("all-args: 1px 2px"), "Got: {}", result);
    }
}

mod string_functions {
    use super::*;

    #[test]
    fn test_uppercase() {
        let mut compiler = Compiler::new();
        let result = compiler
            .compile(r#".test { content: uppercase("hello"); }"#)
            .unwrap();
        assert!(result.contains(r#"content: "HELLO""#), "Got: {}", result);
    }

    #[test]
    fn test_lowercase() {
        let mut compiler = Compiler::new();
        let result = compiler
            .compile(r#".test { content: lowercase("HELLO"); }"#)
            .unwrap();
        assert!(result.contains(r#"content: "hello""#), "Got: {}", result);
    }

    #[test]
    fn test_length() {
        let mut compiler = Compiler::new();
        let result = compiler
            .compile(r#".test { count: length("hello"); }"#)
            .unwrap();
        assert!(result.contains("count: 5"), "Got: {}", result);
    }
}

mod type_check_functions {
    use super::*;

    #[test]
    fn test_isnumber() {
        let mut compiler = Compiler::new();
        let input = r#"
        .mixin(@a) when (isnumber(@a)) { value: @a; }
        .test { .mixin(10px); }
        "#;
        let result = compiler.compile(input).unwrap();
        assert!(result.contains("value: 10px"), "Got: {}", result);
    }

    #[test]
    fn test_ispixel() {
        let mut compiler = Compiler::new();
        let input = r#"
        .mixin(@a) when (ispixel(@a)) { value: @a; }
        .test { .mixin(10px); }
        "#;
        let result = compiler.compile(input).unwrap();
        assert!(result.contains("value: 10px"), "Got: {}", result);
    }
}

mod math_functions {
    use super::*;

    #[test]
    fn test_sqrt() {
        let mut compiler = Compiler::new();
        let result = compiler.compile(r#".test { value: sqrt(16); }"#).unwrap();
        assert!(result.contains("value: 4"), "Got: {}", result);
    }

    #[test]
    fn test_pow() {
        let mut compiler = Compiler::new();
        let result = compiler.compile(r#".test { value: pow(2, 3); }"#).unwrap();
        assert!(result.contains("value: 8"), "Got: {}", result);
    }

    #[test]
    fn test_pi() {
        let mut compiler = Compiler::new();
        let result = compiler.compile(r#".test { value: pi(); }"#).unwrap();
        assert!(result.contains("3.14159"), "Got: {}", result);
    }

    #[test]
    fn test_mod() {
        let mut compiler = Compiler::new();
        let result = compiler.compile(r#".test { value: mod(10, 3); }"#).unwrap();
        assert!(result.contains("value: 1"), "Got: {}", result);
    }

    #[test]
    fn test_sin_cos() {
        let mut compiler = Compiler::new();
        let result = compiler.compile(r#".test { value: sin(0); }"#).unwrap();
        assert!(result.contains("value: 0"), "Got: {}", result);

        let mut compiler2 = Compiler::new();
        let result2 = compiler2.compile(r#".test { value: cos(0); }"#).unwrap();
        assert!(result2.contains("value: 1"), "Got: {}", result2);
    }
}

mod source_map {
    use super::*;
    use sourcemap::SourceMap;

    #[test]
    fn test_source_map_generation() {
        let mut compiler = Compiler::new().with_source_map(true);
        let _result = compiler.compile(r#".test { color: red; }"#).unwrap();
        let sm = compiler.generate_source_map();
        assert!(sm.is_some(), "Source map should be generated when enabled");
        let json = sm.unwrap();
        assert!(
            json.contains("\"version\":3"),
            "Should be v3 source map: {}",
            json
        );
        assert!(
            json.contains("input.less"),
            "Should reference source file: {}",
            json
        );
    }

    #[test]
    fn test_source_map_disabled() {
        let mut compiler = Compiler::new();
        let _result = compiler.compile(r#".test { color: red; }"#).unwrap();
        let sm = compiler.generate_source_map();
        assert!(sm.is_none(), "Source map should be None when disabled");
    }

    #[test]
    fn test_source_map_rule_name_mapping_consistent() {
        let mut compiler = Compiler::new().with_source_map(true);
        let css = compiler.compile(r#".box { color: red; }"#).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let selector_line = css
            .lines()
            .position(|line| line.contains(".box {"))
            .expect("Expected selector line in generated CSS") as u32;

        let token = sm
            .lookup_token(selector_line, 0)
            .expect("Expected source-map token for selector line");

        assert_eq!(
            token.get_name(),
            Some(".box"),
            "Expected rule selector name in source-map token"
        );
    }

    #[test]
    fn test_source_map_property_name_mapping_consistent() {
        let mut compiler = Compiler::new().with_source_map(true);
        let css = compiler.compile(r#".box { color: red; }"#).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let property_line = css
            .lines()
            .position(|line| line.contains("color: red;"))
            .expect("Expected property line in generated CSS") as u32;

        let token = sm
            .lookup_token(property_line, 0)
            .expect("Expected source-map token for property line");

        assert_eq!(
            token.get_name(),
            Some("color"),
            "Expected declaration property name in source-map token"
        );
    }

    #[test]
    fn test_source_map_css_import_mapping_consistent() {
        let input = r#"
        @import "external.css";
        .box { color: red; }
        "#;
        let expected_src_line = input
            .lines()
            .position(|line| line.contains("@import \"external.css\";"))
            .expect("Expected @import in input") as u32;
        let expected_src_col = input
            .lines()
            .nth(expected_src_line as usize)
            .and_then(|line| line.find('@'))
            .expect("Expected @ in @import input line") as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        let css = compiler.compile(input).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let import_line = css
            .lines()
            .position(|line| line.contains("@import \"external.css\";"))
            .expect("Expected @import in generated CSS") as u32;

        let token = sm
            .lookup_token(import_line, 0)
            .expect("Expected source-map token for @import line");

        assert_eq!(
            token.get_src_line(),
            expected_src_line,
            "Expected @import src line {} but got {}",
            expected_src_line,
            token.get_src_line()
        );
        assert_eq!(
            token.get_src_col(),
            expected_src_col,
            "Expected @import src col {} but got {}",
            expected_src_col,
            token.get_src_col()
        );
        assert_eq!(
            token.get_name(),
            Some("@import"),
            "Expected @import name in source-map token"
        );
    }

    #[test]
    fn test_source_map_block_comment_mapping_consistent() {
        let input = r#"
        .box {
            /* note */
            color: red;
        }
        "#;
        let expected_src_line = input
            .lines()
            .position(|line| line.contains("/* note */"))
            .expect("Expected block comment in input") as u32;
        let expected_src_col = input
            .lines()
            .nth(expected_src_line as usize)
            .and_then(|line| line.find("/*"))
            .expect("Expected block comment start column in input")
            as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        let css = compiler.compile(input).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let comment_line = css
            .lines()
            .position(|line| line.contains("/* note */"))
            .expect("Expected block comment in generated CSS") as u32;

        let token = sm
            .lookup_token(comment_line, 0)
            .expect("Expected source-map token for comment line");

        assert_eq!(
            token.get_src_line(),
            expected_src_line,
            "Expected comment src line {} but got {}",
            expected_src_line,
            token.get_src_line()
        );
        assert_eq!(
            token.get_src_col(),
            expected_src_col,
            "Expected comment src col {} but got {}",
            expected_src_col,
            token.get_src_col()
        );
    }

    #[test]
    fn test_source_map_merged_property_name_mapping_consistent() {
        let input = r#"
        .box {
            box-shadow+: 1px 1px #000;
            box-shadow+: 2px 2px #333;
        }
        "#;
        let expected_src_line = input
            .lines()
            .position(|line| line.contains("box-shadow+: 1px 1px #000;"))
            .expect("Expected first merged declaration in input")
            as u32;
        let expected_src_col = input
            .lines()
            .nth(expected_src_line as usize)
            .and_then(|line| line.find("box-shadow"))
            .expect("Expected merged declaration column in input")
            as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        let css = compiler.compile(input).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let merged_line = css
            .lines()
            .position(|line| line.contains("box-shadow:"))
            .expect("Expected merged property in generated CSS") as u32;

        let token = sm
            .lookup_token(merged_line, 0)
            .expect("Expected source-map token for merged property line");

        assert_eq!(
            token.get_name(),
            Some("box-shadow"),
            "Expected merged property name in source-map token"
        );
        assert_eq!(
            token.get_src_line(),
            expected_src_line,
            "Expected merged property src line {} but got {}",
            expected_src_line,
            token.get_src_line()
        );
        assert_eq!(
            token.get_src_col(),
            expected_src_col,
            "Expected merged property src col {} but got {}",
            expected_src_col,
            token.get_src_col()
        );
    }

    #[test]
    fn test_source_map_multi_selector_rule_name_mapping_consistent() {
        let mut compiler = Compiler::new().with_source_map(true);
        let css = compiler.compile(r#".a, .b { color: red; }"#).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let selector_line =
            css.lines()
                .position(|line| line.contains(".a, .b {"))
                .expect("Expected multi-selector line in generated CSS") as u32;

        let token = sm
            .lookup_token(selector_line, 0)
            .expect("Expected source-map token for multi-selector line");

        assert_eq!(
            token.get_name(),
            Some(".a, .b"),
            "Expected joined multi-selector name in source-map token"
        );
    }

    #[test]
    fn test_source_map_descendant_multi_selector_rule_name_mapping_consistent() {
        let input = r#"
        .parent .x, .parent .y {
            color: red;
        }
        "#;
        let mut compiler = Compiler::new().with_source_map(true);
        let css = compiler.compile(input).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let selector_line = css
            .lines()
            .position(|line| line.contains(".parent .x, .parent .y {"))
            .expect("Expected descendant multi-selector line in generated CSS")
            as u32;

        let token = sm
            .lookup_token(selector_line, 0)
            .expect("Expected source-map token for descendant multi-selector line");

        assert_eq!(
            token.get_name(),
            Some(".parent .x, .parent .y"),
            "Expected descendant multi-selector name in source-map token"
        );
    }
}

mod unit_handling {
    use super::*;

    #[test]
    fn test_division_same_units_cancel() {
        let mut compiler = Compiler::new();
        let result = compiler.compile(r#".test { value: 10px / 2px; }"#).unwrap();
        // Same units cancel: 10px / 2px = 5 (unitless)
        assert!(
            result.contains("value: 5;"),
            "Same units should cancel: {}",
            result
        );
    }

    #[test]
    fn test_modulo_preserves_unit() {
        let mut compiler = Compiler::new();
        let result = compiler
            .compile(r#".test { value: mod(10px, 3); }"#)
            .unwrap();
        assert!(
            result.contains("value: 1px"),
            "Mod should preserve unit: {}",
            result
        );
    }
}
