//! LESS 内置函数
//!
//! 此模块提供 LESS 内置函数的实现，用于颜色操作、
//! 数学运算、字符串操作和其他实用功能。

use crate::ast::{Expression, Position};
use crate::error::{Error, Result};

/// Type alias for built-in function signature
type BuiltinFn = Box<dyn Fn(&[Expression], &Position) -> Result<Expression>>;

/// LESS 内置函数注册表
pub struct FunctionRegistry {
    functions: std::collections::HashMap<String, BuiltinFn>,
}

impl FunctionRegistry {
    /// 创建包含所有内置函数的新函数注册表
    pub fn new() -> Self {
        let mut registry = Self {
            functions: std::collections::HashMap::new(),
        };
        registry.register_builtin_functions();
        registry
    }

    /// 注册所有内置函数
    fn register_builtin_functions(&mut self) {
        // 数学函数
        self.register("round", Box::new(round_function));
        self.register("ceil", Box::new(ceil_function));
        self.register("floor", Box::new(floor_function));
        self.register("abs", Box::new(abs_function));
        self.register("min", Box::new(min_function));
        self.register("max", Box::new(max_function));
        self.register("percentage", Box::new(percentage_function));

        // 颜色函数
        self.register("lighten", Box::new(lighten_function));
        self.register("darken", Box::new(darken_function));
        self.register("saturate", Box::new(saturate_function));
        self.register("desaturate", Box::new(desaturate_function));
        self.register("fade", Box::new(fade_function));
        self.register("fadeout", Box::new(fadeout_function));
        self.register("fadein", Box::new(fadein_function));
        self.register("spin", Box::new(spin_function));
        self.register("mix", Box::new(mix_function));
        self.register("rgb", Box::new(rgb_function));
        self.register("rgba", Box::new(rgba_function));
        self.register("hsl", Box::new(hsl_function));
        self.register("hsla", Box::new(hsla_function));

        // String functions
        self.register("e", Box::new(escape_function));
        self.register("escape", Box::new(escape_function));
        self.register("replace", Box::new(replace_function));
        self.register("uppercase", Box::new(uppercase_function));
        self.register("lowercase", Box::new(lowercase_function));
        self.register("length", Box::new(length_function));
        self.register("extract", Box::new(extract_function));

        // Type check functions
        self.register("isnumber", Box::new(isnumber_function));
        self.register("iscolor", Box::new(iscolor_function));
        self.register("isstring", Box::new(isstring_function));
        self.register("iskeyword", Box::new(iskeyword_function));
        self.register("isurl", Box::new(isurl_function));
        self.register("ispixel", Box::new(ispixel_function));
        self.register("isem", Box::new(isem_function));
        self.register("ispercentage", Box::new(ispercentage_function));
        self.register("unit", Box::new(unit_function));
        self.register("get-unit", Box::new(get_unit_function));

        // Advanced math functions
        self.register("sqrt", Box::new(sqrt_function));
        self.register("sin", Box::new(sin_function));
        self.register("cos", Box::new(cos_function));
        self.register("tan", Box::new(tan_function));
        self.register("asin", Box::new(asin_function));
        self.register("acos", Box::new(acos_function));
        self.register("atan", Box::new(atan_function));
        self.register("pow", Box::new(pow_function));
        self.register("pi", Box::new(pi_function));
        self.register("mod", Box::new(mod_function));

        // URL function
        self.register("url", Box::new(url_function));

        // Transform functions
        self.register("scale", Box::new(scale_function));
        self.register("translateX", Box::new(translate_x_function));
        self.register("translateY", Box::new(translate_y_function));
        self.register("rotate", Box::new(rotate_function));

        // Conditional functions
        self.register("if", Box::new(if_function));

        // List functions
        self.register("range", Box::new(range_function));
        self.register("map-get", Box::new(map_get_function));
        self.register("map-has-key", Box::new(map_has_key_function));
        self.register("map-keys", Box::new(map_keys_function));
        self.register("map-values", Box::new(map_values_function));
        self.register("map-merge", Box::new(map_merge_function));
        self.register("map-deep-merge", Box::new(map_deep_merge_function));
        self.register("map-set", Box::new(map_set_function));
        self.register("map-update", Box::new(map_update_function));
        self.register("map-replace", Box::new(map_replace_function));
        self.register("map-remove", Box::new(map_remove_function));
        self.register("map-deep-remove", Box::new(map_deep_remove_function));

        // Unit conversion
        self.register("convert", Box::new(convert_function));

        // Color channel access functions
        self.register("red", Box::new(red_function));
        self.register("green", Box::new(green_function));
        self.register("blue", Box::new(blue_function));
        self.register("alpha", Box::new(alpha_function));
        self.register("hue", Box::new(hue_function));
        self.register("saturation", Box::new(saturation_function));
        self.register("lightness", Box::new(lightness_function));
        self.register("luma", Box::new(luma_function));
        self.register("luminance", Box::new(luma_function)); // alias
        self.register("argb", Box::new(argb_function));

        // Color blending functions
        self.register("multiply", Box::new(multiply_function));
        self.register("screen", Box::new(screen_function));
        self.register("overlay", Box::new(overlay_function));
        self.register("softlight", Box::new(softlight_function));
        self.register("hardlight", Box::new(hardlight_function));
        self.register("difference", Box::new(difference_function));
        self.register("exclusion", Box::new(exclusion_function));
        self.register("average", Box::new(average_function));
        self.register("negation", Box::new(negation_function));

        // Convenience color functions
        self.register("tint", Box::new(tint_function));
        self.register("shade", Box::new(shade_function));
        self.register("contrast", Box::new(contrast_function));

        // default() placeholder for mixin guard matching
        self.register("default", Box::new(default_function));
    }

    /// Register a custom function
    pub fn register<F>(&mut self, name: &str, func: Box<F>)
    where
        F: Fn(&[Expression], &Position) -> Result<Expression> + 'static,
    {
        self.functions.insert(name.to_string(), func);
    }

    /// Call a function by name
    pub fn call(&self, name: &str, args: &[Expression], position: &Position) -> Result<Expression> {
        if let Some(func) = self.functions.get(name) {
            func(args, position)
        } else {
            Err(Error::undefined_function(
                name,
                position.line,
                position.column,
            ))
        }
    }

    /// Check if a function exists
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Math functions

fn round_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 1 {
        return Err(Error::function_error(
            "round",
            "Expected 1 argument",
            position.line,
            position.column,
        ));
    }

    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::Number {
            value: value.round(),
            unit: unit.clone(),
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "round",
            "Expected number argument",
            position.line,
            position.column,
        )),
    }
}

fn ceil_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 1 {
        return Err(Error::function_error(
            "ceil",
            "Expected 1 argument",
            position.line,
            position.column,
        ));
    }

    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::Number {
            value: value.ceil(),
            unit: unit.clone(),
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "ceil",
            "Expected number argument",
            position.line,
            position.column,
        )),
    }
}

fn floor_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 1 {
        return Err(Error::function_error(
            "floor",
            "Expected 1 argument",
            position.line,
            position.column,
        ));
    }

    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::Number {
            value: value.floor(),
            unit: unit.clone(),
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "floor",
            "Expected number argument",
            position.line,
            position.column,
        )),
    }
}

fn abs_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 1 {
        return Err(Error::function_error(
            "abs",
            "Expected 1 argument",
            position.line,
            position.column,
        ));
    }

    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::Number {
            value: value.abs(),
            unit: unit.clone(),
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "abs",
            "Expected number argument",
            position.line,
            position.column,
        )),
    }
}

fn min_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.is_empty() {
        return Err(Error::function_error(
            "min",
            "Expected at least 1 argument",
            position.line,
            position.column,
        ));
    }

    let mut min_val = f64::INFINITY;
    let mut result_unit = None;

    for arg in args {
        match arg {
            Expression::Number { value, unit, .. } => {
                if *value < min_val {
                    min_val = *value;
                    result_unit = unit.clone();
                }
            }
            _ => {
                return Err(Error::function_error(
                    "min",
                    "All arguments must be numbers",
                    position.line,
                    position.column,
                ))
            }
        }
    }

    Ok(Expression::Number {
        value: min_val,
        unit: result_unit,
        position: position.clone(),
    })
}

fn max_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.is_empty() {
        return Err(Error::function_error(
            "max",
            "Expected at least 1 argument",
            position.line,
            position.column,
        ));
    }

    let mut max_val = f64::NEG_INFINITY;
    let mut result_unit = None;

    for arg in args {
        match arg {
            Expression::Number { value, unit, .. } => {
                if *value > max_val {
                    max_val = *value;
                    result_unit = unit.clone();
                }
            }
            _ => {
                return Err(Error::function_error(
                    "max",
                    "All arguments must be numbers",
                    position.line,
                    position.column,
                ))
            }
        }
    }

    Ok(Expression::Number {
        value: max_val,
        unit: result_unit,
        position: position.clone(),
    })
}

fn percentage_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 1 {
        return Err(Error::function_error(
            "percentage",
            "Expected 1 argument",
            position.line,
            position.column,
        ));
    }

    match &args[0] {
        Expression::Number { value, .. } => {
            Ok(Expression::Percentage(value * 100.0, position.clone()))
        }
        _ => Err(Error::function_error(
            "percentage",
            "Expected number argument",
            position.line,
            position.column,
        )),
    }
}

// Color functions (placeholder implementations)

fn lighten_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 2 {
        return Err(Error::function_error(
            "lighten",
            "Expected 2 arguments",
            position.line,
            position.column,
        ));
    }

    // Extract and convert color and percentage
    let (red, green, blue, alpha) = expression_to_color(&args[0], position)?;
    let percentage_value = expression_to_percentage(&args[1], position)?;

    // Convert RGB to HSL
    let (h, s, mut l) = rgb_to_hsl(red, green, blue);

    // Increase lightness by percentage
    l = (l + percentage_value / 100.0).min(1.0);

    // Convert back to RGB
    let (r, g, b) = hsl_to_rgb(h, s, l);

    Ok(Expression::Color {
        red: r,
        green: g,
        blue: b,
        alpha,
        original: None,
        position: position.clone(),
    })
}

fn darken_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 2 {
        return Err(Error::function_error(
            "darken",
            "Expected 2 arguments",
            position.line,
            position.column,
        ));
    }

    // Extract and convert color and percentage
    let (red, green, blue, alpha) = expression_to_color(&args[0], position)?;
    let percentage_value = expression_to_percentage(&args[1], position)?;

    // Convert RGB to HSL
    let (h, s, mut l) = rgb_to_hsl(red, green, blue);

    // Decrease lightness by percentage
    l = (l - percentage_value / 100.0).max(0.0);

    // Convert back to RGB
    let (r, g, b) = hsl_to_rgb(h, s, l);

    Ok(Expression::Color {
        red: r,
        green: g,
        blue: b,
        alpha,
        original: None,
        position: position.clone(),
    })
}

fn saturate_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 2 {
        return Err(Error::function_error(
            "saturate",
            "Expected 2 arguments",
            position.line,
            position.column,
        ));
    }

    // Extract and convert color and percentage
    let (red, green, blue, alpha) = expression_to_color(&args[0], position)?;
    let percentage_value = expression_to_percentage(&args[1], position)?;

    // Convert RGB to HSL
    let (h, mut s, l) = rgb_to_hsl(red, green, blue);

    // Increase saturation by percentage
    s = (s + percentage_value / 100.0).min(1.0);

    // Convert back to RGB
    let (r, g, b) = hsl_to_rgb(h, s, l);

    Ok(Expression::Color {
        red: r,
        green: g,
        blue: b,
        alpha,
        original: None,
        position: position.clone(),
    })
}

