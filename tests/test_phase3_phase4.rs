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

    #[test]
    fn test_supports_nested_under_media_in_rule_keeps_selector() {
        let input = r#"
        .chain-root {
            @media (min-width: 700px) {
                @supports (display: grid) {
                    color: #1492ff;
                }
            }
        }
        "#;

        let mut compiler = Compiler::new();
        let result = compiler.compile(input).unwrap();

        assert!(
            result.contains("@media (min-width: 700px)"),
            "Got: {}",
            result
        );
        assert!(
            result.contains("@supports (display: grid)"),
            "Got: {}",
            result
        );
        assert!(
            result.contains(".chain-root {\n      color: #1492ff;\n    }"),
            "Selector should remain inside nested @supports: {}",
            result
        );
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

    #[test]
    fn test_keyframes_source_map_rule_name_mapping_consistent() {
        let input = r#"
        @keyframes fadeIn {
            from { opacity: 0; }
            to { opacity: 1; }
        }
        "#;
        let expected_src_line = input
            .lines()
            .position(|line| line.contains("@keyframes fadeIn"))
            .expect("Expected @keyframes header line in input")
            as u32;
        let expected_src_col = input
            .lines()
            .nth(expected_src_line as usize)
            .and_then(|line| line.find("@keyframes"))
            .expect("Expected @keyframes header column in input")
            as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        let css = compiler.compile(input).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let generated_line =
            css.lines()
                .position(|line| line.contains("@keyframes fadeIn"))
                .expect("Expected @keyframes header line in generated CSS") as u32;

        let token = sm
            .lookup_token(generated_line, 0)
            .expect("Expected source-map token for @keyframes header");

        assert_eq!(
            token.get_src_line(),
            expected_src_line,
            "Expected @keyframes src line {} but got {}",
            expected_src_line,
            token.get_src_line()
        );
        assert_eq!(
            token.get_src_col(),
            expected_src_col,
            "Expected @keyframes src col {} but got {}",
            expected_src_col,
            token.get_src_col()
        );
        assert_eq!(
            token.get_name(),
            Some("@keyframes fadeIn"),
            "Expected @keyframes header token name to be @keyframes fadeIn"
        );
    }
}

mod media_source_map {
    use super::*;
    use sourcemap::SourceMap;

    #[test]
    fn test_media_source_map_lessjs_compat_screen_keyword_has_no_extra_header_segment() {
        let input = r#"
        @media screen {
            .a {
                margin: 1px;
            }
        }
        "#;

        let expected_src_line = input
            .lines()
            .position(|line| line.contains("@media screen"))
            .expect("Expected @media header line in input") as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        compiler.set_source_map_lessjs_compat(true);
        let css = compiler.compile(input).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let generated_line = css
            .lines()
            .position(|line| line.contains("@media screen"))
            .expect("Expected @media header line in generated CSS") as u32;

        let mut header_cols = Vec::new();
        for token in sm.tokens() {
            if token.get_dst_line() == generated_line
                && token.get_src_line() == expected_src_line
                && token
                    .get_source()
                    .map(|s| s.ends_with("input.less"))
                    .unwrap_or(false)
            {
                header_cols.push(token.get_dst_col());
            }
        }

        assert!(
            header_cols.contains(&0),
            "Expected @media header base mapping at generated col 0, got {:?}",
            header_cols
        );
        assert!(
            !header_cols.contains(&8),
            "Did not expect extra @media header mapping at generated col 8 for `@media screen`, got {:?}",
            header_cols
        );
    }

    #[test]
    fn test_media_source_map_lessjs_compat_parenthesized_query_keeps_extra_header_segment() {
        let input = r#"
        @media (max-width: 900px) {
            .b {
                margin: 2px;
            }
        }
        "#;

        let expected_src_line = input
            .lines()
            .position(|line| line.contains("@media (max-width: 900px)"))
            .expect("Expected @media header line in input") as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        compiler.set_source_map_lessjs_compat(true);
        let css = compiler.compile(input).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let generated_line = css
            .lines()
            .position(|line| line.contains("@media (max-width: 900px)"))
            .expect("Expected @media header line in generated CSS") as u32;

        let mut header_cols = Vec::new();
        for token in sm.tokens() {
            if token.get_dst_line() == generated_line
                && token.get_src_line() == expected_src_line
                && token
                    .get_source()
                    .map(|s| s.ends_with("input.less"))
                    .unwrap_or(false)
            {
                header_cols.push(token.get_dst_col());
            }
        }

        assert!(
            header_cols.contains(&0),
            "Expected @media header base mapping at generated col 0, got {:?}",
            header_cols
        );
        assert!(
            header_cols.contains(&8),
            "Expected extra @media header mapping at generated col 8 for parenthesized media query, got {:?}",
            header_cols
        );
    }

