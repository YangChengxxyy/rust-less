#[cfg(test)]
mod tests {
    use rust_less::{Compiler, Error};

    #[test]
    fn test_string_functions_with_variables() {
        let mut compiler = Compiler::new();
        let input = r#"
            @str: "hello world";
            @search: "world";
            @replace: "LESS";
            
            .test {
                escaped: e(@str);
                replaced: replace(@str, @search, @replace);
                replaced-literal: replace(@str, "world", "Rust");
            }
        "#;

        let result = compiler.compile(input);

        match result {
            Ok(css) => {
                println!("CSS Output:\n{}", css);
                assert!(css.contains("escaped: hello world"), "e(@str) failed");
                // replace() returns a string, so it should be quoted in CSS output
                assert!(
                    css.contains("replaced: \"hello LESS\""),
                    "replace(@str, @search, @replace) failed"
                );
                assert!(
                    css.contains("replaced-literal: \"hello Rust\""),
                    "replace(@str, literal, literal) failed"
                );
            }
            Err(e) => {
                panic!("Compilation failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_escape_function_more() {
        let mut compiler = Compiler::new();
        let input = r#"
            @val: 10px;
            .test {
                content: e(@val);
            }
        "#;
        let css = compiler.compile(input).unwrap();
        assert!(css.contains("content: 10px"));
    }

    #[test]
    fn test_replace_preserves_unquoted() {
        let mut compiler = Compiler::new();
        let input = r#"
            @str: hello;
            .test {
                content: replace(@str, "he", "we");
            }
        "#;
        let css = compiler.compile(input).unwrap();
        assert!(css.contains("content: wello"));
        assert!(!css.contains("content: \"wello\""));
    }

    #[test]
    fn test_replace_empty_pattern_error() {
        let mut compiler = Compiler::new();
        let input = r#"
            .test {
                content: replace("abc", "", "x");
            }
        "#;
        let err = compiler.compile(input).unwrap_err();
        assert!(matches!(err, Error::FunctionError { .. }));
        assert!(err.message().contains("pattern"));
    }
}
