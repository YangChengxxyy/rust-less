use rust_less::compile;

// ====== Phase A: Color Arithmetic ======

#[test]
fn test_color_add_color() {
    let less = r#"
.test {
    color: #336699 + #111111;
}
"#;
    let css = compile(less).unwrap();
    // #33+#11=0x44, #66+#11=0x77, #99+#11=0xaa -> #4477aa or short #47a
    assert!(
        css.contains("color: #4477aa") || css.contains("color: #47a"),
        "Got: {}",
        css
    );
}

#[test]
fn test_color_subtract_color() {
    let less = r#"
.test {
    color: #ff0000 - #003300;
}
"#;
    let css = compile(less).unwrap();
    // #ff-#00=0xff, #00-#33=0x00(clamped), #00-#00=0x00 -> #ff0000 or #f00
    assert!(
        css.contains("color: #ff0000") || css.contains("color: #f00"),
        "Got: {}",
        css
    );
}

#[test]
fn test_color_multiply_number() {
    let less = r#"
.test {
    color: #ff0000 * 0.5;
}
"#;
    let css = compile(less).unwrap();
    // 255 * 0.5 = 127.5 -> rounds to 128 = 0x80
    assert!(css.contains("color: #800000"), "Got: {}", css);
}

#[test]
fn test_number_multiply_color() {
    let less = r#"
.test {
    color: 0.5 * #ff0000;
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("color: #800000"), "Got: {}", css);
}

#[test]
fn test_color_divide_number() {
    let less = r#"
.test {
    color: #ff0000 / 2;
}
"#;
    let css = compile(less).unwrap();
    // 255 / 2 = 127.5 -> rounds to 128 = 0x80
    assert!(css.contains("color: #800000"), "Got: {}", css);
}

#[test]
fn test_color_add_number() {
    let less = r#"
.test {
    color: #333333 + 17;
}
"#;
    let css = compile(less).unwrap();
    // 0x33 = 51, 51 + 17 = 68 = 0x44 -> #444444 or #444
    assert!(
        css.contains("color: #444444") || css.contains("color: #444"),
        "Got: {}",
        css
    );
}

#[test]
fn test_color_arithmetic_with_variable() {
    let less = r#"
@base: #336699;
.test {
    color: @base + #111111;
}
"#;
    let css = compile(less).unwrap();
    assert!(
        css.contains("color: #4477aa") || css.contains("color: #47a"),
        "Got: {}",
        css
    );
}

#[test]
fn test_color_clamp_overflow() {
    let less = r#"
.test {
    color: #ffffff + #111111;
}
"#;
    let css = compile(less).unwrap();
    // Clamped to 255
    assert!(css.contains("color: #fff"), "Got: {}", css);
}

#[test]
fn test_color_clamp_underflow() {
    let less = r#"
.test {
    color: #000000 - #111111;
}
"#;
    let css = compile(less).unwrap();
    // Clamped to 0
    assert!(css.contains("color: #000"), "Got: {}", css);
}

// ====== Phase B: Property Interpolation ======

#[test]
fn test_property_interpolation_simple() {
    let less = r#"
@prop: color;
.test {
    @{prop}: red;
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("color: red"), "Got: {}", css);
}

#[test]
fn test_property_interpolation_compound() {
    let less = r#"
@side: left;
.box {
    border-@{side}: 1px solid #000;
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("border-left:"), "Got: {}", css);
}

// ====== Phase C.1: if() function ======

#[test]
fn test_if_function_true() {
    let less = r#"
.test {
    color: if(true, red, blue);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("color: red"), "Got: {}", css);
}

#[test]
fn test_if_function_false() {
    let less = r#"
.test {
    color: if(false, red, blue);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("color: blue"), "Got: {}", css);
}

#[test]
fn test_if_function_comparison() {
    let less = r#"
.test {
    width: if(1 > 2, 100px, 50px);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("width: 50px"), "Got: {}", css);
}

// ====== Phase C.2: range() function ======

#[test]
fn test_range_function_basic() {
    let less = r#"
@list: range(1, 4);
.test {
    value: @list;
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("value: 1 2 3 4"), "Got: {}", css);
}