    #[test]
    fn test_media_source_map_lessjs_compat_screen_and_query_uses_feature_offset() {
        let input = r#"
        @media screen and (max-width: 900px) {
            .c {
                margin: 3px;
            }
        }
        "#;

        let expected_src_line = input
            .lines()
            .position(|line| line.contains("@media screen and (max-width: 900px)"))
            .expect("Expected @media header line in input") as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        compiler.set_source_map_lessjs_compat(true);
        let css = compiler.compile(input).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let generated_line = css
            .lines()
            .position(|line| line.contains("@media screen and (max-width"))
            .expect("Expected @media header line in generated CSS") as u32;

        let mut header_cols = Vec::new();
        for token in sm.tokens() {
            if token.get_dst_line() == generated_line
                && token.get_src_line() == expected_src_line
                && token
                    .get_source()
                    .map(|s| s.ends_with("input.less"))
                    .unwrap_or(false)
            {
                header_cols.push(token.get_dst_col());
            }
        }

        let expected_feature_col = 19u32;

        assert!(
            header_cols.contains(&0),
            "Expected @media header base mapping at generated col 0, got {:?}",
            header_cols
        );
        assert!(
            header_cols.contains(&expected_feature_col),
            "Expected extra @media header mapping at generated col {} for `@media screen and (...)`, got {:?}",
            expected_feature_col,
            header_cols
        );
    }

    #[test]
    fn test_media_source_map_lessjs_compat_only_screen_and_query_uses_feature_offset() {
        let input = r#"
        @media only screen and (max-width: 900px) {
            .d {
                margin: 4px;
            }
        }
        "#;

        let expected_src_line = input
            .lines()
            .position(|line| line.contains("@media only screen and (max-width: 900px)"))
            .expect("Expected @media header line in input") as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        compiler.set_source_map_lessjs_compat(true);
        let css = compiler.compile(input).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let generated_line = css
            .lines()
            .position(|line| line.contains("@media only screen and (max-width"))
            .expect("Expected @media header line in generated CSS") as u32;

        let mut header_cols = Vec::new();
        for token in sm.tokens() {
            if token.get_dst_line() == generated_line
                && token.get_src_line() == expected_src_line
                && token
                    .get_source()
                    .map(|s| s.ends_with("input.less"))
                    .unwrap_or(false)
            {
                header_cols.push(token.get_dst_col());
            }
        }

        let expected_feature_col = 24u32;

        assert!(
            header_cols.contains(&0),
            "Expected @media header base mapping at generated col 0, got {:?}",
            header_cols
        );
        assert!(
            header_cols.contains(&expected_feature_col),
            "Expected extra @media header mapping at generated col {} for `@media only screen and (...)`, got {:?}",
            expected_feature_col,
            header_cols
        );
    }

    #[test]
    fn test_media_source_map_lessjs_compat_comma_queries_emit_multiple_feature_segments() {
        let input = r#"
        @media only screen and (max-width: 900px), not print and (min-width: 1200px) {
            .e {
                margin: 5px;
            }
        }
        "#;

        let prelude = "only screen and (max-width: 900px), not print and (min-width: 1200px)";
        let expected_cols: Vec<u32> = prelude
            .match_indices('(')
            .map(|(offset, _)| 8 + offset as u32)
            .collect();

        let expected_src_line = input
            .lines()
            .position(|line| line.contains("@media only screen and (max-width: 900px), not print and (min-width: 1200px)"))
            .expect("Expected @media header line in input") as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        compiler.set_source_map_lessjs_compat(true);
        let css = compiler.compile(input).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let generated_line = css
            .lines()
            .position(|line| line.contains("@media only screen and (max-width"))
            .expect("Expected @media header line in generated CSS") as u32;

        let mut header_cols = Vec::new();
        for token in sm.tokens() {
            if token.get_dst_line() == generated_line
                && token.get_src_line() == expected_src_line
                && token
                    .get_source()
                    .map(|s| s.ends_with("input.less"))
                    .unwrap_or(false)
            {
                header_cols.push(token.get_dst_col());
            }
        }

        assert!(
            header_cols.contains(&0),
            "Expected @media header base mapping at generated col 0, got {:?}",
            header_cols
        );

        for col in expected_cols {
            assert!(
                header_cols.contains(&col),
                "Expected @media header feature mapping at generated col {}, got {:?}",
                col,
                header_cols
            );
        }
    }

