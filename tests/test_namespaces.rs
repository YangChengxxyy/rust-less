use rust_less::compile;

#[test]
fn test_namespace_basic() {
    let input = r#"
#bundle {
    .button {
        display: block;
        border: 1px solid black;
        background-color: grey;
        &:hover { background-color: white }
    }
}
#header a {
    color: orange;
    #bundle > .button();
}
"#;
    let css = compile(input).unwrap();
    assert!(css.contains("display: block;"));
    assert!(css.contains("border: 1px solid black;"));
    assert!(css.contains("#header a:hover"));
}

#[test]
fn test_namespace_nested() {
    let input = r#"
#ns1 {
    #ns2 {
        .mixin {
            color: red;
        }
    }
}
.class {
    #ns1 > #ns2 > .mixin();
}
"#;
    let css = compile(input).unwrap();
    assert!(css.contains("color: red;"));
}

#[test]
fn test_namespace_whitespace() {
    let input = r#"
#ns { .mixin { color: blue; } }
.class {
    #ns > .mixin();
}
"#;
    let css = compile(input).unwrap();
    assert!(css.contains("color: blue;"));
}

#[test]
fn test_namespace_no_parens() {
    let input = r#"
#ns { .mixin { width: 10px; } }
.class {
    #ns > .mixin;
}
"#;
    let css = compile(input).unwrap();
    assert!(css.contains("width: 10px;"));
}
