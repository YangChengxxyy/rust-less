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

#[test]
fn test_lessjs_compat_each_over_map_native() {
    let less = r#"
@colors: {
    primary: blue;
    secondary: gray;
};
each(@colors, {
    .color-@{key} {
        color: @value;
        order: @index;
    }
});
"#;
    let css = compile(less).unwrap();
    assert!(css.contains(".color-primary"), "Got: {}", css);
    assert!(css.contains("color: blue"), "Got: {}", css);
    assert!(css.contains("order: 2"), "Got: {}", css);
}

#[test]
fn test_lessjs_compat_each_over_list_variable() {
    let less = r#"
@sizes: 10px 20px;
each(@sizes, {
    .m-@{index} {
        margin: @value;
    }
});
"#;
    let css = compile(less).unwrap();
    assert!(css.contains(".m-1"), "Got: {}", css);
    assert!(css.contains("margin: 20px"), "Got: {}", css);
}

#[test]
fn test_lessjs_compat_each_inside_rule_body() {
    let less = r#"
@colors: {
    red: #f00;
    blue: #00f;
};
.demo {
    each(@colors, {
        c-@{key}: @value;
    });
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("c-red: #f00"), "Got: {}", css);
    assert!(css.contains("c-blue: #00f"), "Got: {}", css);
}

#[test]
fn test_lessjs_compat_map_access_in_media_prelude() {
    let less = r#"
@sizes: {
    tablet: 768px;
};
.container {
    @media (min-width: @sizes[tablet]) {
        width: 100px;
    }
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("@media (min-width: 768px)"), "Got: {}", css);
}

#[test]
fn test_lessjs_compat_map_called_as_detached_ruleset() {
    let less = r#"
@lib: {
    color: red;
    margin: 0;
};
.test {
    @lib();
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("color: red"), "Got: {}", css);
    assert!(css.contains("margin: 0"), "Got: {}", css);
}

#[test]
fn test_lessjs_compat_space_and_comma_variable_values() {
    let less = r#"
@margin: 10px 20px;
@font: Arial, sans-serif;
.test {
    margin: @margin;
    font-family: @font;
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("margin: 10px 20px"), "Got: {}", css);
    assert!(
        css.contains("font-family: Arial, sans-serif"),
        "Got: {}",
        css
    );
}

#[test]
fn test_lessjs_compat_mixin_call_inside_media_keeps_selector() {
    let less = r#"
.mix() {
    letter-spacing: 1px;
}
.entry {
    @media (min-width: 800px) {
        .mix();
    }
}
"#;
    let css = compile(less).unwrap();
    assert!(
        css.contains("@media (min-width: 800px) {\n  .entry {"),
        "Got: {}",
        css
    );
    assert!(css.contains("letter-spacing: 1px"), "Got: {}", css);
}

#[test]
fn test_lessjs_native_dup_key_last_wins_bracket_and_map_get() {
    let less = r#"
@tokens: {
    color: red;
    color: blue;
};
.test {
    bracket: @tokens[color];
    via_get: map-get(@tokens, color);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("bracket: blue"), "Got: {}", css);
    assert!(css.contains("via_get: blue"), "Got: {}", css);
    assert!(!css.contains(": red"), "Got: {}", css);
}

#[test]
fn test_lessjs_native_unit_and_negative_keys() {
    let less = r#"
@metrics: {
    1px: small;
    2em: medium;
    -1: below;
};
.test {
    a: @metrics[1px];
    b: @metrics[2em];
    c: @metrics[-1];
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("a: small"), "Got: {}", css);
    assert!(css.contains("b: medium"), "Got: {}", css);
    assert!(css.contains("c: below"), "Got: {}", css);
}

#[test]
fn test_lessjs_native_map_value_lazy_evaluation_scope() {
    // less.js: map entry values are evaluated lazily at the use site, so a
    // variable defined after the map (but before the access) resolves.
    let less = r#"
@tokens: {
    accent: @semantic;
};
.test {
    @semantic: rebeccapurple;
    c: @tokens[accent];
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("c: rebeccapurple"), "Got: {}", css);

    // Variable defined after the map at top level also resolves (lazy).
    let less_after = r#"
@tokens: {
    accent: @semantic;
};
@semantic: rebeccapurple;
.test {
    c: @tokens[accent];
}
"#;
    let css = compile(less_after).unwrap();
    assert!(css.contains("c: rebeccapurple"), "Got: {}", css);

    // Truly unresolved variables still error at the access site.
    let less_undef = r#"
@tokens: {
    accent: @missing;
};
.test {
    c: @tokens[accent];
}
"#;
    assert!(compile(less_undef).is_err());
}

#[test]
fn test_lessjs_native_interpolated_map_keys() {
    let less = r#"
@key-name: primary;
@tokens: {
    @{key-name}: blue;
    suffix-@{key-name}: gray;
};
.test {
    a: @tokens[primary];
    b: @tokens[suffix-primary];
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("a: blue"), "Got: {}", css);
    assert!(css.contains("b: gray"), "Got: {}", css);

    // Composite identifier key as the FIRST entry also routes to a map.
    let less_first = r#"
@key-name: hover;
@states: {
    default-@{key-name}: 0.9;
    active-@{key-name}: 0.8;
};
.test {
    a: @states[default-hover];
    b: @states[active-hover];
}
"#;
    let css = compile(less_first).unwrap();
    assert!(css.contains("a: 0.9"), "Got: {}", css);
    assert!(css.contains("b: 0.8"), "Got: {}", css);
}

#[test]
fn test_lessjs_native_map_and_ruleset_as_property_value_error() {
    // less.js: "Rulesets cannot be evaluated on a property."
    let less = r#"
@tokens: {
    a: 1px;
};
.test {
    c: @tokens;
}
"#;
    let err = compile(less).unwrap_err().to_string();
    assert!(err.contains("cannot be used as a property value"), "Got: {}", err);

    let less_dr = r#"
@dr: {
    color: red;
};
.test {
    c: @dr;
}
"#;
    let err = compile(less_dr).unwrap_err().to_string();
    assert!(err.contains("cannot be used as a property value"), "Got: {}", err);
}

#[test]
fn test_map_extension_percent_key_supported() {
    // rust-less extension (less.js 4.5.1 rejects percentage keys at parse time).
    let less = r#"
@m: {
    50%: half;
};
.test {
    c: @m[50%];
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("c: half"), "Got: {}", css);
}

#[test]
fn test_map_extension_map_set_updates_effective_dup_key() {
    // With duplicate keys, map-set updates the effective (last) entry.
    let less = r#"
@tokens: {
    a: 1;
    a: 2;
};
.test {
    color: map-get(@tokens, a);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("color: 2"), "Got: {}", css);
}
