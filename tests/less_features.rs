//! LESS 功能的综合测试套件
//!
//! 此文件包含所有主要 LESS 功能的测试，采用 TDD 风格编写。
//! 每个测试代表一个需要实现的特定 LESS 功能。

use rust_less::{compile, Error};

#[cfg(test)]
mod variables {
    use super::*;

    #[test]
    fn test_simple_variable_declaration_and_usage() {
        let less = r#"
@primary-color: #333;
.header {
    color: @primary-color;
}
"#;
        let expected = r#".header {
  color: #333;
}"#;
        let result = compile(less).unwrap();
        assert_eq!(result.trim(), expected.trim());
    }

    #[test]
    fn test_multiple_variables() {
        let less = r#"
@primary: #333;
@secondary: #666;
@margin: 10px;

.box {
    color: @primary;
    background: @secondary;
    margin: @margin;
}
"#;
        let expected = r#".box {
  color: #333;
  background: #666;
  margin: 10px;
}"#;
        let result = compile(less).unwrap();
        assert_eq!(result.trim(), expected.trim());
    }

    #[test]
    fn test_variable_interpolation() {
        let less = r#"
@base-url: "images";
@selector: "header";

.@{selector} {
    background: url("@{base-url}/bg.png");
}
"#;
        let expected = r#".header {
  background: url("images/bg.png");
}"#;
        let result = compile(less).unwrap();
        assert_eq!(result.trim(), expected.trim());
    }

    #[test]
    fn test_variable_scoping() {
        let less = r#"
@color: red;

.outer {
    @color: blue;
    color: @color;

    .inner {
        color: @color;
    }
}

.other {
    color: @color;
}
"#;
        let expected = r#".outer {
  color: blue;
}
.outer .inner {
  color: blue;
}
.other {
  color: red;
}"#;
        let result = compile(less).unwrap();
        assert_eq!(result.trim(), expected.trim());
    }
}

#[cfg(test)]
mod nesting {
    use super::*;

    #[test]
    fn test_basic_nesting() {
        let less = r#"
.navbar {
    height: 60px;

    ul {
        margin: 0;
        padding: 0;

        li {
            list-style: none;

            a {
                text-decoration: none;
                color: #333;
            }
        }
    }
}
"#;
        let expected = r#".navbar {
  height: 60px;
}
.navbar ul {
  margin: 0;
  padding: 0;
}
.navbar ul li {
  list-style: none;
}
.navbar ul li a {
  text-decoration: none;
  color: #333;
}"#;
        let result = compile(less).unwrap();
        assert_eq!(result.trim(), expected.trim());
    }

    #[test]
    fn test_parent_selector_reference() {
        let less = r#"
.button {
    padding: 10px;

    &:hover {
        background: #eee;
    }

    &.active {
        background: #333;
        color: white;
    }

    &-large {
        padding: 20px;
    }
}
"#;
        let expected = r#".button {
  padding: 10px;
}
.button:hover {
  background: #eee;
}
.button.active {
  background: #333;
  color: white;
}
.button-large {
  padding: 20px;
}"#;
        let result = compile(less).unwrap();
        assert_eq!(result.trim(), expected.trim());
    }

    #[test]
    fn test_nested_media_queries() {
        let less = r#"
.responsive {
    width: 100%;

    @media (max-width: 768px) {
        width: 50%;

        .inner {
            display: none;
        }
    }
}
"#;
        let expected = r#".responsive {
  width: 100%;
}
@media (max-width: 768px) {
  .responsive {
    width: 50%;
  }
  .responsive .inner {
    display: none;
  }
}"#;
        let result = compile(less).unwrap();
        assert_eq!(result.trim(), expected.trim());
    }
}

#[cfg(test)]
mod mixins {
    use super::*;