fn desaturate_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 2 {
        return Err(Error::function_error(
            "desaturate",
            "Expected 2 arguments",
            position.line,
            position.column,
        ));
    }

    // Extract and convert color and percentage
    let (red, green, blue, alpha) = expression_to_color(&args[0], position)?;
    let percentage_value = expression_to_percentage(&args[1], position)?;

    // Convert RGB to HSL
    let (h, mut s, l) = rgb_to_hsl(red, green, blue);

    // Decrease saturation by percentage
    s = (s - percentage_value / 100.0).max(0.0);

    // Convert back to RGB
    let (r, g, b) = hsl_to_rgb(h, s, l);

    Ok(Expression::Color {
        red: r,
        green: g,
        blue: b,
        alpha,
        original: None,
        position: position.clone(),
    })
}

fn fade_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 2 {
        return Err(Error::function_error(
            "fade",
            "Expected 2 arguments",
            position.line,
            position.column,
        ));
    }

    let (red, green, blue, _) = expression_to_color(&args[0], position)?;
    let alpha = expression_to_percentage(&args[1], position)? / 100.0;

    Ok(Expression::Color {
        red,
        green,
        blue,
        alpha: alpha.clamp(0.0, 1.0),
        original: None,
        position: position.clone(),
    })
}

fn fadeout_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 2 {
        return Err(Error::function_error(
            "fadeout",
            "Expected 2 arguments",
            position.line,
            position.column,
        ));
    }

    let (red, green, blue, alpha) = expression_to_color(&args[0], position)?;
    let amount = expression_to_percentage(&args[1], position)? / 100.0;

    Ok(Expression::Color {
        red,
        green,
        blue,
        alpha: (alpha - amount).max(0.0),
        original: None,
        position: position.clone(),
    })
}

fn fadein_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 2 {
        return Err(Error::function_error(
            "fadein",
            "Expected 2 arguments",
            position.line,
            position.column,
        ));
    }

    let (red, green, blue, alpha) = expression_to_color(&args[0], position)?;
    let amount = expression_to_percentage(&args[1], position)? / 100.0;

    Ok(Expression::Color {
        red,
        green,
        blue,
        alpha: (alpha + amount).min(1.0),
        original: None,
        position: position.clone(),
    })
}

fn spin_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 2 {
        return Err(Error::function_error(
            "spin",
            "Expected 2 arguments",
            position.line,
            position.column,
        ));
    }

    let (red, green, blue, alpha) = expression_to_color(&args[0], position)?;

    let angle = match &args[1] {
        Expression::Number { value, .. } => *value,
        _ => {
            return Err(Error::function_error(
                "spin",
                "Angle must be a number",
                position.line,
                position.column,
            ))
        }
    };

    // Convert RGB to HSL
    let (mut h, s, l) = rgb_to_hsl(red, green, blue);

    // Rotate hue
    // Hue is 0.0-1.0 in our HSL implementation, but input is degrees
    h = (h * 360.0 + angle) % 360.0;
    if h < 0.0 {
        h += 360.0;
    }
    h /= 360.0;

    // Convert back to RGB
    let (r, g, b) = hsl_to_rgb(h, s, l);

    Ok(Expression::Color {
        red: r,
        green: g,
        blue: b,
        alpha,
        original: None,
        position: position.clone(),
    })
}

fn mix_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() < 2 || args.len() > 3 {
        return Err(Error::function_error(
            "mix",
            "Expected 2 or 3 arguments",
            position.line,
            position.column,
        ));
    }

    let (r1, g1, b1, a1) = expression_to_color(&args[0], position)?;
    let (r2, g2, b2, a2) = expression_to_color(&args[1], position)?;

    let weight = if args.len() == 3 {
        expression_to_percentage(&args[2], position)? / 100.0
    } else {
        0.5
    };

    // LESS mix algorithm
    let p = weight;
    let w = p * 2.0 - 1.0;
    let a = a1 - a2;

    let w1 = if (w * a - -1.0).abs() < f64::EPSILON {
        w
    } else {
        (w + a) / (1.0 + w * a)
    };

    let w1 = (w1 + 1.0) / 2.0;
    let w2 = 1.0 - w1;

    let r = (r1 as f64 * w1 + r2 as f64 * w2).round() as u8;
    let g = (g1 as f64 * w1 + g2 as f64 * w2).round() as u8;
    let b = (b1 as f64 * w1 + b2 as f64 * w2).round() as u8;

    let alpha = a1 * p + a2 * (1.0 - p);

    Ok(Expression::Color {
        red: r,
        green: g,
        blue: b,
        alpha,
        original: None,
        position: position.clone(),
    })
}

fn rgb_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 3 {
        return Err(Error::function_error(
            "rgb",
            "Expected 3 arguments",
            position.line,
            position.column,
        ));
    }

    // Extract RGB values with clamping
    let mut rgb_values = Vec::new();
    for arg in args {
        match arg {
            Expression::Number { value, .. } => {
                rgb_values.push(value.clamp(0.0, 255.0) as u8);
            }
            _ => {
                return Err(Error::function_error(
                    "rgb",
                    "All arguments must be numbers",
                    position.line,
                    position.column,
                ))
            }
        }
    }

    Ok(Expression::Color {
        red: rgb_values[0],
        green: rgb_values[1],
        blue: rgb_values[2],
        alpha: 1.0,
        original: None,
        position: position.clone(),
    })
}

fn rgba_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 4 {
        return Err(Error::function_error(
            "rgba",
            "Expected 4 arguments",
            position.line,
            position.column,
        ));
    }

    // Extract RGBA values with clamping
    let mut rgb_values = Vec::new();
    let mut alpha = 1.0;

    for (i, arg) in args.iter().enumerate() {
        match arg {
            Expression::Number { value, .. } => {
                if i < 3 {
                    rgb_values.push(value.clamp(0.0, 255.0) as u8);
                } else {
                    alpha = *value;
                }
            }
            _ => {
                return Err(Error::function_error(
                    "rgba",
                    "All arguments must be numbers",
                    position.line,
                    position.column,
                ))
            }
        }
    }

    Ok(Expression::Color {
        red: rgb_values[0],
        green: rgb_values[1],
        blue: rgb_values[2],
        alpha,
        original: None,
        position: position.clone(),
    })
}

fn hsl_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 3 {
        return Err(Error::function_error(
            "hsl",
            "Expected 3 arguments",
            position.line,
            position.column,
        ));
    }

    let h = match &args[0] {
        Expression::Number { value, .. } => (value % 360.0) / 360.0,
        _ => {
            return Err(Error::function_error(
                "hsl",
                "Hue must be a number",
                position.line,
                position.column,
            ))
        }
    };

    let s = expression_to_percentage(&args[1], position)? / 100.0;
    let l = expression_to_percentage(&args[2], position)? / 100.0;

    let (red, green, blue) = hsl_to_rgb(h, s, l);

    Ok(Expression::Color {
        red,
        green,
        blue,
        alpha: 1.0,
        original: None,
        position: position.clone(),
    })
}

fn hsla_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 4 {
        return Err(Error::function_error(
            "hsla",
            "Expected 4 arguments",
            position.line,
            position.column,
        ));
    }

    let h = match &args[0] {
        Expression::Number { value, .. } => (value % 360.0) / 360.0,
        _ => {
            return Err(Error::function_error(
                "hsla",
                "Hue must be a number",
                position.line,
                position.column,
            ))
        }
    };

    let s = expression_to_percentage(&args[1], position)? / 100.0;
    let l = expression_to_percentage(&args[2], position)? / 100.0;

    let alpha = match &args[3] {
        Expression::Number { value, .. } => *value,
        Expression::Percentage(value, _) => value / 100.0,
        _ => {
            return Err(Error::function_error(
                "hsla",
                "Alpha must be a number",
                position.line,
                position.column,
            ))
        }
    };

    let (red, green, blue) = hsl_to_rgb(h, s, l);

    Ok(Expression::Color {
        red,
        green,
        blue,
        alpha,
        original: None,
        position: position.clone(),
    })
}

// String functions

fn escape_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("escape", args, 1, position)?;
    let value = string_value(&args[0]).value;
    Ok(Expression::Escaped(value, position.clone()))
}

fn replace_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() < 3 || args.len() > 4 {
        return Err(Error::function_error(
            "replace",
            "Expected 3 or 4 arguments",
            position.line,
            position.column,
        ));
    }

    let input = string_value(&args[0]);
    let pattern_str = string_value(&args[1]).value;
    let replacement = string_value(&args[2]).value;
    let flags = if args.len() == 4 {
        string_value(&args[3]).value
    } else {
        String::new()
    };

    if pattern_str.is_empty() {
        return Err(Error::function_error(
            "replace",
            "Argument 2 (pattern) must not be empty",
            position.line,
            position.column,
        ));
    }

    // Attempt to compile pattern as Regex
    // Note: LESS regex syntax is JS-like, Rust is similar but might have differences.
    // We construct the regex with flags if needed.
    // Rust regex doesn't support global flag in the pattern string itself usually,
    // but the `regex` crate `replace_all` implies global.
    // If 'g' is NOT in flags, we should use `replace` (replace first).
    // wait, regex::Regex::replace replaces first (left-most-first). replace_all replaces all.

    let mut regex_builder = regex::RegexBuilder::new(&pattern_str);

    // Handle flags
    if flags.contains('i') {
        regex_builder.case_insensitive(true);
    }
    // 'g' is handled by which method we call (replace vs replace_all)
    // 'm' for multiline?
    if flags.contains('m') {
        regex_builder.multi_line(true);
    }

    let re = regex_builder.build().map_err(|e| {
        Error::function_error(
            "replace",
            format!("Invalid regular expression: {}", e),
            position.line,
            position.column,
        )
    })?;

    let result = if flags.contains('g') {
        re.replace_all(&input.value, replacement.as_str())
            .to_string()
    } else {
        // If no 'g', LESS replace typically replaces all?
        // No, LESS documentation says: "By default ... it replaces only the first occurrence. To replace all ... use 'g' flag".
        // Rust's replace() replaces first? regex::Regex::replace "Replaces the leftmost-first match".
        // String::replace replaces ALL.
        re.replace(&input.value, replacement.as_str()).to_string()
    };

    if input.quoted {
        Ok(Expression::string(result, position.clone()))
    } else {
        Ok(Expression::identifier(result, position.clone()))
    }
}

/// URL function - creates a URL expression
fn url_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 1 {
        return Err(Error::function_error(
            "url",
            "function expects exactly 1 argument",
            position.line,
            position.column,
        ));
    }

    let url_value = match &args[0] {
        Expression::String { value, .. } => format!("\"{}\"", value),
        Expression::Interpolation(_, _) => {
            // For interpolation, we need to return it as-is and let the compiler handle it
            return Ok(Expression::Url(format!("{:?}", args[0]), position.clone()));
        }
        _ => {
            return Err(Error::function_error(
                "url",
                "function argument must be a string",
                position.line,
                position.column,
            ));
        }
    };

    Ok(Expression::Url(url_value, position.clone()))
}

struct StringValue {
    value: String,
    quoted: bool,
}

fn ensure_arg_count(
    function: &str,
    args: &[Expression],
    expected: usize,
    position: &Position,
) -> Result<()> {
    if args.len() != expected {
        return Err(Error::function_error(
            function,
            format!(
                "Expected {} argument{}, got {}",
                expected,
                if expected == 1 { "" } else { "s" },
                args.len()
            ),
            position.line,
            position.column,
        ));
    }
    Ok(())
}

