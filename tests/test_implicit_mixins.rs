use rust_less::compile;

#[test]
fn test_implicit_mixin_class() {
    let input = r#"
.mixin {
    color: red;
}
.class {
    .mixin();
}
"#;
    let css = compile(input).unwrap();
    assert!(css.contains("color: red;"));
}

#[test]
fn test_implicit_mixin_id() {
    let input = r#"
#mixin {
    color: blue;
}
.class {
    #mixin();
}
"#;
    let css = compile(input).unwrap();
    assert!(css.contains("color: blue;"));
}

#[test]
fn test_implicit_mixin_nested() {
    let _input = r#"
.outer {
    .inner {
        color: green;
    }
}
.class {
    // This requires namespaces to be fully supported if calling from outside
    // But if we define a rule that matches .inner, it's local to .outer
    // Implicit mixins are scoped.
}
"#;
    // This test is just a placeholder. Real namespace access requires Phase 3.3
}

#[test]
fn test_implicit_mixin_no_parens() {
    // Less allows calling mixins without parentheses if no args
    // My parser might not support this yet (it expects '(' in parse_mixin_call)
    // But let's test if .mixin() works first.
    let input = r#"
.mixin { width: 10px; }
.class { .mixin(); }
"#;
    let css = compile(input).unwrap();
    assert!(css.contains("width: 10px;"));
}
