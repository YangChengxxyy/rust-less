
#[cfg(test)]
mod tests {
    use rust_less::Compiler;

    #[test]
    fn test_emoji_variables() {
        let mut compiler = Compiler::new();
        // Emoji unquoted might fail if lexer doesn't support it as identifier start or part
        // So we test quoted emoji first
        let input = r#"
            @emoji: "🦀";
            
            .test {
                content: @emoji;
            }
        "#;
        
        let result = compiler.compile(input);
        
        match result {
            Ok(css) => {
                println!("CSS: {}", css);
                assert!(css.contains("content: \"🦀\""));
            },
            Err(e) => {
                panic!("Emoji compilation failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_unicode_identifiers() {
        let mut compiler = Compiler::new();
        let input = r#"
            @变量: 10px;
            .类名 {
                width: @变量;
            }
        "#;
        
        let result = compiler.compile(input);
        
        match result {
            Ok(css) => {
                println!("CSS: {}", css);
                assert!(css.contains(".类名"));
                assert!(css.contains("width: 10px"));
            },
            Err(e) => {
                panic!("Unicode identifiers failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_emoji_identifier_unquoted() {
        let mut compiler = Compiler::new();
        let input = r#"
            @🚀: red;
            .rocket {
                color: @🚀;
            }
        "#;
        
        let result = compiler.compile(input);
        
        match result {
            Ok(css) => {
                println!("CSS: {}", css);
                assert!(css.contains("color: red"));
            },
            Err(e) => {
                panic!("Emoji identifier failed: {:?}", e);
            }
        }
    }
}