fn string_value(expr: &Expression) -> StringValue {
    match expr {
        Expression::String { value, quoted, .. } => StringValue {
            value: value.clone(),
            quoted: *quoted,
        },
        Expression::Escaped(value, _) | Expression::Anonymous(value, _) => StringValue {
            value: value.clone(),
            quoted: false,
        },
        _ => StringValue {
            value: expr.to_css(),
            quoted: false,
        },
    }
}

fn map_key_string(expr: &Expression) -> String {
    match expr {
        Expression::String { value, .. } => value.clone(),
        Expression::Number { value, unit, .. } => match unit {
            Some(unit) => format!("{}{}", value, unit),
            None => value.to_string(),
        },
        Expression::Percentage(value, _) => format!("{}%", value),
        Expression::Parenthesized(inner, _) => map_key_string(inner),
        _ => expr.to_css(),
    }
}

fn unquote_map_key(key: &str) -> Option<&str> {
    if key.len() >= 2 && key.starts_with('"') && key.ends_with('"') {
        Some(&key[1..key.len() - 1])
    } else {
        None
    }
}

fn canonical_map_key(key: &str) -> &str {
    unquote_map_key(key).unwrap_or(key)
}

fn map_keys_equal(left: &str, right: &str) -> bool {
    canonical_map_key(left) == canonical_map_key(right)
}

fn map_path_keys(args: &[Expression], start: usize) -> Vec<String> {
    args[start..].iter().map(map_key_string).collect()
}

fn map_lookup_path<'a>(
    entries: &'a [(String, Expression, Position)],
    path: &[String],
) -> std::result::Result<Option<&'a Expression>, String> {
    let mut current_entries = entries;
    for (index, key) in path.iter().enumerate() {
        let value = if let Some((_, value, _)) = current_entries
            .iter()
            .find(|(k, _, _)| map_keys_equal(k, key))
        {
            value
        } else {
            return Ok(None);
        };

        if index == path.len() - 1 {
            return Ok(Some(value));
        }

        if let Expression::MapLiteral {
            entries: nested_entries,
            ..
        } = value
        {
            current_entries = nested_entries;
        } else {
            return Err(format!("Intermediate key '{}' is not a map", key));
        }
    }

    Ok(None)
}

enum RemoveMapPathResult {
    Removed,
    Missing,
    IntermediateNotMap(String),
}

fn remove_map_path(
    entries: &mut Vec<(String, Expression, Position)>,
    path: &[String],
) -> RemoveMapPathResult {
    if path.is_empty() {
        return RemoveMapPathResult::Missing;
    }

    if path.len() == 1 {
        if let Some(index) = entries
            .iter()
            .position(|(k, _, _)| map_keys_equal(k, &path[0]))
        {
            entries.remove(index);
            return RemoveMapPathResult::Removed;
        }
        return RemoveMapPathResult::Missing;
    }

    if let Some((
        _,
        Expression::MapLiteral {
            entries: nested_entries,
            ..
        },
        _,
    )) = entries
        .iter_mut()
        .find(|(k, _, _)| map_keys_equal(k, &path[0]))
    {
        return remove_map_path(nested_entries, &path[1..]);
    }

    if entries.iter().any(|(k, _, _)| map_keys_equal(k, &path[0])) {
        return RemoveMapPathResult::IntermediateNotMap(path[0].clone());
    }

    RemoveMapPathResult::Missing
}

fn deep_remove_map_path(
    entries: &mut Vec<(String, Expression, Position)>,
    path: &[String],
) -> RemoveMapPathResult {
    if path.is_empty() {
        return RemoveMapPathResult::Missing;
    }

    if path.len() == 1 {
        if let Some(index) = entries
            .iter()
            .position(|(k, _, _)| map_keys_equal(k, &path[0]))
        {
            entries.remove(index);
            return RemoveMapPathResult::Removed;
        }
        return RemoveMapPathResult::Missing;
    }

    let Some(index) = entries
        .iter()
        .position(|(k, _, _)| map_keys_equal(k, &path[0]))
    else {
        return RemoveMapPathResult::Missing;
    };

    let mut should_prune_parent = false;
    let result = match &mut entries[index].1 {
        Expression::MapLiteral {
            entries: nested_entries,
            ..
        } => {
            let result = deep_remove_map_path(nested_entries, &path[1..]);
            if matches!(result, RemoveMapPathResult::Removed) && nested_entries.is_empty() {
                should_prune_parent = true;
            }
            result
        }
        _ => RemoveMapPathResult::IntermediateNotMap(path[0].clone()),
    };

    if should_prune_parent {
        entries.remove(index);
    }

    result
}

fn set_map_path(
    entries: &mut Vec<(String, Expression, Position)>,
    path: &[String],
    value: &Expression,
    position: &Position,
) -> std::result::Result<(), String> {
    if path.is_empty() {
        return Err("Key path cannot be empty".to_string());
    }

    if path.len() == 1 {
        if let Some((_, existing, _)) = entries
            .iter_mut()
            .find(|(k, _, _)| map_keys_equal(k, &path[0]))
        {
            *existing = value.clone();
        } else {
            entries.push((path[0].clone(), value.clone(), position.clone()));
        }
        return Ok(());
    }

    if let Some((_, existing, _)) = entries
        .iter_mut()
        .find(|(k, _, _)| map_keys_equal(k, &path[0]))
    {
        if let Expression::MapLiteral {
            entries: nested_entries,
            ..
        } = existing
        {
            return set_map_path(nested_entries, &path[1..], value, position);
        }

        return Err(format!("Intermediate key '{}' is not a map", path[0]));
    }

    entries.push((
        path[0].clone(),
        Expression::MapLiteral {
            entries: Vec::new(),
            position: position.clone(),
        },
        position.clone(),
    ));

    if let Some((
        _,
        Expression::MapLiteral {
            entries: nested_entries,
            ..
        },
        _,
    )) = entries.last_mut()
    {
        set_map_path(nested_entries, &path[1..], value, position)
    } else {
        Err("Failed to create nested map path".to_string())
    }
}

fn update_map_path(
    entries: &mut [(String, Expression, Position)],
    path: &[String],
    value: &Expression,
) -> std::result::Result<bool, String> {
    if path.is_empty() {
        return Err("Key path cannot be empty".to_string());
    }

    if path.len() == 1 {
        if let Some((_, existing, _)) = entries
            .iter_mut()
            .find(|(k, _, _)| map_keys_equal(k, &path[0]))
        {
            *existing = value.clone();
            return Ok(true);
        }
        return Ok(false);
    }

    if let Some((_, existing, _)) = entries
        .iter_mut()
        .find(|(k, _, _)| map_keys_equal(k, &path[0]))
    {
        if let Expression::MapLiteral {
            entries: nested_entries,
            ..
        } = existing
        {
            return update_map_path(nested_entries, &path[1..], value);
        }
        return Err(format!("Intermediate key '{}' is not a map", path[0]));
    }

    Ok(false)
}

fn deep_merge_entries(
    target: &mut Vec<(String, Expression, Position)>,
    source: &[(String, Expression, Position)],
) {
    for (key, value, key_position) in source {
        if let Some((_, existing, _)) = target.iter_mut().find(|(k, _, _)| map_keys_equal(k, key)) {
            if let Expression::MapLiteral {
                entries: target_nested,
                ..
            } = existing
            {
                if let Expression::MapLiteral {
                    entries: source_nested,
                    ..
                } = value
                {
                    deep_merge_entries(target_nested, source_nested);
                    continue;
                }
            }

            *existing = value.clone();
        } else {
            target.push((key.clone(), value.clone(), key_position.clone()));
        }
    }
}

// String functions

fn uppercase_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("uppercase", args, 1, position)?;
    let sv = string_value(&args[0]);
    if sv.quoted {
        Ok(Expression::string(
            sv.value.to_uppercase(),
            position.clone(),
        ))
    } else {
        Ok(Expression::identifier(
            sv.value.to_uppercase(),
            position.clone(),
        ))
    }
}

fn lowercase_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("lowercase", args, 1, position)?;
    let sv = string_value(&args[0]);
    if sv.quoted {
        Ok(Expression::string(
            sv.value.to_lowercase(),
            position.clone(),
        ))
    } else {
        Ok(Expression::identifier(
            sv.value.to_lowercase(),
            position.clone(),
        ))
    }
}

fn length_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("length", args, 1, position)?;
    match &args[0] {
        Expression::String { value, .. } => {
            Ok(Expression::number(value.len() as f64, position.clone()))
        }
        Expression::List { values, .. } => {
            Ok(Expression::number(values.len() as f64, position.clone()))
        }
        _ => {
            let sv = string_value(&args[0]);
            Ok(Expression::number(sv.value.len() as f64, position.clone()))
        }
    }
}

fn extract_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 2 {
        return Err(Error::function_error(
            "extract",
            "Expected 2 arguments",
            position.line,
            position.column,
        ));
    }
    let index = match &args[1] {
        Expression::Number { value, .. } => *value as usize,
        _ => {
            return Err(Error::function_error(
                "extract",
                "Index must be a number",
                position.line,
                position.column,
            ))
        }
    };
    if index == 0 {
        return Err(Error::function_error(
            "extract",
            "Index is 1-based, cannot be 0",
            position.line,
            position.column,
        ));
    }
    match &args[0] {
        Expression::List { values, .. } => {
            if index > values.len() {
                Err(Error::function_error(
                    "extract",
                    "Index out of range",
                    position.line,
                    position.column,
                ))
            } else {
                Ok(values[index - 1].clone())
            }
        }
        _ => {
            if index == 1 {
                Ok(args[0].clone())
            } else {
                Err(Error::function_error(
                    "extract",
                    "Index out of range",
                    position.line,
                    position.column,
                ))
            }
        }
    }
}

fn map_get_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() < 2 {
        return Err(Error::function_error(
            "map-get",
            "Expected at least 2 arguments (map, key...)",
            position.line,
            position.column,
        ));
    }

    let entries = match &args[0] {
        Expression::MapLiteral { entries, .. } => entries,
        _ => {
            return Err(Error::function_error(
                "map-get",
                "First argument must be a map",
                position.line,
                position.column,
            ));
        }
    };

    let path = map_path_keys(args, 1);
    match map_lookup_path(entries, &path) {
        Ok(Some(value)) => return Ok(value.clone()),
        Ok(None) => {}
        Err(message) => {
            return Err(Error::function_error(
                "map-get",
                message,
                position.line,
                position.column,
            ));
        }
    }

    Err(Error::function_error(
        "map-get",
        format!("Key path '{}' not found in map", path.join(" -> ")),
        position.line,
        position.column,
    ))
}

fn map_has_key_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() < 2 {
        return Err(Error::function_error(
            "map-has-key",
            "Expected at least 2 arguments (map, key...)",
            position.line,
            position.column,
        ));
    }

    let entries = match &args[0] {
        Expression::MapLiteral { entries, .. } => entries,
        _ => {
            return Err(Error::function_error(
                "map-has-key",
                "First argument must be a map",
                position.line,
                position.column,
            ));
        }
    };

    let path = map_path_keys(args, 1);
    match map_lookup_path(entries, &path) {
        Ok(result) => Ok(Expression::Boolean(result.is_some(), position.clone())),
        Err(message) => Err(Error::function_error(
            "map-has-key",
            message,
            position.line,
            position.column,
        )),
    }
}

fn map_keys_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("map-keys", args, 1, position)?;
    let entries = match &args[0] {
        Expression::MapLiteral { entries, .. } => entries,
        _ => {
            return Err(Error::function_error(
                "map-keys",
                "Argument must be a map",
                position.line,
                position.column,
            ));
        }
    };

    let values = entries
        .iter()
        .map(|(k, _, _)| {
            if let Some(unquoted) = unquote_map_key(k) {
                Expression::string(unquoted.to_string(), position.clone())
            } else {
                Expression::identifier(k.clone(), position.clone())
            }
        })
        .collect();
    Ok(Expression::list(
        values,
        crate::ast::ListSeparator::Comma,
        position.clone(),
    ))
}