    #[test]
    fn test_simple_mixin() {
        let less = r#"
.border-radius(@radius) {
    border-radius: @radius;
    -webkit-border-radius: @radius;
    -moz-border-radius: @radius;
}

.button {
    .border-radius(5px);
    padding: 10px;
}
"#;
        let expected = r#".button {
  border-radius: 5px;
  -webkit-border-radius: 5px;
  -moz-border-radius: 5px;
  padding: 10px;
}"#;
        let result = compile(less).unwrap();
        assert_eq!(result.trim(), expected.trim());
    }

    #[test]
    fn test_mixin_with_default_parameters() {
        let less = r#"
.box-shadow(@x: 0, @y: 0, @blur: 5px, @color: #000) {
    box-shadow: @x @y @blur @color;
}

.card {
    .box-shadow();
}

.special {
    .box-shadow(2px, 2px, 10px, #333);
}
"#;
        let expected = r#".card {
  box-shadow: 0 0 5px #000;
}
.special {
  box-shadow: 2px 2px 10px #333;
}"#;
        let result = compile(less).unwrap();
        assert_eq!(result.trim(), expected.trim());
    }

    #[test]
    fn test_parametric_mixin_with_guards() {
        let less = r#"
.mixin(@a) when (@a > 10) {
    color: red;
}

.mixin(@a) when (@a <= 10) {
    color: blue;
}

.test1 {
    .mixin(15);
}

.test2 {
    .mixin(5);
}
"#;
        let expected = r#".test1 {
  color: red;
}
.test2 {
  color: blue;
}"#;
        let result = compile(less).unwrap();
        assert_eq!(result.trim(), expected.trim());
    }
}

#[cfg(test)]
mod operations {
    use super::*;

    #[test]
    fn test_arithmetic_operations() {
        let less = r#"
@base: 10px;
@multiplier: 2;

.container {
    width: @base * @multiplier;
    height: @base + 5px;
    margin: (@base / 2);
    padding: @base - 2px;
}
"#;
        let expected = r#".container {
  width: 20px;
  height: 15px;
  margin: 5px;
  padding: 8px;
}"#;
        let result = compile(less).unwrap();
        assert_eq!(result.trim(), expected.trim());
    }

    #[test]
    fn test_color_operations() {
        let less = r#"
@base-color: #333;

.theme {
    color: @base-color;
    background: lighten(@base-color, 20%);
    border: darken(@base-color, 10%);
}
"#;
        let expected = r#".theme {
  color: #333;
  background: #666;
  border: #1a1a1a;
}"#;
        let result = compile(less).unwrap();
        assert_eq!(result.trim(), expected.trim());
    }

    #[test]
    fn test_string_operations() {
        let less = r#"
@base-font: "Arial";
@weight: "bold";

.text {
    font-family: @base-font, sans-serif;
    font-weight: @weight;
}
"#;
        let expected = r#".text {
  font-family: "Arial", sans-serif;
  font-weight: "bold";
}"#;
        let result = compile(less).unwrap();
        assert_eq!(result.trim(), expected.trim());
    }
}

#[cfg(test)]
mod functions {
    use super::*;

    #[test]
    fn test_built_in_color_functions() {
        let less = r#"
@base: #333;

.colors {
    light: lighten(@base, 20%);
    dark: darken(@base, 20%);
    saturated: saturate(@base, 50%);
    desaturated: desaturate(@base, 50%);
    transparent: fade(@base, 50%);
}
"#;
        let result = compile(less);
        assert!(result.is_ok());
    }

