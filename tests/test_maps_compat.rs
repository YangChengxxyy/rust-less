use rust_less::compile;

#[test]
fn test_lessjs_compat_map_merge_shallow_last_wins() {
    let less = r#"
@base: {
    token: 1;
    nested: {
        a: 1;
        b: 1;
    };
};
@override: {
    token: 2;
    nested: {
        c: 3;
    };
};
@merged: map-merge(@base, @override);
.test {
    token: map-get(@merged, token);
    has_nested_a: map-has-key(@merged, nested, a);
    has_nested_c: map-has-key(@merged, nested, c);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("token: 2"), "Got: {}", css);
    assert!(css.contains("has_nested_a: false"), "Got: {}", css);
    assert!(css.contains("has_nested_c: true"), "Got: {}", css);
}

#[test]
fn test_lessjs_compat_map_deep_merge_nested_and_order() {
    let less = r#"
@base: {
    config: {
        a: 1;
        b: 1;
    };
};
@ov1: {
    config: {
        b: 2;
        c: 2;
    };
};
@ov2: {
    config: {
        c: 3;
    };
};
@merged: map-deep-merge(@base, @ov1, @ov2);
.test {
    a: map-get(@merged, config, a);
    b: map-get(@merged, config, b);
    c: map-get(@merged, config, c);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("a: 1"), "Got: {}", css);
    assert!(css.contains("b: 2"), "Got: {}", css);
    assert!(css.contains("c: 3"), "Got: {}", css);
}

#[test]
fn test_lessjs_compat_map_remove_missing_path_is_noop() {
    let less = r#"
@tokens: {
    a: 1;
    nested: {
        b: 2;
    };
};
@r1: map-remove(@tokens, missing);
@r2: map-deep-remove(@tokens, nested, missing);
.test {
    r1_a: map-get(@r1, a);
    r2_b: map-get(@r2, nested, b);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("r1_a: 1"), "Got: {}", css);
    assert!(css.contains("r2_b: 2"), "Got: {}", css);
}

#[test]
fn test_lessjs_compat_map_get_missing_key_errors() {
    let less = r#"
@tokens: {
    a: 1;
};
.test {
    x: map-get(@tokens, missing);
}
"#;
    let err = compile(less).unwrap_err().to_string();
    assert!(err.contains("not found in map"), "Got: {}", err);
}

#[test]
fn test_lessjs_compat_map_update_replace_missing_key_errors() {
    let update_less = r#"
@tokens: {
    a: 1;
};
.test {
    x: map-update(@tokens, missing, 2);
}
"#;
    let update_err = compile(update_less).unwrap_err().to_string();
    assert!(
        update_err.contains("not found in map"),
        "Got: {}",
        update_err
    );

    let replace_less = r#"
@tokens: {
    a: 1;
};
.test {
    x: map-replace(@tokens, missing, 2);
}
"#;
    let replace_err = compile(replace_less).unwrap_err().to_string();
    assert!(
        replace_err.contains("not found in map"),
        "Got: {}",
        replace_err
    );
}

#[test]
fn test_lessjs_compat_map_intermediate_path_not_map_errors() {
    let less = r#"
@tokens: {
    a: 1;
};
.test {
    x: map-deep-remove(@tokens, a, b);
}
"#;
    let err = compile(less).unwrap_err().to_string();
    assert!(
        err.contains("Intermediate key 'a' is not a map"),
        "Got: {}",
        err
    );
}

#[test]
fn test_lessjs_native_map_access_with_unquoted_key_succeeds() {
    let less = r#"
@tokens: {
    name: alpha;
    3: 30px;
};
.test {
    by_name: @tokens[name];
    by_number: @tokens[3];
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("by_name: alpha"), "Got: {}", css);
    assert!(css.contains("by_number: 30px"), "Got: {}", css);
}

#[test]
fn test_lessjs_native_map_access_identifier_does_not_match_quoted_key() {
    let less = r#"
@tokens: {
    "name": alpha;
    3: 30px;
};
.test {
    by_name: @tokens[name];
    by_number: @tokens[3];
}
"#;
    let err = compile(less).unwrap_err().to_string();
    assert!(err.contains("not found in map"), "Got: {}", err);
}