fn map_values_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("map-values", args, 1, position)?;
    let entries = match &args[0] {
        Expression::MapLiteral { entries, .. } => entries,
        _ => {
            return Err(Error::function_error(
                "map-values",
                "Argument must be a map",
                position.line,
                position.column,
            ));
        }
    };

    let values = entries.iter().map(|(_, v, _)| v.clone()).collect();
    Ok(Expression::list(
        values,
        crate::ast::ListSeparator::Comma,
        position.clone(),
    ))
}

fn map_merge_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() < 2 {
        return Err(Error::function_error(
            "map-merge",
            "Expected at least 2 map arguments",
            position.line,
            position.column,
        ));
    }

    let mut merged: Vec<(String, Expression, Position)> = Vec::new();
    for arg in args {
        let entries = match arg {
            Expression::MapLiteral { entries, .. } => entries,
            _ => {
                return Err(Error::function_error(
                    "map-merge",
                    "All arguments must be maps",
                    position.line,
                    position.column,
                ));
            }
        };

        for (key, value, key_position) in entries {
            if let Some((_, existing, _)) =
                merged.iter_mut().find(|(k, _, _)| map_keys_equal(k, key))
            {
                *existing = value.clone();
            } else {
                merged.push((key.clone(), value.clone(), key_position.clone()));
            }
        }
    }

    Ok(Expression::MapLiteral {
        entries: merged,
        position: position.clone(),
    })
}

fn map_deep_merge_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() < 2 {
        return Err(Error::function_error(
            "map-deep-merge",
            "Expected at least 2 map arguments",
            position.line,
            position.column,
        ));
    }

    let mut merged = match &args[0] {
        Expression::MapLiteral { entries, .. } => entries.clone(),
        _ => {
            return Err(Error::function_error(
                "map-deep-merge",
                "All arguments must be maps",
                position.line,
                position.column,
            ));
        }
    };

    for arg in &args[1..] {
        let entries = match arg {
            Expression::MapLiteral { entries, .. } => entries,
            _ => {
                return Err(Error::function_error(
                    "map-deep-merge",
                    "All arguments must be maps",
                    position.line,
                    position.column,
                ));
            }
        };
        deep_merge_entries(&mut merged, entries);
    }

    Ok(Expression::MapLiteral {
        entries: merged,
        position: position.clone(),
    })
}

fn map_set_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() < 3 {
        return Err(Error::function_error(
            "map-set",
            "Expected at least 3 arguments (map, key..., value)",
            position.line,
            position.column,
        ));
    }

    let mut entries = match &args[0] {
        Expression::MapLiteral { entries, .. } => entries.clone(),
        _ => {
            return Err(Error::function_error(
                "map-set",
                "First argument must be a map",
                position.line,
                position.column,
            ));
        }
    };

    let final_path = map_path_keys(&args[..args.len() - 1], 1);
    let value_expr = &args[args.len() - 1];

    if final_path.is_empty() {
        return Err(Error::function_error(
            "map-set",
            "Key path cannot be empty",
            position.line,
            position.column,
        ));
    }

    set_map_path(&mut entries, &final_path, value_expr, position).map_err(|message| {
        Error::function_error("map-set", message, position.line, position.column)
    })?;

    Ok(Expression::MapLiteral {
        entries,
        position: position.clone(),
    })
}

fn map_update_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() < 3 {
        return Err(Error::function_error(
            "map-update",
            "Expected at least 3 arguments (map, key..., value)",
            position.line,
            position.column,
        ));
    }

    let mut entries = match &args[0] {
        Expression::MapLiteral { entries, .. } => entries.clone(),
        _ => {
            return Err(Error::function_error(
                "map-update",
                "First argument must be a map",
                position.line,
                position.column,
            ));
        }
    };

    let final_path = map_path_keys(&args[..args.len() - 1], 1);
    let value_expr = &args[args.len() - 1];

    if final_path.is_empty() {
        return Err(Error::function_error(
            "map-update",
            "Key path cannot be empty",
            position.line,
            position.column,
        ));
    }

    match update_map_path(&mut entries, &final_path, value_expr) {
        Ok(true) => {}
        Ok(false) => {
            return Err(Error::function_error(
                "map-update",
                format!("Key path '{}' not found in map", final_path.join(" -> ")),
                position.line,
                position.column,
            ));
        }
        Err(message) => {
            return Err(Error::function_error(
                "map-update",
                message,
                position.line,
                position.column,
            ));
        }
    }

    Ok(Expression::MapLiteral {
        entries,
        position: position.clone(),
    })
}

fn map_replace_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() < 3 {
        return Err(Error::function_error(
            "map-replace",
            "Expected at least 3 arguments (map, key..., value)",
            position.line,
            position.column,
        ));
    }

    let mut entries = match &args[0] {
        Expression::MapLiteral { entries, .. } => entries.clone(),
        _ => {
            return Err(Error::function_error(
                "map-replace",
                "First argument must be a map",
                position.line,
                position.column,
            ));
        }
    };

    let final_path = map_path_keys(&args[..args.len() - 1], 1);
    let value_expr = &args[args.len() - 1];

    if final_path.is_empty() {
        return Err(Error::function_error(
            "map-replace",
            "Key path cannot be empty",
            position.line,
            position.column,
        ));
    }

    match update_map_path(&mut entries, &final_path, value_expr) {
        Ok(true) => {}
        Ok(false) => {
            return Err(Error::function_error(
                "map-replace",
                format!("Key path '{}' not found in map", final_path.join(" -> ")),
                position.line,
                position.column,
            ));
        }
        Err(message) => {
            return Err(Error::function_error(
                "map-replace",
                message,
                position.line,
                position.column,
            ));
        }
    }

    Ok(Expression::MapLiteral {
        entries,
        position: position.clone(),
    })
}

fn map_remove_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() < 2 {
        return Err(Error::function_error(
            "map-remove",
            "Expected at least 2 arguments (map, key...)",
            position.line,
            position.column,
        ));
    }

    let mut entries = match &args[0] {
        Expression::MapLiteral { entries, .. } => entries.clone(),
        _ => {
            return Err(Error::function_error(
                "map-remove",
                "First argument must be a map",
                position.line,
                position.column,
            ));
        }
    };

    let path = map_path_keys(args, 1);
    if path.is_empty() {
        return Err(Error::function_error(
            "map-remove",
            "Key path cannot be empty",
            position.line,
            position.column,
        ));
    }

    match remove_map_path(&mut entries, &path) {
        RemoveMapPathResult::Removed | RemoveMapPathResult::Missing => {}
        RemoveMapPathResult::IntermediateNotMap(key) => {
            return Err(Error::function_error(
                "map-remove",
                format!("Intermediate key '{}' is not a map", key),
                position.line,
                position.column,
            ));
        }
    }

    Ok(Expression::MapLiteral {
        entries,
        position: position.clone(),
    })
}

fn map_deep_remove_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() < 2 {
        return Err(Error::function_error(
            "map-deep-remove",
            "Expected at least 2 arguments (map, key...)",
            position.line,
            position.column,
        ));
    }

    let mut entries = match &args[0] {
        Expression::MapLiteral { entries, .. } => entries.clone(),
        _ => {
            return Err(Error::function_error(
                "map-deep-remove",
                "First argument must be a map",
                position.line,
                position.column,
            ));
        }
    };

    let path = map_path_keys(args, 1);
    if path.is_empty() {
        return Err(Error::function_error(
            "map-deep-remove",
            "Key path cannot be empty",
            position.line,
            position.column,
        ));
    }

    match deep_remove_map_path(&mut entries, &path) {
        RemoveMapPathResult::Removed | RemoveMapPathResult::Missing => {}
        RemoveMapPathResult::IntermediateNotMap(key) => {
            return Err(Error::function_error(
                "map-deep-remove",
                format!("Intermediate key '{}' is not a map", key),
                position.line,
                position.column,
            ));
        }
    }

    Ok(Expression::MapLiteral {
        entries,
        position: position.clone(),
    })
}

// Type check functions

fn isnumber_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("isnumber", args, 1, position)?;
    Ok(Expression::Boolean(
        matches!(&args[0], Expression::Number { .. }),
        position.clone(),
    ))
}

fn iscolor_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("iscolor", args, 1, position)?;
    Ok(Expression::Boolean(
        matches!(&args[0], Expression::Color { .. }),
        position.clone(),
    ))
}

fn isstring_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("isstring", args, 1, position)?;
    Ok(Expression::Boolean(
        matches!(&args[0], Expression::String { .. }),
        position.clone(),
    ))
}

fn iskeyword_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("iskeyword", args, 1, position)?;
    let is_keyword = matches!(&args[0], Expression::String { quoted: false, .. });
    Ok(Expression::Boolean(is_keyword, position.clone()))
}

fn isurl_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("isurl", args, 1, position)?;
    Ok(Expression::Boolean(
        matches!(&args[0], Expression::Url(_, _)),
        position.clone(),
    ))
}

fn ispixel_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("ispixel", args, 1, position)?;
    let is_px = matches!(&args[0], Expression::Number { unit: Some(u), .. } if u == "px");
    Ok(Expression::Boolean(is_px, position.clone()))
}

fn isem_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("isem", args, 1, position)?;
    let is_em = matches!(&args[0], Expression::Number { unit: Some(u), .. } if u == "em");
    Ok(Expression::Boolean(is_em, position.clone()))
}

fn ispercentage_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("ispercentage", args, 1, position)?;
    let is_pct = matches!(&args[0], Expression::Percentage(_, _))
        || matches!(&args[0], Expression::Number { unit: Some(u), .. } if u == "%");
    Ok(Expression::Boolean(is_pct, position.clone()))
}

fn unit_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.is_empty() || args.len() > 2 {
        return Err(Error::function_error(
            "unit",
            "Expected 1 or 2 arguments",
            position.line,
            position.column,
        ));
    }
    match &args[0] {
        Expression::Number { value, .. } | Expression::Percentage(value, _) => {
            if args.len() == 2 {
                // Change unit
                let new_unit = string_value(&args[1]).value;
                if new_unit.is_empty() {
                    Ok(Expression::number(*value, position.clone()))
                } else {
                    Ok(Expression::number_with_unit(
                        *value,
                        new_unit,
                        position.clone(),
                    ))
                }
            } else {
                // Remove unit (return dimensionless number)
                Ok(Expression::number(*value, position.clone()))
            }
        }
        _ => Err(Error::function_error(
            "unit",
            "First argument must be a number",
            position.line,
            position.column,
        )),
    }
}

fn get_unit_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("get-unit", args, 1, position)?;
    let unit_str = match &args[0] {
        Expression::Number { unit: Some(u), .. } => u.clone(),
        Expression::Percentage(_, _) => "%".to_string(),
        Expression::Number { unit: None, .. } => String::new(),
        _ => {
            return Err(Error::function_error(
                "get-unit",
                "Argument must be a number",
                position.line,
                position.column,
            ))
        }
    };
    Ok(Expression::identifier(unit_str, position.clone()))
}

// Advanced math functions

fn sqrt_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("sqrt", args, 1, position)?;
    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::Number {
            value: value.sqrt(),
            unit: unit.clone(),
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "sqrt",
            "Expected number argument",
            position.line,
            position.column,
        )),
    }
}

fn sin_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("sin", args, 1, position)?;
    match &args[0] {
        Expression::Number { value, .. } => Ok(Expression::number(value.sin(), position.clone())),
        _ => Err(Error::function_error(
            "sin",
            "Expected number argument",
            position.line,
            position.column,
        )),
    }
}