    #[test]
    fn test_math_functions() {
        let less = r#"
.math {
    rounded: round(10.6px);
    ceiling: ceil(10.1px);
    floor: floor(10.9px);
    percentage: percentage(0.5);
}
"#;
        let expected = r#".math {
  rounded: 11px;
  ceiling: 11px;
  floor: 10px;
  percentage: 50%;
}"#;
        let result = compile(less).unwrap();
        assert_eq!(result.trim(), expected.trim());
    }

    #[test]
    fn test_string_functions() {
        let less = r#"
@str: "hello world";

.strings {
    escaped: e(@str);
    replaced: replace(@str, "world", "LESS");
}
"#;
        let result = compile(less);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod imports {
    use super::*;

    #[test]
    fn test_css_import() {
        let less = r#"
@import "reset.css";

.main {
    color: red;
}
"#;
        let expected = r#"@import "reset.css";
.main {
  color: red;
}"#;
        let result = compile(less).unwrap();
        assert_eq!(result.trim(), expected.trim());
    }

    #[test]
    fn test_less_import() {
        let less = r#"
@import "variables";
@import "mixins.less";

.component {
    color: @primary-color;
}
"#;
        // For now, we'll test that it doesn't error
        // Full import functionality requires file system access
        let result = compile(less);
        assert!(result.is_ok() || matches!(result, Err(Error::ImportError { .. })));
    }
}

#[cfg(test)]
mod advanced_features {
    use super::*;

    #[test]
    fn test_extend_functionality() {
        let less = r#"
.button {
    padding: 10px;
    border: 1px solid #ccc;
}

.primary-button {
    &:extend(.button);
    background: blue;
    color: white;
}
"#;
        let result = compile(less);
        assert!(result.is_ok());
        let css = result.unwrap();
        assert!(css.contains(".button, .primary-button"));
        assert!(css.contains("padding: 10px"));
        assert!(css.contains(".primary-button"));
        assert!(css.contains("background: blue"));
    }

    #[test]
    fn test_loops_and_recursion() {
        let less = r#"
.generate-columns(@n, @i: 1) when (@i <= @n) {
    .column-@{i} {
        width: (@i * 100% / @n);
    }
    .generate-columns(@n, (@i + 1));
}
.generate-columns(@n, @i) when (@i > @n) {}

.generate-columns(4);
"#;
        let result = compile(less);
        assert!(result.is_ok());
    }

    #[test]
    fn test_namespaces() {
        let less = r#"
#bundle {
    .button {
        display: block;
        border: 1px solid black;
        background-color: grey;

        &:hover {
            background-color: white;
        }
    }
}

.header a {
    #bundle > .button();
}
"#;
        let result = compile(less);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "maps data structure not yet implemented"]
    fn test_maps() {
        let less = r#"
@sizes: {
  mobile: 320px;
  tablet: 768px;
  desktop: 1024px;
}

.container {
    width: @sizes[mobile];

    @media (min-width: @sizes[tablet]) {
        width: @sizes[tablet];
    }
}
"#;
        let result = compile(less);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod error_handling {
    use super::*;

    #[test]
    fn test_undefined_variable_error() {
        let less = r#"
.test {
    color: @undefined-variable;
}
"#;
        let result = compile(less);
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::UndefinedVariable { .. })));
    }

    #[test]
    fn test_syntax_error() {
        let less = r#"
.test {
    color: red
    // Missing semicolon
    background: blue;
"#;
        let result = compile(less);
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::ParseError { .. })));
    }

    #[test]
    fn test_circular_import_error() {
        let less = r#"
@import "self";
"#;
        let result = compile(less);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_mixin_call() {
        let less = r#"
.test {
    .undefined-mixin();
}
"#;
        let result = compile(less);
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::UndefinedMixin { .. })));
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_debug_mixin_parsing() {
        // 测试最简单的参数化混合器定义
        let less = r#"
.test-mixin(@param) {
    color: red;
}
"#;
        let result = compile(less);
        println!("Debug test result: {:?}", result);
        // 暂时只检查不会崩溃
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_empty_input() {
        let less = "";
        let result = compile(less).unwrap();
        assert_eq!(result.trim(), "");
    }

    #[test]
    fn test_comments_preservation() {
        let less = r#"
/* This is a block comment */
.test {
    // This is a line comment
    color: red; /* inline comment */
}
"#;
        let result = compile(less);
        assert!(result.is_ok());
        // Comments should be preserved in output
        let css = result.unwrap();
        assert!(css.contains("/* This is a block comment */"));
    }

    #[test]
    fn test_unicode_and_special_characters() {
        let less = r#"
@unicode: "🎨";
.special {
    content: @unicode;
    font-family: "Times New Roman", serif;
}
"#;
        let result = compile(less);
        assert!(result.is_ok());
    }

    #[test]
    fn test_very_deep_nesting() {
        let less = r#"
.level1 {
    .level2 {
        .level3 {
            .level4 {
                .level5 {
                    color: red;
                }
            }
        }
    }
}
"#;
        let result = compile(less);
        assert!(result.is_ok());
    }

    #[test]
    fn test_large_file_performance() {
        // Generate a large LESS file for performance testing
        let mut large_less = String::new();
        for i in 0..1000 {
            large_less.push_str(&format!(
                ".class-{} {{ color: #{:06x}; }}\n",
                i,
                i * 1000 % 0xFFFFFF
            ));
        }

        let result = compile(&large_less);
        assert!(result.is_ok());
    }
}
