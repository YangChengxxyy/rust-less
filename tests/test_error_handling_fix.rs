#[cfg(test)]
mod tests {
    use rust_less::{Compiler, Error};

    #[test]
    fn test_undefined_mixin_error() {
        let mut compiler = Compiler::new();
        let input = r#"
            .test {
                .undefined-mixin();
            }
        "#;

        let result = compiler.compile(input);

        assert!(result.is_err());
        let err = result.unwrap_err();
        println!("Error: {:?}", err);

        assert!(
            matches!(err, Error::UndefinedMixin { .. }),
            "Expected UndefinedMixin error, got {:?}",
            err
        );
    }

    #[test]
    fn test_nested_undefined_mixin_error() {
        let mut compiler = Compiler::new();
        let input = r#"
            .outer {
                .inner {
                    .undefined();
                }
            }
        "#;

        let result = compiler.compile(input);

        assert!(result.is_err());
        let err = result.unwrap_err();

        assert!(
            matches!(err, Error::UndefinedMixin { .. }),
            "Expected UndefinedMixin error in nested rule, got {:?}",
            err
        );
    }
}
