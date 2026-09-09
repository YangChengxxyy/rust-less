//! LESS 4.x (less.js) lazy variable semantics contract tests.
//!
//! Model (verified against less.js 4.5.1):
//! - Variable values are stored raw and evaluated lazily in the scope chain
//!   that defines them; the evaluated result is memoized.
//! - A block registers all its variable declarations before evaluating any
//!   value, so a value may reference variables declared later in the block
//!   (also transitively).
//! - Once a block (stylesheet, rule body, expanded mixin/DR body) finishes,
//!   every registered variable is evaluated: dangling references error even
//!   when the variable is never used.
//! - Map literals and detached rulesets are deferred values: their contents
//!   are only resolved when invoked/accessed.

use rust_less::compile;

#[test]
fn test_chained_forward_references_resolve_lazily() {
    let less = r#"
@a: @b;
@b: @c;
@c: red;
.test {
    color: @a;
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("color: red"), "Got: {}", css);
}

#[test]
fn test_variable_value_resolves_in_definition_scope() {
    // @a is defined at root; @b declared inside .test is not visible to it.
    let less = r#"
@a: @b;
.test {
    @b: inner;
    color: @a;
}
"#;
    let err = compile(less).unwrap_err().to_string();
    assert!(err.contains("b"), "Got: {}", err);
}

#[test]
fn test_last_declaration_in_scope_wins() {
    let less = r#"
.test {
    color: @a;
    @a: 2;
}
@a: 1;
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("color: 2"), "Got: {}", css);
}

#[test]
fn test_dangling_variable_errors_at_block_end_root() {
    let less = r#"
@unused: @missing;
.test {
    color: red;
}
"#;
    let err = compile(less).unwrap_err().to_string();
    assert!(err.contains("missing"), "Got: {}", err);
}

#[test]
fn test_dangling_variable_errors_at_block_end_rule() {
    let less = r#"
.test {
    @bad: @missing;
    color: red;
}
"#;
    let err = compile(less).unwrap_err().to_string();
    assert!(err.contains("missing"), "Got: {}", err);
}

#[test]
fn test_dangling_variable_mixin_body_deferred_until_call() {
    // Deferred: uncalled mixin bodies are not evaluated.
    let less = r#"
.mixin-body() {
    @bad: @missing;
    color: red;
}
.test {
    x: 1;
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("x: 1"), "Got: {}", css);

    // Expanded: dangling references error on expansion.
    let less_called = r#"
.mixin-body() {
    @bad: @missing;
    color: red;
}
.test {
    .mixin-body();
}
"#;
    let err = compile(less_called).unwrap_err().to_string();
    assert!(err.contains("missing"), "Got: {}", err);
}

#[test]
fn test_dangling_variable_detached_ruleset_deferred_until_call() {
    let less = r#"
@dr: {
    @bad: @missing;
    color: red;
};
.test {
    x: 1;
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("x: 1"), "Got: {}", css);

    let less_called = r#"
@dr: {
    @bad: @missing;
    color: red;
};
.test {
    @dr();
}
"#;
    let err = compile(less_called).unwrap_err().to_string();
    assert!(err.contains("missing"), "Got: {}", err);
}

#[test]
fn test_map_entries_stay_lazy_with_dangling_value_until_access() {
    // Unused map with a dangling entry: no error (deferred like rulesets).
    let less = r#"
@tokens: {
    broken: @missing;
};
.test {
    x: 1;
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("x: 1"), "Got: {}", css);

    // Accessing the entry errors.
    let less_access = r#"
@tokens: {
    broken: @missing;
};
.test {
    x: @tokens[broken];
}
"#;
    assert!(compile(less_access).is_err());
}

#[test]
fn test_recursive_variable_definition_errors() {
    for (label, less) in [
        (
            "self",
            "@a: @a; .test { color: @a; }",
        ),
        (
            "mutual",
            "@a: @b; @b: @a; .test { color: @a; }",
        ),
        (
            "shadow-self",
            "@x: 1; .test { @x: @x; color: @x; }",
        ),
    ] {
        let err = compile(less).unwrap_err().to_string();
        assert!(
            err.contains("Recursive variable definition"),
            "{} case, got: {}",
            label,
            err
        );
    }
}

#[test]
fn test_map_alias_keeps_entries_lazy_at_use_site() {
    // Aliasing a map must not freeze entry evaluation at the alias's scope.
    let less = r#"
@source: {
    tone: @computed;
};
@alias: @source;
.test {
    @computed: rebeccapurple;
    color: @alias[tone];
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("color: rebeccapurple"), "Got: {}", css);
}

#[test]
fn test_variable_alias_chain() {
    let less = r#"
@x: @y;
@y: 10px;
.test {
    width: @x;
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("width: 10px"), "Got: {}", css);
}

#[test]
fn test_detached_ruleset_and_map_invocation_through_alias() {
    // less.js: @alias() works when @alias points at a detached ruleset / map.
    let less_dr = r#"
@dr: {
    color: @y;
};
@alias: @dr;
.test {
    @y: inner;
    @alias();
}
"#;
    let css = compile(less_dr).unwrap();
    assert!(css.contains("color: inner"), "Got: {}", css);

    let less_map = r#"
@tokens: {
    size: 10px;
};
@alias: @tokens;
.test {
    @alias();
}
"#;
    let css = compile(less_map).unwrap();
    assert!(css.contains("size: 10px"), "Got: {}", css);
}