fn cos_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("cos", args, 1, position)?;
    match &args[0] {
        Expression::Number { value, .. } => Ok(Expression::number(value.cos(), position.clone())),
        _ => Err(Error::function_error(
            "cos",
            "Expected number argument",
            position.line,
            position.column,
        )),
    }
}

fn tan_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("tan", args, 1, position)?;
    match &args[0] {
        Expression::Number { value, .. } => Ok(Expression::number(value.tan(), position.clone())),
        _ => Err(Error::function_error(
            "tan",
            "Expected number argument",
            position.line,
            position.column,
        )),
    }
}

fn asin_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("asin", args, 1, position)?;
    match &args[0] {
        Expression::Number { value, .. } => Ok(Expression::number(value.asin(), position.clone())),
        _ => Err(Error::function_error(
            "asin",
            "Expected number argument",
            position.line,
            position.column,
        )),
    }
}

fn acos_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("acos", args, 1, position)?;
    match &args[0] {
        Expression::Number { value, .. } => Ok(Expression::number(value.acos(), position.clone())),
        _ => Err(Error::function_error(
            "acos",
            "Expected number argument",
            position.line,
            position.column,
        )),
    }
}

fn atan_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("atan", args, 1, position)?;
    match &args[0] {
        Expression::Number { value, .. } => Ok(Expression::number(value.atan(), position.clone())),
        _ => Err(Error::function_error(
            "atan",
            "Expected number argument",
            position.line,
            position.column,
        )),
    }
}

fn pow_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 2 {
        return Err(Error::function_error(
            "pow",
            "Expected 2 arguments",
            position.line,
            position.column,
        ));
    }
    match (&args[0], &args[1]) {
        (
            Expression::Number {
                value: base, unit, ..
            },
            Expression::Number { value: exp, .. },
        ) => Ok(Expression::Number {
            value: base.powf(*exp),
            unit: unit.clone(),
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "pow",
            "Arguments must be numbers",
            position.line,
            position.column,
        )),
    }
}

fn pi_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if !args.is_empty() {
        return Err(Error::function_error(
            "pi",
            "Expected 0 arguments",
            position.line,
            position.column,
        ));
    }
    Ok(Expression::number(std::f64::consts::PI, position.clone()))
}

fn mod_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 2 {
        return Err(Error::function_error(
            "mod",
            "Expected 2 arguments",
            position.line,
            position.column,
        ));
    }
    match (&args[0], &args[1]) {
        (Expression::Number { value: a, unit, .. }, Expression::Number { value: b, .. }) => {
            if *b == 0.0 {
                return Err(Error::function_error(
                    "mod",
                    "Division by zero",
                    position.line,
                    position.column,
                ));
            }
            Ok(Expression::Number {
                value: a % b,
                unit: unit.clone(),
                position: position.clone(),
            })
        }
        _ => Err(Error::function_error(
            "mod",
            "Arguments must be numbers",
            position.line,
            position.column,
        )),
    }
}

/// Transform function: scale()
fn scale_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.is_empty() {
        return Err(Error::function_error(
            "scale",
            "requires at least 1 argument",
            position.line,
            position.column,
        ));
    }

    let scale_values: Result<Vec<String>> = args
        .iter()
        .map(|arg| match arg {
            Expression::Number { value, unit, .. } => {
                if let Some(u) = unit {
                    Ok(format!("{}{}", value, u))
                } else {
                    Ok(value.to_string())
                }
            }
            Expression::String { value, .. } => Ok(value.clone()),
            _ => Err(Error::function_error(
                "scale",
                "arguments must be numbers",
                position.line,
                position.column,
            )),
        })
        .collect();

    let values = scale_values?;
    let scale_str = if values.len() == 1 {
        format!("scale({})", values[0])
    } else {
        format!("scale({})", values.join(", "))
    };

    Ok(Expression::identifier(scale_str, position.clone()))
}

/// Transform function: translateX()
fn translate_x_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 1 {
        return Err(Error::function_error(
            "translateX",
            "requires exactly 1 argument",
            position.line,
            position.column,
        ));
    }

    match &args[0] {
        Expression::Number { value, unit, .. } => {
            let unit_str = unit.as_deref().unwrap_or("");
            let translate_str = format!("translateX({}{})", value, unit_str);
            Ok(Expression::identifier(translate_str, position.clone()))
        }
        Expression::String { value, .. } => {
            let translate_str = format!("translateX({})", value);
            Ok(Expression::identifier(translate_str, position.clone()))
        }
        _ => Err(Error::function_error(
            "translateX",
            "argument must be a number",
            position.line,
            position.column,
        )),
    }
}

/// Transform function: translateY()
fn translate_y_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 1 {
        return Err(Error::function_error(
            "translateY",
            "requires exactly 1 argument",
            position.line,
            position.column,
        ));
    }

    match &args[0] {
        Expression::Number { value, unit, .. } => {
            let unit_str = unit.as_deref().unwrap_or("");
            let translate_str = format!("translateY({}{})", value, unit_str);
            Ok(Expression::identifier(translate_str, position.clone()))
        }
        Expression::String { value, .. } => {
            let translate_str = format!("translateY({})", value);
            Ok(Expression::identifier(translate_str, position.clone()))
        }
        _ => Err(Error::function_error(
            "translateY",
            "argument must be a number",
            position.line,
            position.column,
        )),
    }
}

/// Transform function: rotate()
fn rotate_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 1 {
        return Err(Error::function_error(
            "rotate",
            "requires exactly 1 argument",
            position.line,
            position.column,
        ));
    }

    match &args[0] {
        Expression::Number { value, unit, .. } => {
            let unit_str = unit.as_deref().unwrap_or("");
            let rotate_str = format!("rotate({}{})", value, unit_str);
            Ok(Expression::identifier(rotate_str, position.clone()))
        }
        Expression::String { value, .. } => {
            let rotate_str = format!("rotate({})", value);
            Ok(Expression::identifier(rotate_str, position.clone()))
        }
        _ => Err(Error::function_error(
            "rotate",
            "argument must be a number",
            position.line,
            position.column,
        )),
    }
}

// Conditional function: if(condition, trueVal, falseVal)

fn if_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 3 {
        return Err(Error::function_error(
            "if",
            "Expected 3 arguments (condition, trueVal, falseVal)",
            position.line,
            position.column,
        ));
    }

    let is_truthy = match &args[0] {
        Expression::Boolean(b, _) => *b,
        Expression::Number { value, .. } => *value != 0.0,
        Expression::String { value, .. } => !value.is_empty() && value != "false",
        _ => false,
    };

    if is_truthy {
        Ok(args[1].clone())
    } else {
        Ok(args[2].clone())
    }
}

// List generation: range(start, end, step?)

fn range_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.is_empty() || args.len() > 3 {
        return Err(Error::function_error(
            "range",
            "Expected 1 to 3 arguments (start, end?, step?)",
            position.line,
            position.column,
        ));
    }

    let (start, end, step, unit) = if args.len() == 1 {
        // range(count) -> 1..count
        match &args[0] {
            Expression::Number { value, unit, .. } => (1.0, *value, 1.0, unit.clone()),
            _ => {
                return Err(Error::function_error(
                    "range",
                    "Arguments must be numbers",
                    position.line,
                    position.column,
                ))
            }
        }
    } else {
        let (start_val, unit) = match &args[0] {
            Expression::Number { value, unit, .. } => (*value, unit.clone()),
            _ => {
                return Err(Error::function_error(
                    "range",
                    "Arguments must be numbers",
                    position.line,
                    position.column,
                ))
            }
        };
        let end_val = match &args[1] {
            Expression::Number { value, .. } => *value,
            _ => {
                return Err(Error::function_error(
                    "range",
                    "Arguments must be numbers",
                    position.line,
                    position.column,
                ))
            }
        };
        let step_val = if args.len() == 3 {
            match &args[2] {
                Expression::Number { value, .. } => *value,
                _ => {
                    return Err(Error::function_error(
                        "range",
                        "Step must be a number",
                        position.line,
                        position.column,
                    ))
                }
            }
        } else {
            1.0
        };
        (start_val, end_val, step_val, unit)
    };

    if step == 0.0 {
        return Err(Error::function_error(
            "range",
            "Step cannot be zero",
            position.line,
            position.column,
        ));
    }

    let mut values = Vec::new();
    let mut current = start;
    while (step > 0.0 && current <= end) || (step < 0.0 && current >= end) {
        values.push(Expression::Number {
            value: current,
            unit: unit.clone(),
            position: position.clone(),
        });
        current += step;
        // Safety: limit iterations
        if values.len() > 10000 {
            return Err(Error::function_error(
                "range",
                "Range exceeds maximum 10000 items",
                position.line,
                position.column,
            ));
        }
    }

    Ok(Expression::list(
        values,
        crate::ast::ListSeparator::Space,
        position.clone(),
    ))
}

// Unit conversion: convert(value, targetUnit)

fn convert_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 2 {
        return Err(Error::function_error(
            "convert",
            "Expected 2 arguments (value, targetUnit)",
            position.line,
            position.column,
        ));
    }

    let (value, from_unit) = match &args[0] {
        Expression::Number {
            value,
            unit: Some(u),
            ..
        } => (*value, u.clone()),
        _ => {
            return Err(Error::function_error(
                "convert",
                "First argument must be a number with a unit",
                position.line,
                position.column,
            ))
        }
    };

    let to_unit = string_value(&args[1]).value;

    let result = convert_units(value, &from_unit, &to_unit);
    match result {
        Some(converted) => Ok(Expression::Number {
            value: converted,
            unit: Some(to_unit),
            position: position.clone(),
        }),
        None => Err(Error::function_error(
            "convert",
            format!("Cannot convert from '{}' to '{}'", from_unit, to_unit),
            position.line,
            position.column,
        )),
    }
}

/// Convert a value between compatible units
fn convert_units(value: f64, from: &str, to: &str) -> Option<f64> {
    if from == to {
        return Some(value);
    }

    // Length: base unit = px
    let to_px = |u: &str| -> Option<f64> {
        match u {
            "px" => Some(1.0),
            "in" => Some(96.0),
            "cm" => Some(96.0 / 2.54),
            "mm" => Some(96.0 / 25.4),
            "pt" => Some(96.0 / 72.0),
            "pc" => Some(96.0 / 6.0),
            _ => None,
        }
    };

    // Try length conversion
    if let (Some(from_factor), Some(to_factor)) = (to_px(from), to_px(to)) {
        return Some(value * from_factor / to_factor);
    }

    // Time: base unit = s
    let to_s = |u: &str| -> Option<f64> {
        match u {
            "s" => Some(1.0),
            "ms" => Some(0.001),
            _ => None,
        }
    };

    if let (Some(from_factor), Some(to_factor)) = (to_s(from), to_s(to)) {
        return Some(value * from_factor / to_factor);
    }

    // Angle: base unit = deg
    let to_deg = |u: &str| -> Option<f64> {
        match u {
            "deg" => Some(1.0),
            "rad" => Some(180.0 / std::f64::consts::PI),
            "grad" => Some(0.9),
            "turn" => Some(360.0),
            _ => None,
        }
    };

    if let (Some(from_factor), Some(to_factor)) = (to_deg(from), to_deg(to)) {
        return Some(value * from_factor / to_factor);
    }

    None
}

// default() function placeholder for mixin guards
fn default_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if !args.is_empty() {
        return Err(Error::function_error(
            "default",
            "Expected 0 arguments",
            position.line,
            position.column,
        ));
    }
    // Returns true; actual logic is handled by mixin matching in compiler/mixin.rs
    Ok(Expression::Boolean(true, position.clone()))
}

// ===== Color channel access functions =====