    #[test]
    fn test_media_source_map_lessjs_compat_calc_function_and_feature_segments() {
        let input = r#"
        @media screen and (min-width: calc(100px + (2 * 10px))), print and (max-width: calc(1200px - (2 * 10px))) {
            .f {
                margin: 6px;
            }
        }
        "#;

        let expected_src_line = input
            .lines()
            .position(|line| line.contains("@media screen and (min-width: calc(100px + (2 * 10px))), print and (max-width: calc(1200px - (2 * 10px)))"))
            .expect("Expected @media header line in input") as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        compiler.set_source_map_lessjs_compat(true);
        let css = compiler.compile(input).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let generated_line = css
            .lines()
            .position(|line| line.contains("@media screen and (min-width: calc(100px + (2 * 10px))), print and (max-width: calc(1200px - (2 * 10px)))"))
            .expect("Expected @media header line in generated CSS") as u32;

        let mut header_cols = Vec::new();
        for token in sm.tokens() {
            if token.get_dst_line() == generated_line
                && token.get_src_line() == expected_src_line
                && token
                    .get_source()
                    .map(|s| s.ends_with("input.less"))
                    .unwrap_or(false)
            {
                header_cols.push(token.get_dst_col());
            }
        }

        let expected_cols = [19u32, 30u32, 68u32, 79u32];
        assert!(
            header_cols.contains(&0),
            "Expected @media header base mapping at generated col 0, got {:?}",
            header_cols
        );
        for col in expected_cols {
            assert!(
                header_cols.contains(&col),
                "Expected @media calc/feature mapping at generated col {}, got {:?}",
                col,
                header_cols
            );
        }
    }

    #[test]
    fn test_media_source_map_lessjs_compat_url_var_and_nested_functions_emit_expected_segments() {
        let input = r#"
        @media screen and (min-width: min(1200px, calc(1000px + var(--offset)))) and (background-image: url("hero.png")), print and (max-width: max(640px, calc(40vw + var(--gutter)))) {
            .g {
                margin: 7px;
            }
        }
        "#;

        let prelude = "screen and (min-width: min(1200px, calc(1000px + var(--offset)))) and (background-image: url(\"hero.png\")), print and (max-width: max(640px, calc(40vw + var(--gutter))))";
        let expected_cols = [
            8 + prelude
                .find("(min-width")
                .expect("Expected first media feature in prelude") as u32,
            7 + prelude.find("min(").expect("Expected min() in prelude") as u32,
            7 + prelude
                .find("calc(")
                .expect("Expected first calc() in prelude") as u32,
            7 + prelude.find("var(").expect("Expected first var() in prelude") as u32,
            8 + prelude
                .find("(background-image")
                .expect("Expected background-image media feature in prelude") as u32,
            8 + prelude.find("url(").expect("Expected url() in prelude") as u32 + 3,
            8 + prelude
                .find("(max-width")
                .expect("Expected max-width media feature in prelude") as u32,
            7 + prelude
                .rfind("max(")
                .expect("Expected max() in prelude") as u32,
            7 + prelude
                .rfind("calc(")
                .expect("Expected second calc() in prelude") as u32,
            7 + prelude
                .rfind("var(")
                .expect("Expected second var() in prelude") as u32,
        ];

        let expected_src_line = input
            .lines()
            .position(|line| line.contains("@media screen and (min-width: min(1200px, calc(1000px + var(--offset)))) and (background-image: url(\"hero.png\")), print and (max-width: max(640px, calc(40vw + var(--gutter))))"))
            .expect("Expected @media header line in input") as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        compiler.set_source_map_lessjs_compat(true);
        let css = compiler.compile(input).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let generated_line = css
            .lines()
            .position(|line| line.contains("@media screen and (min-width: min(1200px, calc(1000px + var(--offset)))) and (background-image: url(\"hero.png\")), print and (max-width: max(640px, calc(40vw + var(--gutter))))"))
            .expect("Expected @media header line in generated CSS") as u32;

        let mut header_cols = Vec::new();
        for token in sm.tokens() {
            if token.get_dst_line() == generated_line
                && token.get_src_line() == expected_src_line
                && token
                    .get_source()
                    .map(|s| s.ends_with("input.less"))
                    .unwrap_or(false)
            {
                header_cols.push(token.get_dst_col());
            }
        }

        assert!(
            header_cols.contains(&0),
            "Expected @media header base mapping at generated col 0, got {:?}",
            header_cols
        );
        for col in expected_cols {
            assert!(
                header_cols.contains(&col),
                "Expected @media url/var/function mapping at generated col {}, got {:?}",
                col,
                header_cols
            );
        }
    }

