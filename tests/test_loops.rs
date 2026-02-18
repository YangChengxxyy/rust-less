use rust_less::{compile, Compiler};

#[test]
fn test_loop_basic() {
    let input = r#"
.loop(@n) when (@n > 0) {
    .class-@{n} {
        width: @n * 10px;
    }
    .loop(@n - 1);
}
.loop(@n) when (@n = 0) {} // Base case

.loop(3);
"#;

    let css = compile(input).unwrap();
    assert!(css.contains(".class-3"));
    assert!(css.contains("width: 30px;"));
    assert!(css.contains(".class-2"));
    assert!(css.contains("width: 20px;"));
    assert!(css.contains(".class-1"));
    assert!(css.contains("width: 10px;"));
    // Should stop at 0
    assert!(!css.contains(".class-0"));
}

#[test]
fn test_infinite_recursion() {
    let input = r#"
.loop(@n) {
    .loop(@n + 1);
}

.loop(1);
"#;

    let result = compile(input);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(format!("{}", err).contains("Infinite recursion"));
}

#[test]
fn test_recursion_limit_config() {
    let input = r#"
.loop(@n) when (@n > 0) {
    .loop(@n - 1);
}
.loop(@n) when (@n = 0) {} // Base case
.loop(20);
"#;

    // Default limit is 100, so this should pass
    let result = Compiler::new().compile(input);
    assert!(result.is_ok());

    // Set limit to 10, should fail
    let result = Compiler::new().with_recursion_limit(10).compile(input);

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(format!("{}", err).contains("Infinite recursion"));
}