fn red_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("red", args, 1, position)?;
    let (r, _, _, _) = expression_to_color(&args[0], position)?;
    Ok(Expression::number(r as f64, position.clone()))
}

fn green_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("green", args, 1, position)?;
    let (_, g, _, _) = expression_to_color(&args[0], position)?;
    Ok(Expression::number(g as f64, position.clone()))
}

fn blue_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("blue", args, 1, position)?;
    let (_, _, b, _) = expression_to_color(&args[0], position)?;
    Ok(Expression::number(b as f64, position.clone()))
}

fn alpha_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("alpha", args, 1, position)?;
    let (_, _, _, a) = expression_to_color(&args[0], position)?;
    Ok(Expression::number(a, position.clone()))
}

fn hue_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("hue", args, 1, position)?;
    let (r, g, b, _) = expression_to_color(&args[0], position)?;
    let (h, _, _) = rgb_to_hsl(r, g, b);
    Ok(Expression::number((h * 360.0).round(), position.clone()))
}

fn saturation_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("saturation", args, 1, position)?;
    let (r, g, b, _) = expression_to_color(&args[0], position)?;
    let (_, s, _) = rgb_to_hsl(r, g, b);
    Ok(Expression::Percentage(
        (s * 100.0).round(),
        position.clone(),
    ))
}

fn lightness_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("lightness", args, 1, position)?;
    let (r, g, b, _) = expression_to_color(&args[0], position)?;
    let (_, _, l) = rgb_to_hsl(r, g, b);
    Ok(Expression::Percentage(
        (l * 100.0).round(),
        position.clone(),
    ))
}

fn luma_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("luma", args, 1, position)?;
    let (r, g, b, _) = expression_to_color(&args[0], position)?;
    let luma = calculate_luma(r, g, b);
    Ok(Expression::Percentage(
        (luma * 100.0 * 100.0).round() / 100.0,
        position.clone(),
    ))
}

/// Calculate perceptual luma with gamma correction
fn calculate_luma(r: u8, g: u8, b: u8) -> f64 {
    let r_lin = (r as f64 / 255.0).powf(2.2);
    let g_lin = (g as f64 / 255.0).powf(2.2);
    let b_lin = (b as f64 / 255.0).powf(2.2);
    0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin
}

fn argb_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("argb", args, 1, position)?;
    let (r, g, b, a) = expression_to_color(&args[0], position)?;
    let alpha_byte = (a * 255.0).round().clamp(0.0, 255.0) as u8;
    let result = format!("#{:02x}{:02x}{:02x}{:02x}", alpha_byte, r, g, b);
    Ok(Expression::identifier(result, position.clone()))
}

// ===== Color blending functions =====

/// Apply a per-channel blend operation to two colors
fn blend_colors(
    name: &str,
    args: &[Expression],
    position: &Position,
    blend_fn: fn(f64, f64) -> f64,
) -> Result<Expression> {
    if args.len() != 2 {
        return Err(Error::function_error(
            name,
            "Expected 2 arguments",
            position.line,
            position.column,
        ));
    }
    let (r1, g1, b1, _) = expression_to_color(&args[0], position)?;
    let (r2, g2, b2, _) = expression_to_color(&args[1], position)?;

    let r = (blend_fn(r1 as f64 / 255.0, r2 as f64 / 255.0) * 255.0)
        .round()
        .clamp(0.0, 255.0) as u8;
    let g = (blend_fn(g1 as f64 / 255.0, g2 as f64 / 255.0) * 255.0)
        .round()
        .clamp(0.0, 255.0) as u8;
    let b = (blend_fn(b1 as f64 / 255.0, b2 as f64 / 255.0) * 255.0)
        .round()
        .clamp(0.0, 255.0) as u8;

    Ok(Expression::Color {
        red: r,
        green: g,
        blue: b,
        alpha: 1.0,
        original: None,
        position: position.clone(),
    })
}

fn multiply_function(args: &[Expression], position: &Position) -> Result<Expression> {
    blend_colors("multiply", args, position, |c1, c2| c1 * c2)
}

fn screen_function(args: &[Expression], position: &Position) -> Result<Expression> {
    blend_colors("screen", args, position, |c1, c2| {
        1.0 - (1.0 - c1) * (1.0 - c2)
    })
}

fn overlay_function(args: &[Expression], position: &Position) -> Result<Expression> {
    blend_colors("overlay", args, position, |c1, c2| {
        if c1 < 0.5 {
            2.0 * c1 * c2
        } else {
            1.0 - 2.0 * (1.0 - c1) * (1.0 - c2)
        }
    })
}

fn softlight_function(args: &[Expression], position: &Position) -> Result<Expression> {
    // Pegtop formula
    blend_colors("softlight", args, position, |c1, c2| {
        (1.0 - 2.0 * c2) * c1 * c1 + 2.0 * c2 * c1
    })
}

fn hardlight_function(args: &[Expression], position: &Position) -> Result<Expression> {
    // hardlight is overlay with swapped arguments
    blend_colors("hardlight", args, position, |c1, c2| {
        if c2 < 0.5 {
            2.0 * c1 * c2
        } else {
            1.0 - 2.0 * (1.0 - c1) * (1.0 - c2)
        }
    })
}

fn difference_function(args: &[Expression], position: &Position) -> Result<Expression> {
    blend_colors("difference", args, position, |c1, c2| (c1 - c2).abs())
}

fn exclusion_function(args: &[Expression], position: &Position) -> Result<Expression> {
    blend_colors("exclusion", args, position, |c1, c2| {
        c1 + c2 - 2.0 * c1 * c2
    })
}

fn average_function(args: &[Expression], position: &Position) -> Result<Expression> {
    blend_colors("average", args, position, |c1, c2| (c1 + c2) / 2.0)
}

fn negation_function(args: &[Expression], position: &Position) -> Result<Expression> {
    blend_colors("negation", args, position, |c1, c2| {
        1.0 - (1.0 - c1 - c2).abs()
    })
}

// ===== Convenience color functions =====

fn tint_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 2 {
        return Err(Error::function_error(
            "tint",
            "Expected 2 arguments",
            position.line,
            position.column,
        ));
    }
    // tint(@color, @amount) = mix(white, @color, @amount)
    let white = Expression::Color {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 1.0,
        original: None,
        position: position.clone(),
    };
    let mix_args = vec![white, args[0].clone(), args[1].clone()];
    mix_function(&mix_args, position)
}

fn shade_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 2 {
        return Err(Error::function_error(
            "shade",
            "Expected 2 arguments",
            position.line,
            position.column,
        ));
    }
    // shade(@color, @amount) = mix(black, @color, @amount)
    let black = Expression::Color {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 1.0,
        original: None,
        position: position.clone(),
    };
    let mix_args = vec![black, args[0].clone(), args[1].clone()];
    mix_function(&mix_args, position)
}

fn contrast_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.is_empty() || args.len() > 4 {
        return Err(Error::function_error(
            "contrast",
            "Expected 1 to 4 arguments",
            position.line,
            position.column,
        ));
    }

    let (r, g, b, _) = expression_to_color(&args[0], position)?;

    let dark = if args.len() >= 2 {
        args[1].clone()
    } else {
        Expression::Color {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 1.0,
            original: None,
            position: position.clone(),
        }
    };

    let light = if args.len() >= 3 {
        args[2].clone()
    } else {
        Expression::Color {
            red: 255,
            green: 255,
            blue: 255,
            alpha: 1.0,
            original: None,
            position: position.clone(),
        }
    };

    let threshold = if args.len() >= 4 {
        expression_to_percentage(&args[3], position)? / 100.0
    } else {
        0.43
    };

    let luma = calculate_luma(r, g, b);

    if luma < threshold {
        Ok(light)
    } else {
        Ok(dark)
    }
}

// Helper functions for color and percentage conversion

/// Convert an expression to a color, handling hex strings and existing colors
fn expression_to_color(expr: &Expression, position: &Position) -> Result<(u8, u8, u8, f64)> {
    match expr {
        Expression::Color {
            red,
            green,
            blue,
            alpha,
            ..
        } => Ok((*red, *green, *blue, *alpha)),
        Expression::String { value, .. } => {
            // Try to parse as hex color
            if value.starts_with('#') {
                match Expression::color_hex(value, position.clone()) {
                    Ok(Expression::Color {
                        red,
                        green,
                        blue,
                        alpha,
                        ..
                    }) => Ok((red, green, blue, alpha)),
                    _ => Err(Error::function_error(
                        "color conversion",
                        "Invalid hex color format",
                        position.line,
                        position.column,
                    )),
                }
            } else {
                Err(Error::function_error(
                    "color conversion",
                    "String must be a valid hex color",
                    position.line,
                    position.column,
                ))
            }
        }
        _ => Err(Error::function_error(
            "color conversion",
            "Argument must be a color or hex string",
            position.line,
            position.column,
        )),
    }
}

/// Convert an expression to a percentage value
fn expression_to_percentage(expr: &Expression, position: &Position) -> Result<f64> {
    match expr {
        Expression::Percentage(value, _) => Ok(*value),
        Expression::Number { value, unit, .. } => {
            if unit.as_deref() == Some("%") {
                Ok(*value)
            } else {
                // Allow unitless numbers to be treated as percentages
                Ok(*value)
            }
        }
        _ => Err(Error::function_error(
            "percentage conversion",
            "Argument must be a number or percentage",
            position.line,
            position.column,
        )),
    }
}

// Color utility functions for HSL conversion

/// Convert RGB to HSL color space
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;

    // Lightness
    let l = (max + min) / 2.0;

    if delta == 0.0 {
        // Achromatic (gray)
        return (0.0, 0.0, l);
    }

    // Saturation
    let s = if l < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    // Hue
    let h = if max == r {
        ((g - b) / delta + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / delta + 2.0) / 6.0
    } else {
        ((r - g) / delta + 4.0) / 6.0
    };

    (h, s, l)
}