    #[test]
    fn test_media_source_map_lessjs_compat_clamp_min_calc_var_segments() {
        let input = r#"
        @media (min-width: clamp(320px, calc(50vw + var(--x-offset)), 1440px)) and (max-width: min(1280px, calc(960px + var(--y-offset)))) {
            .h {
                margin: 8px;
            }
        }
        "#;

        let prelude = "(min-width: clamp(320px, calc(50vw + var(--x-offset)), 1440px)) and (max-width: min(1280px, calc(960px + var(--y-offset))))";
        let expected_cols = [
            8 + prelude
                .find("(min-width")
                .expect("Expected min-width media feature in prelude") as u32,
            7 + prelude
                .find("clamp(")
                .expect("Expected clamp() in prelude") as u32,
            7 + prelude
                .find("calc(")
                .expect("Expected first calc() in prelude") as u32,
            7 + prelude.find("var(").expect("Expected first var() in prelude") as u32,
            8 + prelude
                .find("(max-width")
                .expect("Expected max-width media feature in prelude") as u32,
            7 + prelude.find("min(").expect("Expected min() in prelude") as u32,
            7 + prelude
                .rfind("calc(")
                .expect("Expected second calc() in prelude") as u32,
            7 + prelude
                .rfind("var(")
                .expect("Expected second var() in prelude") as u32,
        ];

        let expected_src_line = input
            .lines()
            .position(|line| line.contains("@media (min-width: clamp(320px, calc(50vw + var(--x-offset)), 1440px)) and (max-width: min(1280px, calc(960px + var(--y-offset))))"))
            .expect("Expected @media header line in input") as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        compiler.set_source_map_lessjs_compat(true);
        let css = compiler.compile(input).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let generated_line = css
            .lines()
            .position(|line| line.contains("@media (min-width: clamp(320px, calc(50vw + var(--x-offset)), 1440px)) and (max-width: min(1280px, calc(960px + var(--y-offset))))"))
            .expect("Expected @media header line in generated CSS") as u32;

        let mut header_cols = Vec::new();
        for token in sm.tokens() {
            if token.get_dst_line() == generated_line
                && token.get_src_line() == expected_src_line
                && token
                    .get_source()
                    .map(|s| s.ends_with("input.less"))
                    .unwrap_or(false)
            {
                header_cols.push(token.get_dst_col());
            }
        }

        assert!(
            header_cols.contains(&0),
            "Expected @media header base mapping at generated col 0, got {:?}",
            header_cols
        );
        for col in expected_cols {
            assert!(
                header_cols.contains(&col),
                "Expected @media clamp/min/calc/var mapping at generated col {}, got {:?}",
                col,
                header_cols
            );
        }
    }

