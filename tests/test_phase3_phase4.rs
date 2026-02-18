use rust_less::Compiler;

mod supports_nesting {
    use super::*;

    #[test]
    fn test_supports_nested_in_rule() {
        let input = r#"
        .container {
            color: red;
            @supports (display: grid) {
                display: grid;
            }
        }
        "#;
        let mut compiler = Compiler::new();
        let result = compiler.compile(input).unwrap();
        assert!(
            result.contains(".container {"),
            "Should have .container rule: {}",
            result
        );
        assert!(
            result.contains("color: red"),
            "Should have color: red: {}",
            result
        );
        assert!(
            result.contains("@supports (display: grid)"),
            "Should have @supports: {}",
            result
        );
        assert!(
            result.contains(".container {\n    display: grid;\n  }"),
            "Selector should bubble into @supports: {}",
            result
        );
    }

    #[test]
    fn test_supports_top_level() {
        let input = r#"
        @supports (display: flex) {
            .flex-item {
                display: flex;
            }
        }
        "#;
        let mut compiler = Compiler::new();
        let result = compiler.compile(input).unwrap();
        assert!(
            result.contains("@supports (display: flex)"),
            "Got: {}",
            result
        );
        assert!(result.contains(".flex-item"), "Got: {}", result);
    }
}

mod keyframes {
    use super::*;
    use sourcemap::SourceMap;

    #[test]
    fn test_keyframes_with_variables() {
        let input = r#"
        @start-opacity: 0;
        @keyframes fadeIn {
            from { opacity: @start-opacity; }
            to { opacity: 1; }
        }
        "#;
        let mut compiler = Compiler::new();
        let result = compiler.compile(input).unwrap();
        assert!(result.contains("@keyframes fadeIn"), "Got: {}", result);
        assert!(
            result.contains("opacity: 0"),
            "Variable should be resolved: {}",
            result
        );
        assert!(result.contains("opacity: 1"), "Got: {}", result);
    }

    #[test]
    fn test_keyframes_percentage() {
        let input = r#"
        @keyframes slide {
            0% { transform: translateX(0); }
            50% { transform: translateX(50px); }
            100% { transform: translateX(100px); }
        }
        "#;
        let mut compiler = Compiler::new();
        let result = compiler.compile(input).unwrap();
        assert!(result.contains("0%"), "Got: {}", result);
        assert!(result.contains("50%"), "Got: {}", result);
        assert!(result.contains("100%"), "Got: {}", result);
    }

    #[test]
    fn test_keyframes_source_map_lookup_points_to_declaration() {
        let input = r#"
        @keyframes fadeIn {
            from { opacity: 0; }
            to { opacity: 1; }
        }
        "#;
        let expected_src_line = input
            .lines()
            .position(|line| line.contains("opacity: 0;"))
            .expect("Expected keyframe declaration line in input")
            as u32;
        let expected_src_col = input
            .lines()
            .nth(expected_src_line as usize)
            .and_then(|line| line.find("opacity"))
            .expect("Expected keyframe declaration column in input")
            as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        let css = compiler.compile(input).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let generated_line =
            css.lines()
                .position(|line| line.contains("opacity: 0;"))
                .expect("Expected keyframe declaration line in generated CSS") as u32;

        let token = sm
            .lookup_token(generated_line, 0)
            .expect("Expected source-map token for keyframe declaration");
        let source = token
            .get_source()
            .expect("Expected source file for keyframe token");

        assert!(
            source.ends_with("input.less"),
            "Expected inline compile source to be input.less, got: {}",
            source
        );
        assert_eq!(
            token.get_src_line(),
            expected_src_line,
            "Expected keyframe src line {} but got {}",
            expected_src_line,
            token.get_src_line()
        );
        assert_eq!(
            token.get_src_col(),
            expected_src_col,
            "Expected keyframe src col {} but got {}",
            expected_src_col,
            token.get_src_col()
        );
        assert_eq!(
            token.get_name(),
            Some("opacity"),
            "Expected keyframe declaration token name to be opacity"
        );
    }
}

mod clippy_clean {
    use super::*;

    #[test]
    fn test_no_regressions_basic() {
        let input = r#"
        @color: #333;
        .header {
            color: @color;
            font-size: 16px;
            &:hover {
                color: darken(@color, 10%);
            }
        }
        .mixin(@size) {
            font-size: @size;
        }
        .body {
            .mixin(14px);
        }
        "#;
        let mut compiler = Compiler::new();
        let result = compiler.compile(input).unwrap();
        assert!(result.contains(".header"), "Got: {}", result);
        assert!(result.contains("color: #333"), "Got: {}", result);
        assert!(result.contains(".header:hover"), "Got: {}", result);
        assert!(result.contains("font-size: 14px"), "Got: {}", result);
    }
}