#[test]
fn test_range_function_with_step() {
    let less = r#"
@list: range(10px, 30px, 10);
.test {
    value: @list;
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("value: 10px 20px 30px"), "Got: {}", css);
}

// ====== Phase C.4: convert() function ======

#[test]
fn test_convert_px_to_in() {
    let less = r#"
.test {
    width: convert(96px, in);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("width: 1in"), "Got: {}", css);
}

#[test]
fn test_convert_s_to_ms() {
    let less = r#"
.test {
    duration: convert(1s, ms);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("duration: 1000ms"), "Got: {}", css);
}

#[test]
fn test_convert_deg_to_rad() {
    let less = r#"
.test {
    angle: convert(180deg, rad);
}
"#;
    let css = compile(less).unwrap();
    // 180 * PI / 180 = PI ≈ 3.14159...
    let css_lower = css.to_lowercase();
    assert!(css_lower.contains("angle: 3.14159"), "Got: {}", css);
}

// ====== Phase C.3: when default() guard ======

#[test]
fn test_mixin_default_guard() {
    let less = r#"
.mixin(dark) {
    color: white;
}
.mixin(light) {
    color: black;
}
.mixin(@_) when (default()) {
    color: grey;
}
.test {
    .mixin(other);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("color: grey"), "Got: {}", css);
}

#[test]
fn test_mixin_default_guard_not_used() {
    let less = r#"
.mixin(dark) {
    color: white;
}
.mixin(@_) when (default()) {
    color: grey;
}
.test {
    .mixin(dark);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("color: white"), "Got: {}", css);
    assert!(
        !css.contains("color: grey"),
        "Default should not be used. Got: {}",
        css
    );
}

// ====== Phase D.1: @import (optional) ======

#[test]
fn test_import_optional_missing_file() {
    let less = r#"
@import (optional) "nonexistent-file.less";
.test {
    color: red;
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("color: red"), "Got: {}", css);
}

// ====== Phase D.2: compile_with_options source_map ======

#[test]
fn test_compile_with_options_source_map() {
    let options = rust_less::CompilerOptions {
        source_map: true,
        compress: false,
        include_paths: vec![],
    };
    let result = rust_less::compile_with_options(".test { color: red; }", options);
    assert!(result.is_ok(), "Got: {:?}", result);
}

// ====== Phase E.1: Maps ======

#[test]
fn test_map_lookup() {
    let less = r#"
@sizes: {
    small: 10px;
    medium: 20px;
    large: 30px;
};
.test {
    width: @sizes[medium];
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("width: 20px"), "Got: {}", css);
}

#[test]
fn test_map_get_function() {
    let less = r#"
@sizes: {
    small: 10px;
    medium: 20px;
    large: 30px;
};
.test {
    width: map-get(@sizes, medium);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("width: 20px"), "Got: {}", css);
}

#[test]
fn test_map_keys_function() {
    let less = r#"
@sizes: {
    small: 10px;
    medium: 20px;
    large: 30px;
};
.test {
    keys: map-keys(@sizes);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("keys: small, medium, large"), "Got: {}", css);
}

#[test]
fn test_map_values_function() {
    let less = r#"
@sizes: {
    small: 10px;
    medium: 20px;
    large: 30px;
};
.test {
    values: map-values(@sizes);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("values: 10px, 20px, 30px"), "Got: {}", css);
}

#[test]
fn test_map_merge_function() {
    let less = r#"
@base: {
    small: 10px;
    medium: 20px;
};
@override: {
    medium: 22px;
    large: 30px;
};
@merged: map-merge(@base, @override);
.test {
    small: @merged[small];
    medium: @merged[medium];
    large: @merged[large];
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("small: 10px"), "Got: {}", css);
    assert!(css.contains("medium: 22px"), "Got: {}", css);
    assert!(css.contains("large: 30px"), "Got: {}", css);
}

#[test]
fn test_map_get_nested_path_function() {
    let less = r#"
@tokens: {
    breakpoints: {
        sm: 480px;
        md: 768px;
    };
};
.test {
    width: map-get(@tokens, breakpoints, md);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("width: 768px"), "Got: {}", css);
}