    #[test]
    fn test_media_source_map_lessjs_compat_orientation_resolution_segments() {
        let input = r#"
        @media screen and (orientation: landscape) and (min-resolution: 2dppx), print and (max-resolution: 300dpi) and (min-width: calc(1200px - 10px)) {
            .i {
                margin: 9px;
            }
        }
        "#;

        let prelude = "screen and (orientation: landscape) and (min-resolution: 2dppx), print and (max-resolution: 300dpi) and (min-width: calc(1200px - 10px))";
        let expected_cols = [
            8 + prelude
                .find("(orientation")
                .expect("Expected orientation media feature in prelude") as u32,
            8 + prelude
                .find("(min-resolution")
                .expect("Expected min-resolution media feature in prelude") as u32,
            8 + prelude
                .find("(max-resolution")
                .expect("Expected max-resolution media feature in prelude") as u32,
            8 + prelude
                .find("(min-width")
                .expect("Expected min-width media feature in prelude") as u32,
            7 + prelude.find("calc(").expect("Expected calc() in prelude") as u32,
        ];

        let expected_src_line = input
            .lines()
            .position(|line| line.contains("@media screen and (orientation: landscape) and (min-resolution: 2dppx), print and (max-resolution: 300dpi) and (min-width: calc(1200px - 10px))"))
            .expect("Expected @media header line in input") as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        compiler.set_source_map_lessjs_compat(true);
        let css = compiler.compile(input).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let generated_line = css
            .lines()
            .position(|line| line.contains("@media screen and (orientation: landscape) and (min-resolution: 2dppx), print and (max-resolution: 300dpi) and (min-width: calc(1200px - 10px))"))
            .expect("Expected @media header line in generated CSS") as u32;

        let mut header_cols = Vec::new();
        for token in sm.tokens() {
            if token.get_dst_line() == generated_line
                && token.get_src_line() == expected_src_line
                && token
                    .get_source()
                    .map(|s| s.ends_with("input.less"))
                    .unwrap_or(false)
            {
                header_cols.push(token.get_dst_col());
            }
        }

        assert!(
            header_cols.contains(&0),
            "Expected @media header base mapping at generated col 0, got {:?}",
            header_cols
        );
        for col in expected_cols {
            assert!(
                header_cols.contains(&col),
                "Expected @media orientation/resolution mapping at generated col {}, got {:?}",
                col,
                header_cols
            );
        }
    }

    #[test]
    fn test_media_source_map_lessjs_compat_custom_function_name_segments() {
        let input = r#"
        @media (min-width: clamp(320px, viewport-step(50vw, var(--x-offset)), 1440px)) and (hover: hover) and (max-width: min(1280px, calc(960px + var(--y-offset)))) {
            .j {
                margin: 10px;
            }
        }
        "#;

        let prelude = "(min-width: clamp(320px, viewport-step(50vw, var(--x-offset)), 1440px)) and (hover: hover) and (max-width: min(1280px, calc(960px + var(--y-offset))))";
        let expected_cols = [
            8 + prelude
                .find("(min-width")
                .expect("Expected min-width media feature in prelude") as u32,
            7 + prelude
                .find("clamp(")
                .expect("Expected clamp() in prelude") as u32,
            7 + prelude
                .find("viewport-step(")
                .expect("Expected viewport-step() in prelude") as u32,
            7 + prelude.find("var(").expect("Expected first var() in prelude") as u32,
            8 + prelude
                .find("(hover")
                .expect("Expected hover media feature in prelude") as u32,
            8 + prelude
                .find("(max-width")
                .expect("Expected max-width media feature in prelude") as u32,
            7 + prelude.find("min(").expect("Expected min() in prelude") as u32,
            7 + prelude
                .rfind("calc(")
                .expect("Expected calc() in prelude") as u32,
            7 + prelude
                .rfind("var(")
                .expect("Expected second var() in prelude") as u32,
        ];

        let expected_src_line = input
            .lines()
            .position(|line| line.contains("@media (min-width: clamp(320px, viewport-step(50vw, var(--x-offset)), 1440px)) and (hover: hover) and (max-width: min(1280px, calc(960px + var(--y-offset))))"))
            .expect("Expected @media header line in input") as u32;

        let mut compiler = Compiler::new().with_source_map(true);
        compiler.set_source_map_lessjs_compat(true);
        let css = compiler.compile(input).unwrap();
        let source_map = compiler.generate_source_map().unwrap();
        let sm = SourceMap::from_slice(source_map.as_bytes()).expect("Expected valid source map");

        let generated_line = css
            .lines()
            .position(|line| line.contains("@media (min-width: clamp(320px, viewport-step(50vw, var(--x-offset)), 1440px)) and (hover: hover) and (max-width: min(1280px, calc(960px + var(--y-offset))))"))
            .expect("Expected @media header line in generated CSS") as u32;

        let mut header_cols = Vec::new();
        for token in sm.tokens() {
            if token.get_dst_line() == generated_line
                && token.get_src_line() == expected_src_line
                && token
                    .get_source()
                    .map(|s| s.ends_with("input.less"))
                    .unwrap_or(false)
            {
                header_cols.push(token.get_dst_col());
            }
        }

        assert!(
            header_cols.contains(&0),
            "Expected @media header base mapping at generated col 0, got {:?}",
            header_cols
        );
        for col in expected_cols {
            assert!(
                header_cols.contains(&col),
                "Expected @media custom-function mapping at generated col {}, got {:?}",
                col,
                header_cols
            );
        }
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