/// Convert HSL to RGB color space
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r_prime, g_prime, b_prime) = match (h * 6.0) as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        5 => (c, 0.0, x),
        _ => (c, x, 0.0), // fallback for h = 1.0
    };

    let r = ((r_prime + m) * 255.0).round() as u8;
    let g = ((g_prime + m) * 255.0).round() as u8;
    let b = ((b_prime + m) * 255.0).round() as u8;

    (r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_function() {
        let registry = FunctionRegistry::new();
        let pos = Position::new(1, 1);
        let args = vec![Expression::Number {
            value: 10.6,
            unit: Some("px".to_string()),
            position: pos.clone(),
        }];

        let result = registry.call("round", &args, &pos).unwrap();

        if let Expression::Number { value, unit, .. } = result {
            assert_eq!(value, 11.0);
            assert_eq!(unit, Some("px".to_string()));
        } else {
            panic!("Expected number result");
        }
    }

    #[test]
    fn test_percentage_function() {
        let registry = FunctionRegistry::new();
        let pos = Position::new(1, 1);
        let args = vec![Expression::Number {
            value: 0.5,
            unit: None,
            position: pos.clone(),
        }];

        let result = registry.call("percentage", &args, &pos).unwrap();

        if let Expression::Percentage(value, _) = result {
            assert_eq!(value, 50.0);
        } else {
            panic!("Expected percentage result");
        }
    }

    #[test]
    fn test_rgb_function() {
        let registry = FunctionRegistry::new();
        let pos = Position::new(1, 1);
        let args = vec![
            Expression::Number {
                value: 255.0,
                unit: None,
                position: pos.clone(),
            },
            Expression::Number {
                value: 0.0,
                unit: None,
                position: pos.clone(),
            },
            Expression::Number {
                value: 0.0,
                unit: None,
                position: pos.clone(),
            },
        ];

        let result = registry.call("rgb", &args, &pos).unwrap();

        if let Expression::Color {
            red,
            green,
            blue,
            alpha,
            ..
        } = result
        {
            assert_eq!(red, 255);
            assert_eq!(green, 0);
            assert_eq!(blue, 0);
            assert_eq!(alpha, 1.0);
        } else {
            panic!("Expected color result");
        }
    }

    #[test]
    fn test_min_max_functions() {
        let registry = FunctionRegistry::new();
        let pos = Position::new(1, 1);
        let args = vec![
            Expression::Number {
                value: 10.0,
                unit: Some("px".to_string()),
                position: pos.clone(),
            },
            Expression::Number {
                value: 5.0,
                unit: Some("px".to_string()),
                position: pos.clone(),
            },
            Expression::Number {
                value: 15.0,
                unit: Some("px".to_string()),
                position: pos.clone(),
            },
        ];

        let min_result = registry.call("min", &args, &pos).unwrap();
        let max_result = registry.call("max", &args, &pos).unwrap();

        if let Expression::Number { value, .. } = min_result {
            assert_eq!(value, 5.0);
        } else {
            panic!("Expected number result from min");
        }

        if let Expression::Number { value, .. } = max_result {
            assert_eq!(value, 15.0);
        } else {
            panic!("Expected number result from max");
        }
    }

    #[test]
    fn test_undefined_function() {
        let registry = FunctionRegistry::new();
        let pos = Position::new(1, 1);
        let args = vec![];

        let result = registry.call("nonexistent", &args, &pos);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::UndefinedFunction { .. }
        ));
    }

    #[test]
    fn test_escape_function() {
        let registry = FunctionRegistry::new();
        let pos = Position::new(1, 1);
        let args = vec![Expression::string("test".to_string(), pos.clone())];

        let result = registry.call("e", &args, &pos).unwrap();

        if let Expression::Escaped(value, _) = result {
            assert_eq!(value, "test");
        } else {
            panic!("Expected escaped result");
        }
    }

    #[test]
    fn test_replace_function() {
        let registry = FunctionRegistry::new();
        let pos = Position::new(1, 1);
        let args = vec![
            Expression::string("hello world".to_string(), pos.clone()),
            Expression::string("world".to_string(), pos.clone()),
            Expression::string("LESS".to_string(), pos.clone()),
        ];

        let result = registry.call("replace", &args, &pos).unwrap();

        if let Expression::String { value, .. } = result {
            assert_eq!(value, "hello LESS");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_map_get_function() {
        let registry = FunctionRegistry::new();
        let pos = Position::new(1, 1);
        let map = Expression::MapLiteral {
            entries: vec![
                (
                    "small".to_string(),
                    Expression::number_with_unit(10.0, "px", pos.clone()),
                    pos.clone(),
                ),
                (
                    "medium".to_string(),
                    Expression::number_with_unit(20.0, "px", pos.clone()),
                    pos.clone(),
                ),
            ],
            position: pos.clone(),
        };
        let args = vec![
            map,
            Expression::identifier("medium".to_string(), pos.clone()),
        ];

        let result = registry.call("map-get", &args, &pos).unwrap();
        assert_eq!(result.to_css(), "20px");
    }

    #[test]
    fn test_map_keys_values_merge_functions() {
        let registry = FunctionRegistry::new();
        let pos = Position::new(1, 1);
        let map1 = Expression::MapLiteral {
            entries: vec![
                (
                    "small".to_string(),
                    Expression::number_with_unit(10.0, "px", pos.clone()),
                    pos.clone(),
                ),
                (
                    "medium".to_string(),
                    Expression::number_with_unit(20.0, "px", pos.clone()),
                    pos.clone(),
                ),
            ],
            position: pos.clone(),
        };
        let map2 = Expression::MapLiteral {
            entries: vec![
                (
                    "medium".to_string(),
                    Expression::number_with_unit(22.0, "px", pos.clone()),
                    pos.clone(),
                ),
                (
                    "large".to_string(),
                    Expression::number_with_unit(30.0, "px", pos.clone()),
                    pos.clone(),
                ),
            ],
            position: pos.clone(),
        };

        let keys = registry
            .call("map-keys", std::slice::from_ref(&map1), &pos)
            .unwrap();
        assert_eq!(keys.to_css(), "small, medium");

        let values = registry
            .call("map-values", std::slice::from_ref(&map1), &pos)
            .unwrap();
        assert_eq!(values.to_css(), "10px, 20px");

        let merged = registry.call("map-merge", &[map1, map2], &pos).unwrap();
        if let Expression::MapLiteral { entries, .. } = merged {
            assert_eq!(entries.len(), 3);
            assert_eq!(entries[0].0, "small");
            assert_eq!(entries[0].1.to_css(), "10px");
            assert_eq!(entries[1].0, "medium");
            assert_eq!(entries[1].1.to_css(), "22px");
            assert_eq!(entries[2].0, "large");
            assert_eq!(entries[2].1.to_css(), "30px");
        } else {
            panic!("Expected map result from map-merge");
        }
    }

    #[test]
    fn test_map_nested_path_has_key_and_remove_functions() {
        let registry = FunctionRegistry::new();
        let pos = Position::new(1, 1);

        let nested_map = Expression::MapLiteral {
            entries: vec![
                (
                    "breakpoints".to_string(),
                    Expression::MapLiteral {
                        entries: vec![
                            (
                                "sm".to_string(),
                                Expression::number_with_unit(480.0, "px", pos.clone()),
                                pos.clone(),
                            ),
                            (
                                "md".to_string(),
                                Expression::number_with_unit(768.0, "px", pos.clone()),
                                pos.clone(),
                            ),
                        ],
                        position: pos.clone(),
                    },
                    pos.clone(),
                ),
                (
                    "columns".to_string(),
                    Expression::number(12.0, pos.clone()),
                    pos.clone(),
                ),
            ],
            position: pos.clone(),
        };

        let get_result = registry
            .call(
                "map-get",
                &[
                    nested_map.clone(),
                    Expression::identifier("breakpoints".to_string(), pos.clone()),
                    Expression::identifier("md".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(get_result.to_css(), "768px");

        let has_true = registry
            .call(
                "map-has-key",
                &[
                    nested_map.clone(),
                    Expression::identifier("breakpoints".to_string(), pos.clone()),
                    Expression::identifier("sm".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(has_true.to_css(), "true");

        let has_false = registry
            .call(
                "map-has-key",
                &[
                    nested_map.clone(),
                    Expression::identifier("breakpoints".to_string(), pos.clone()),
                    Expression::identifier("lg".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(has_false.to_css(), "false");

        let removed = registry
            .call(
                "map-remove",
                &[
                    nested_map,
                    Expression::identifier("breakpoints".to_string(), pos.clone()),
                    Expression::identifier("sm".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();

        if let Expression::MapLiteral { entries, .. } = removed {
            assert_eq!(entries.len(), 2);
            let breakpoints = entries
                .iter()
                .find(|(k, _, _)| k == "breakpoints")
                .expect("breakpoints should exist");
            if let Expression::MapLiteral {
                entries: nested_entries,
                ..
            } = &breakpoints.1
            {
                assert_eq!(nested_entries.len(), 1);
                assert_eq!(nested_entries[0].0, "md");
                assert_eq!(nested_entries[0].1.to_css(), "768px");
            } else {
                panic!("Expected nested map for breakpoints");
            }
        } else {
            panic!("Expected map result from map-remove");
        }

        let deep_removed = registry
            .call(
                "map-deep-remove",
                &[
                    Expression::MapLiteral {
                        entries: vec![
                            (
                                "a".to_string(),
                                Expression::MapLiteral {
                                    entries: vec![(
                                        "b".to_string(),
                                        Expression::MapLiteral {
                                            entries: vec![(
                                                "c".to_string(),
                                                Expression::number(1.0, pos.clone()),
                                                pos.clone(),
                                            )],
                                            position: pos.clone(),
                                        },
                                        pos.clone(),
                                    )],
                                    position: pos.clone(),
                                },
                                pos.clone(),
                            ),
                            (
                                "keep".to_string(),
                                Expression::number(2.0, pos.clone()),
                                pos.clone(),
                            ),
                        ],
                        position: pos.clone(),
                    },
                    Expression::identifier("a".to_string(), pos.clone()),
                    Expression::identifier("b".to_string(), pos.clone()),
                    Expression::identifier("c".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();

        if let Expression::MapLiteral { entries, .. } = deep_removed {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].0, "keep");
            assert_eq!(entries[0].1.to_css(), "2");
        } else {
            panic!("Expected map result from map-deep-remove");
        }
    }

    #[test]
    fn test_map_set_and_deep_merge_functions() {
        let registry = FunctionRegistry::new();
        let pos = Position::new(1, 1);

        let base = Expression::MapLiteral {
            entries: vec![(
                "config".to_string(),
                Expression::MapLiteral {
                    entries: vec![
                        (
                            "theme".to_string(),
                            Expression::identifier("light".to_string(), pos.clone()),
                            pos.clone(),
                        ),
                        (
                            "spacing".to_string(),
                            Expression::number(8.0, pos.clone()),
                            pos.clone(),
                        ),
                    ],
                    position: pos.clone(),
                },
                pos.clone(),
            )],
            position: pos.clone(),
        };

        let updated = registry
            .call(
                "map-set",
                &[
                    base.clone(),
                    Expression::identifier("config".to_string(), pos.clone()),
                    Expression::identifier("radius".to_string(), pos.clone()),
                    Expression::number(4.0, pos.clone()),
                ],
                &pos,
            )
            .unwrap();

        let radius = registry
            .call(
                "map-get",
                &[
                    updated.clone(),
                    Expression::identifier("config".to_string(), pos.clone()),
                    Expression::identifier("radius".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(radius.to_css(), "4");

        let override_map = Expression::MapLiteral {
            entries: vec![(
                "config".to_string(),
                Expression::MapLiteral {
                    entries: vec![
                        (
                            "spacing".to_string(),
                            Expression::number(10.0, pos.clone()),
                            pos.clone(),
                        ),
                        (
                            "density".to_string(),
                            Expression::identifier("compact".to_string(), pos.clone()),
                            pos.clone(),
                        ),
                    ],
                    position: pos.clone(),
                },
                pos.clone(),
            )],
            position: pos.clone(),
        };

        let deep_merged = registry
            .call("map-deep-merge", &[base, override_map], &pos)
            .unwrap();

        let theme = registry
            .call(
                "map-get",
                &[
                    deep_merged.clone(),
                    Expression::identifier("config".to_string(), pos.clone()),
                    Expression::identifier("theme".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(theme.to_css(), "light");

        let spacing = registry
            .call(
                "map-get",
                &[
                    deep_merged.clone(),
                    Expression::identifier("config".to_string(), pos.clone()),
                    Expression::identifier("spacing".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(spacing.to_css(), "10");

        let density = registry
            .call(
                "map-get",
                &[
                    deep_merged,
                    Expression::identifier("config".to_string(), pos.clone()),
                    Expression::identifier("density".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(density.to_css(), "compact");
    }

    #[test]
    fn test_map_deep_merge_boundary_semantics() {
        let registry = FunctionRegistry::new();
        let pos = Position::new(1, 1);

        let base = Expression::MapLiteral {
            entries: vec![
                (
                    "theme".to_string(),
                    Expression::MapLiteral {
                        entries: vec![(
                            "name".to_string(),
                            Expression::identifier("light".to_string(), pos.clone()),
                            pos.clone(),
                        )],
                        position: pos.clone(),
                    },
                    pos.clone(),
                ),
                (
                    "mode".to_string(),
                    Expression::number(1.0, pos.clone()),
                    pos.clone(),
                ),
            ],
            position: pos.clone(),
        };

        let override_map = Expression::MapLiteral {
            entries: vec![
                (
                    "theme".to_string(),
                    Expression::identifier("flat".to_string(), pos.clone()),
                    pos.clone(),
                ),
                (
                    "mode".to_string(),
                    Expression::MapLiteral {
                        entries: vec![(
                            "nested".to_string(),
                            Expression::identifier("yes".to_string(), pos.clone()),
                            pos.clone(),
                        )],
                        position: pos.clone(),
                    },
                    pos.clone(),
                ),
            ],
            position: pos.clone(),
        };

        let merged = registry
            .call("map-deep-merge", &[base.clone(), override_map], &pos)
            .unwrap();

        let merged_theme = registry
            .call(
                "map-get",
                &[
                    merged.clone(),
                    Expression::identifier("theme".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(merged_theme.to_css(), "flat");

        let merged_mode_nested = registry
            .call(
                "map-get",
                &[
                    merged,
                    Expression::identifier("mode".to_string(), pos.clone()),
                    Expression::identifier("nested".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(merged_mode_nested.to_css(), "yes");

        let base_theme = registry
            .call(
                "map-get",
                &[
                    base.clone(),
                    Expression::identifier("theme".to_string(), pos.clone()),
                    Expression::identifier("name".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(base_theme.to_css(), "light");

        let base_mode = registry
            .call(
                "map-get",
                &[
                    base,
                    Expression::identifier("mode".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(base_mode.to_css(), "1");
    }

    #[test]
    fn test_map_deep_merge_override_order() {
        let registry = FunctionRegistry::new();
        let pos = Position::new(1, 1);

        let base = Expression::MapLiteral {
            entries: vec![(
                "config".to_string(),
                Expression::MapLiteral {
                    entries: vec![(
                        "a".to_string(),
                        Expression::number(1.0, pos.clone()),
                        pos.clone(),
                    )],
                    position: pos.clone(),
                },
                pos.clone(),
            )],
            position: pos.clone(),
        };

        let override1 = Expression::MapLiteral {
            entries: vec![(
                "config".to_string(),
                Expression::MapLiteral {
                    entries: vec![
                        (
                            "a".to_string(),
                            Expression::number(2.0, pos.clone()),
                            pos.clone(),
                        ),
                        (
                            "b".to_string(),
                            Expression::number(3.0, pos.clone()),
                            pos.clone(),
                        ),
                    ],
                    position: pos.clone(),
                },
                pos.clone(),
            )],
            position: pos.clone(),
        };

        let override2 = Expression::MapLiteral {
            entries: vec![(
                "config".to_string(),
                Expression::MapLiteral {
                    entries: vec![(
                        "b".to_string(),
                        Expression::number(4.0, pos.clone()),
                        pos.clone(),
                    )],
                    position: pos.clone(),
                },
                pos.clone(),
            )],
            position: pos.clone(),
        };

        let merged = registry
            .call("map-deep-merge", &[base, override1, override2], &pos)
            .unwrap();

        let a = registry
            .call(
                "map-get",
                &[
                    merged.clone(),
                    Expression::identifier("config".to_string(), pos.clone()),
                    Expression::identifier("a".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(a.to_css(), "2");

        let b = registry
            .call(
                "map-get",
                &[
                    merged,
                    Expression::identifier("config".to_string(), pos.clone()),
                    Expression::identifier("b".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(b.to_css(), "4");
    }

    #[test]
    fn test_map_update_and_replace_functions() {
        let registry = FunctionRegistry::new();
        let pos = Position::new(1, 1);

        let base = Expression::MapLiteral {
            entries: vec![(
                "config".to_string(),
                Expression::MapLiteral {
                    entries: vec![
                        (
                            "theme".to_string(),
                            Expression::identifier("light".to_string(), pos.clone()),
                            pos.clone(),
                        ),
                        (
                            "spacing".to_string(),
                            Expression::number(8.0, pos.clone()),
                            pos.clone(),
                        ),
                    ],
                    position: pos.clone(),
                },
                pos.clone(),
            )],
            position: pos.clone(),
        };

        let updated_theme = registry
            .call(
                "map-update",
                &[
                    base.clone(),
                    Expression::identifier("config".to_string(), pos.clone()),
                    Expression::identifier("theme".to_string(), pos.clone()),
                    Expression::identifier("dark".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        let theme = registry
            .call(
                "map-get",
                &[
                    updated_theme.clone(),
                    Expression::identifier("config".to_string(), pos.clone()),
                    Expression::identifier("theme".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(theme.to_css(), "dark");

        let replaced_spacing = registry
            .call(
                "map-replace",
                &[
                    updated_theme,
                    Expression::identifier("config".to_string(), pos.clone()),
                    Expression::identifier("spacing".to_string(), pos.clone()),
                    Expression::number(12.0, pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        let spacing = registry
            .call(
                "map-get",
                &[
                    replaced_spacing,
                    Expression::identifier("config".to_string(), pos.clone()),
                    Expression::identifier("spacing".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(spacing.to_css(), "12");
    }

    #[test]
    fn test_map_key_normalization_string_identifier_number() {
        let registry = FunctionRegistry::new();
        let pos = Position::new(1, 1);

        let map = Expression::MapLiteral {
            entries: vec![
                (
                    "name".to_string(),
                    Expression::identifier("alpha".to_string(), pos.clone()),
                    pos.clone(),
                ),
                (
                    "3".to_string(),
                    Expression::number_with_unit(30.0, "px", pos.clone()),
                    pos.clone(),
                ),
                (
                    "4px".to_string(),
                    Expression::identifier("hit".to_string(), pos.clone()),
                    pos.clone(),
                ),
            ],
            position: pos.clone(),
        };

        let by_identifier = registry
            .call(
                "map-get",
                &[
                    map.clone(),
                    Expression::identifier("name".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(by_identifier.to_css(), "alpha");

        let by_string = registry
            .call(
                "map-get",
                &[
                    map.clone(),
                    Expression::string("name".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap();
        assert_eq!(by_string.to_css(), "alpha");

        let by_number = registry
            .call(
                "map-get",
                &[map.clone(), Expression::number(3.0, pos.clone())],
                &pos,
            )
            .unwrap();
        assert_eq!(by_number.to_css(), "30px");

        let by_number_with_unit = registry
            .call(
                "map-get",
                &[map, Expression::number_with_unit(4.0, "px", pos.clone())],
                &pos,
            )
            .unwrap();
        assert_eq!(by_number_with_unit.to_css(), "hit");
    }

    #[test]
    fn test_map_error_semantics() {
        let registry = FunctionRegistry::new();
        let pos = Position::new(1, 1);
        let nested = Expression::MapLiteral {
            entries: vec![(
                "a".to_string(),
                Expression::number(1.0, pos.clone()),
                pos.clone(),
            )],
            position: pos.clone(),
        };

        let non_map_err = registry
            .call(
                "map-set",
                &[
                    Expression::number(1.0, pos.clone()),
                    Expression::identifier("k".to_string(), pos.clone()),
                    Expression::number(2.0, pos.clone()),
                ],
                &pos,
            )
            .unwrap_err();
        assert!(matches!(
            non_map_err,
            Error::FunctionError {
                function,
                message,
                ..
            } if function == "map-set" && message.contains("First argument must be a map")
        ));

        let empty_path_err = registry
            .call(
                "map-set",
                &[nested.clone(), Expression::number(2.0, pos.clone())],
                &pos,
            )
            .unwrap_err();
        assert!(matches!(
            empty_path_err,
            Error::FunctionError {
                function,
                message,
                ..
            } if function == "map-set" && message.contains("Expected at least 3 arguments")
        ));

        let intermediate_err = registry
            .call(
                "map-set",
                &[
                    nested.clone(),
                    Expression::identifier("a".to_string(), pos.clone()),
                    Expression::identifier("b".to_string(), pos.clone()),
                    Expression::number(2.0, pos.clone()),
                ],
                &pos,
            )
            .unwrap_err();
        assert!(matches!(
            intermediate_err,
            Error::FunctionError {
                function,
                message,
                ..
            } if function == "map-set" && message.contains("Intermediate key 'a' is not a map")
        ));

        let deep_merge_err = registry
            .call(
                "map-deep-merge",
                &[nested.clone(), Expression::number(2.0, pos.clone())],
                &pos,
            )
            .unwrap_err();
        assert!(matches!(
            deep_merge_err,
            Error::FunctionError {
                function,
                message,
                ..
            } if function == "map-deep-merge" && message.contains("All arguments must be maps")
        ));

        let remove_intermediate_err = registry
            .call(
                "map-remove",
                &[
                    nested.clone(),
                    Expression::identifier("a".to_string(), pos.clone()),
                    Expression::identifier("b".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap_err();
        assert!(matches!(
            remove_intermediate_err,
            Error::FunctionError {
                function,
                message,
                ..
            } if function == "map-remove" && message.contains("Intermediate key 'a' is not a map")
        ));

        let get_intermediate_err = registry
            .call(
                "map-get",
                &[
                    nested.clone(),
                    Expression::identifier("a".to_string(), pos.clone()),
                    Expression::identifier("b".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap_err();
        assert!(matches!(
            get_intermediate_err,
            Error::FunctionError {
                function,
                message,
                ..
            } if function == "map-get" && message.contains("Intermediate key 'a' is not a map")
        ));

        let update_not_found_err = registry
            .call(
                "map-update",
                &[
                    nested.clone(),
                    Expression::identifier("missing".to_string(), pos.clone()),
                    Expression::number(2.0, pos.clone()),
                ],
                &pos,
            )
            .unwrap_err();
        assert!(matches!(
            update_not_found_err,
            Error::FunctionError {
                function,
                message,
                ..
            } if function == "map-update" && message.contains("not found in map")
        ));

        let replace_intermediate_err = registry
            .call(
                "map-replace",
                &[
                    nested.clone(),
                    Expression::identifier("a".to_string(), pos.clone()),
                    Expression::identifier("b".to_string(), pos.clone()),
                    Expression::number(2.0, pos.clone()),
                ],
                &pos,
            )
            .unwrap_err();
        assert!(matches!(
            replace_intermediate_err,
            Error::FunctionError {
                function,
                message,
                ..
            } if function == "map-replace" && message.contains("Intermediate key 'a' is not a map")
        ));

        let update_non_map_err = registry
            .call(
                "map-update",
                &[
                    Expression::number(1.0, pos.clone()),
                    Expression::identifier("k".to_string(), pos.clone()),
                    Expression::number(2.0, pos.clone()),
                ],
                &pos,
            )
            .unwrap_err();
        assert!(matches!(
            update_non_map_err,
            Error::FunctionError {
                function,
                message,
                ..
            } if function == "map-update" && message.contains("First argument must be a map")
        ));

        let deep_remove_non_map_err = registry
            .call(
                "map-deep-remove",
                &[
                    Expression::number(1.0, pos.clone()),
                    Expression::identifier("k".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap_err();
        assert!(matches!(
            deep_remove_non_map_err,
            Error::FunctionError {
                function,
                message,
                ..
            } if function == "map-deep-remove" && message.contains("First argument must be a map")
        ));

        let deep_remove_intermediate_err = registry
            .call(
                "map-deep-remove",
                &[
                    nested.clone(),
                    Expression::identifier("a".to_string(), pos.clone()),
                    Expression::identifier("b".to_string(), pos.clone()),
                ],
                &pos,
            )
            .unwrap_err();
        assert!(matches!(
            deep_remove_intermediate_err,
            Error::FunctionError {
                function,
                message,
                ..
            } if function == "map-deep-remove" && message.contains("Intermediate key 'a' is not a map")
        ));
    }
}