#[test]
fn test_map_has_key_function() {
    let less = r#"
@tokens: {
    colors: {
        primary: #111;
    };
};
.test {
    has_primary: map-has-key(@tokens, colors, primary);
    has_secondary: map-has-key(@tokens, colors, secondary);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("has_primary: true"), "Got: {}", css);
    assert!(css.contains("has_secondary: false"), "Got: {}", css);
}

#[test]
fn test_map_remove_function() {
    let less = r#"
@tokens: {
    colors: {
        primary: #111;
        secondary: #222;
    };
};
@clean: map-remove(@tokens, colors, primary);
.test {
    has_primary: map-has-key(@clean, colors, primary);
    has_secondary: map-has-key(@clean, colors, secondary);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("has_primary: false"), "Got: {}", css);
    assert!(css.contains("has_secondary: true"), "Got: {}", css);
}

#[test]
fn test_nested_map_access_bracket_chain() {
    let less = r#"
@tokens: {
    breakpoints: {
        md: 768px;
    };
};
.test {
    width: @tokens[breakpoints][md];
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("width: 768px"), "Got: {}", css);
}

#[test]
fn test_map_set_function() {
    let less = r#"
@base: {
    config: {
        theme: light;
    };
};
@next: map-set(@base, config, density, compact);
.test {
    density: map-get(@next, config, density);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("density: compact"), "Got: {}", css);
}

#[test]
fn test_map_deep_merge_function() {
    let less = r#"
@base: {
    config: {
        theme: light;
        spacing: 8;
    };
};
@override: {
    config: {
        spacing: 10;
        radius: 4;
    };
};
@merged: map-deep-merge(@base, @override);
.test {
    theme: map-get(@merged, config, theme);
    spacing: map-get(@merged, config, spacing);
    radius: map-get(@merged, config, radius);
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("theme: light"), "Got: {}", css);
    assert!(css.contains("spacing: 10"), "Got: {}", css);
    assert!(css.contains("radius: 4"), "Got: {}", css);
}

#[test]
fn test_map_key_normalization_consistency() {
    let less = r#"
@tokens: {
    "name": alpha;
    size: 12px;
    3: 30px;
};
.test {
    by_identifier: map-get(@tokens, name);
    by_string: map-get(@tokens, "name");
    by_number: map-get(@tokens, 3);
    by_bracket: @tokens[3];
    has_number_string: map-has-key(@tokens, "3");
}
"#;
    let css = compile(less).unwrap();
    assert!(css.contains("by_identifier: alpha"), "Got: {}", css);
    assert!(css.contains("by_string: alpha"), "Got: {}", css);
    assert!(css.contains("by_number: 30px"), "Got: {}", css);
    assert!(css.contains("by_bracket: 30px"), "Got: {}", css);
    assert!(css.contains("has_number_string: true"), "Got: {}", css);
}

#[test]
fn test_map_error_non_map_argument() {
    let less = r#"
@value: 1;
.test {
    x: map-set(@value, a, 2);
}
"#;
    let err = compile(less).unwrap_err().to_string();
    assert!(err.contains("First argument must be a map"), "Got: {}", err);
}

#[test]
fn test_map_error_intermediate_path_not_map() {
    let less = r#"
@tokens: {
    a: 1;
};
.test {
    x: map-set(@tokens, a, b, 2);
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
fn test_map_error_empty_path() {
    let less = r#"
@tokens: {
    a: 1;
};
.test {
    x: map-set(@tokens, 2);
}
"#;
    let err = compile(less).unwrap_err().to_string();
    assert!(
        err.contains("Expected at least 3 arguments"),
        "Got: {}",
        err
    );
}

#[test]
fn test_map_remove_error_empty_path() {
    let less = r#"
@tokens: {
    a: 1;
};
.test {
    x: map-remove(@tokens);
}
"#;
    let err = compile(less).unwrap_err().to_string();
    assert!(
        err.contains("Expected at least 2 arguments"),
        "Got: {}",
        err
    );
}

// ====== Phase E.2: each() ======

#[test]
fn test_each_list() {
    let less = r#"
each(a, b, c, {
    .item-@{value} {
        content: @value;
    }
});
"#;
    let css = compile(less).unwrap();
    assert!(css.contains(".item-a"), "Got: {}", css);
    assert!(css.contains(".item-b"), "Got: {}", css);
    assert!(css.contains(".item-c"), "Got: {}", css);
}
