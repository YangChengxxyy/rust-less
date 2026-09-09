//! LESS 内置函数
//!
//! 此模块提供 LESS 内置函数的实现，用于颜色操作、
//! 数学运算、字符串操作和其他实用功能。

use crate::ast::{Expression, Position, ListSeparator, TemplateStringPart, BinaryOperator, UnaryOperator};
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
        // 数学函数模块
        self.register_math_functions();

        // 颜色函数模块
        self.register_color_functions();

        // 字符串函数模块
        self.register_string_functions();

        // 类型检查函数模块
        self.register_type_check_functions();

        // 高级数学函数模块
        self.register_advanced_math_functions();

        // 变换函数模块
        self.register_transform_functions();

        // URL 函数
        self.register("url", Box::new(url_function));

        // 条件函数
        self.register("if", Box::new(if_function));

        // 列表函数模块
        self.register_list_functions();

        // 单位转换函数
        self.register("convert", Box::new(convert_function));

        // 颜色通道访问函数模块
        self.register_color_channel_functions();

        // 颜色混合函数模块
        self.register_color_blending_functions();
    }

    /// 注册数学函数
    fn register_math_functions(&mut self) {
        self.register("round", Box::new(round_function));
        self.register("ceil", Box::new(ceil_function));
        self.register("floor", Box::new(floor_function));
        self.register("abs", Box::new(abs_function));
        self.register("min", Box::new(min_function));
        self.register("max", Box::new(max_function));
        self.register("percentage", Box::new(percentage_function));
    }

    /// 注册颜色函数
    fn register_color_functions(&mut self) {
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
    }

    /// 注册字符串函数
    fn register_string_functions(&mut self) {
        self.register("e", Box::new(e_function));
        self.register("escape", Box::new(escape_function));
        self.register("replace", Box::new(replace_function));
        self.register("uppercase", Box::new(uppercase_function));
        self.register("lowercase", Box::new(lowercase_function));
        self.register("length", Box::new(length_function));
        self.register("extract", Box::new(extract_function));
    }

    /// 注册类型检查函数
    fn register_type_check_functions(&mut self) {
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
    }

    /// 注册高级数学函数
    fn register_advanced_math_functions(&mut self) {
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
    }

    /// 注册变换函数
    fn register_transform_functions(&mut self) {
        self.register("scale", Box::new(scale_function));
        self.register("translateX", Box::new(translate_x_function));
        // Guard-only function: default() evaluates true in the second
        // matching pass (mixin.rs skips default()-guarded mixins in pass 1).
        self.register("default", Box::new(default_function));
    }

    /// 注册列表函数
    fn register_list_functions(&mut self) {
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
    }

    /// 注册颜色通道访问函数
    fn register_color_channel_functions(&mut self) {
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
    }

    /// 注册颜色混合函数
    fn register_color_blending_functions(&mut self) {
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
    }

    /// 注册自定义函数
    pub fn register<F>(&mut self, name: &str, func: Box<F>)
    where
        F: Fn(&[Expression], &Position) -> Result<Expression> + 'static,
    {
        self.functions.insert(name.to_string(), func);
    }

    /// 注册插件函数（[`crate::plugin::LessFunction`]），与内置函数同一调用路径。
    ///
    /// 插件错误统一包装为 [`crate::Error::PluginError`]，不静默吞掉。
    pub fn register_plugin(&mut self, plugin: Box<dyn crate::plugin::LessFunction>) {
        let name = plugin.name().to_string();
        let plugin_name = name.clone();
        self.functions.insert(
            name,
            Box::new(move |args, position| {
                plugin
                    .call(args, position)
                    .map_err(|e| crate::plugin::wrap_plugin_error(&plugin_name, e))
            }),
        );
    }

    /// Call a function by name
    pub fn call(&self, name: &str, args: &[Expression], position: &Position) -> Result<Expression> {
        if let Some(func) = self.functions.get(name) {
            func(args, position)
        } else {
            Err(Error::undefined_function(name, position.line, position.column))
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
    if args.is_empty() || args.len() > 2 {
        return Err(Error::function_error(
            "round",
            "expected 1 or 2 arguments (value, [decimal_places])",
            position.line,
            position.column,
        ));
    }

    let value = match &args[0] {
        Expression::Number { value, unit, .. } => (value, unit),
        _ => {
            return Err(Error::function_error(
                "round",
                "expected numeric value argument",
                position.line,
                position.column,
            ));
        }
    };

    let decimal_places = if args.len() == 2 {
        match &args[1] {
            Expression::Number { value, .. } => *value as u32,
            _ => {
                return Err(Error::function_error(
                    "round",
                    "expected numeric decimal_places argument",
                    position.line,
                    position.column,
                ));
            }
        }
    } else {
        0
    };

    // less.js: round to specified decimal places (half away from zero)
    let factor = 10.0f64.powi(decimal_places as i32);
    let rounded = (value.0 * factor).round() / factor;

    Ok(Expression::Number {
        value: rounded,
        unit: value.1.clone(),
        position: position.clone(),
    })
}

// default() for mixin guards: evaluates true in the second matching pass.
// The mixin compiler statically skips default()-guarded mixins in pass 1,
// so by the time this is called the default branch should match.
fn default_function(_args: &[Expression], position: &Position) -> Result<Expression> {
    Ok(Expression::Boolean(true, position.clone()))
}

fn ceil_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("ceil", 1, args, position)?;
    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::Number {
            value: value.ceil(),
            unit: unit.clone(),
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "ceil",
            "expected one numeric argument",
            position.line,
            position.column,
        )),
    }
}

fn floor_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("floor", 1, args, position)?;
    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::Number {
            value: value.floor(),
            unit: unit.clone(),
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "floor",
            "expected one numeric argument",
            position.line,
            position.column,
        )),
    }
}

fn abs_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("abs", 1, args, position)?;
    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::Number {
            value: value.abs(),
            unit: unit.clone(),
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "abs",
            "expected one numeric argument",
            position.line,
            position.column,
        )),
    }
}

fn min_function(args: &[Expression], position: &Position) -> Result<Expression> {
    let mut min_val = f64::MAX;
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
                    "all arguments must be numeric",
                    position.line,
                    position.column,
                ));
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
    let mut max_val = f64::MIN;
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
                    "all arguments must be numeric",
                    position.line,
                    position.column,
                ));
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
    ensure_arg_count("percentage", 1, args, position)?;
    match &args[0] {
        Expression::Number { value, .. } => {
            Ok(Expression::Percentage(value * 100.0, position.clone()))
        }
        _ => Err(Error::function_error(
            "percentage",
            "expected one numeric argument",
            position.line,
            position.column,
        )),
    }
}

// Color functions (placeholder implementations)

fn lighten_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("lighten", 2, args, position)?;
    let (r, g, b, a) = expression_to_color(&args[0], position)?;
    let amount = expression_to_percentage(&args[1], position)?;

    let (h, s, l) = rgb_to_hsl(r, g, b);
    let new_l = (l + amount / 100.0).clamp(0.0, 1.0);
    let (new_r, new_g, new_b) = hsl_to_rgb(h, s, new_l);

    Ok(Expression::Color {
        red: new_r,
        green: new_g,
        blue: new_b,
        alpha: a,
        original: None,
        position: position.clone(),
    })
}

fn darken_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("darken", 2, args, position)?;
    let (r, g, b, a) = expression_to_color(&args[0], position)?;
    let amount = expression_to_percentage(&args[1], position)?;

    let (h, s, l) = rgb_to_hsl(r, g, b);
    let new_l = (l - amount / 100.0).clamp(0.0, 1.0);
    let (new_r, new_g, new_b) = hsl_to_rgb(h, s, new_l);

    Ok(Expression::Color {
        red: new_r,
        green: new_g,
        blue: new_b,
        alpha: a,
        original: None,
        position: position.clone(),
    })
}

fn saturate_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("saturate", 2, args, position)?;
    let (r, g, b, a) = expression_to_color(&args[0], position)?;
    let amount = expression_to_percentage(&args[1], position)?;

    let (h, s, l) = rgb_to_hsl(r, g, b);
    let new_s = (s + amount / 100.0).clamp(0.0, 1.0);
    let (new_r, new_g, new_b) = hsl_to_rgb(h, new_s, l);

    Ok(Expression::Color {
        red: new_r,
        green: new_g,
        blue: new_b,
        alpha: a,
        original: None,
        position: position.clone(),
    })
}

fn desaturate_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("desaturate", 2, args, position)?;
    let (r, g, b, a) = expression_to_color(&args[0], position)?;
    let amount = expression_to_percentage(&args[1], position)?;

    let (h, s, l) = rgb_to_hsl(r, g, b);
    let new_s = (s - amount / 100.0).clamp(0.0, 1.0);
    let (new_r, new_g, new_b) = hsl_to_rgb(h, new_s, l);

    Ok(Expression::Color {
        red: new_r,
        green: new_g,
        blue: new_b,
        alpha: a,
        original: None,
        position: position.clone(),
    })
}

fn fade_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("fade", 2, args, position)?;
    let (r, g, b, _) = expression_to_color(&args[0], position)?;
    let amount = expression_to_percentage(&args[1], position)?;

    let new_alpha = (amount / 100.0).clamp(0.0, 1.0);

    Ok(Expression::Color {
        red: r,
        green: g,
        blue: b,
        alpha: new_alpha,
        original: None,
        position: position.clone(),
    })
}

fn fadeout_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("fadeout", 2, args, position)?;
    let (r, g, b, a) = expression_to_color(&args[0], position)?;
    let amount = expression_to_percentage(&args[1], position)?;

    let new_alpha = (a - amount / 100.0).clamp(0.0, 1.0);

    Ok(Expression::Color {
        red: r,
        green: g,
        blue: b,
        alpha: new_alpha,
        original: None,
        position: position.clone(),
    })
}

fn fadein_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("fadein", 2, args, position)?;
    let (r, g, b, a) = expression_to_color(&args[0], position)?;
    let amount = expression_to_percentage(&args[1], position)?;

    let new_alpha = (a + amount / 100.0).clamp(0.0, 1.0);

    Ok(Expression::Color {
        red: r,
        green: g,
        blue: b,
        alpha: new_alpha,
        original: None,
        position: position.clone(),
    })
}

fn spin_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("spin", 2, args, position)?;
    let (r, g, b, a) = expression_to_color(&args[0], position)?;
    let amount = match &args[1] {
        Expression::Number { value, .. } => *value,
        _ => {
            return Err(Error::function_error(
                "spin",
                "expected numeric argument for hue rotation",
                position.line,
                position.column,
            ));
        }
    };

    let (h, s, l) = rgb_to_hsl(r, g, b);
    let new_h = (h + amount).rem_euclid(360.0);
    let (new_r, new_g, new_b) = hsl_to_rgb(new_h, s, l);

    Ok(Expression::Color {
        red: new_r,
        green: new_g,
        blue: new_b,
        alpha: a,
        original: None,
        position: position.clone(),
    })
}

fn mix_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() < 2 || args.len() > 3 {
        return Err(Error::function_error(
            "mix",
            "expected 2 or 3 arguments",
            position.line,
            position.column,
        ));
    }

    let (r1, g1, b1, a1) = expression_to_color(&args[0], position)?;
    let (r2, g2, b2, a2) = expression_to_color(&args[1], position)?;
    let weight = if args.len() == 3 {
        expression_to_percentage(&args[2], position)? / 100.0
    } else {
        0.5 // default 50%
    };

    let w = weight * 2.0 - 1.0;
    let a = a1 - a2;

    let weight_factor = if w * a == -1.0 {
        (w + 1.0) / 2.0
    } else {
        (w + 1.0) / 2.0 * (1.0 + (w * a))
    };
    let w1 = weight_factor;
    let w2 = 1.0 - w1;

    let r = (r1 as f64 * w1 + r2 as f64 * w2).round() as u8;
    let g = (g1 as f64 * w1 + g2 as f64 * w2).round() as u8;
    let b = (b1 as f64 * w1 + b2 as f64 * w2).round() as u8;
    let alpha = a1 * weight + a2 * (1.0 - weight);

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
    ensure_arg_count("rgb", 3, args, position)?;

    let r = match &args[0] {
        Expression::Number { value, .. } => (*value as u8).clamp(0, 255),
        _ => {
            return Err(Error::function_error(
                "rgb",
                "red component must be numeric",
                position.line,
                position.column,
            ));
        }
    };

    let g = match &args[1] {
        Expression::Number { value, .. } => (*value as u8).clamp(0, 255),
        _ => {
            return Err(Error::function_error(
                "rgb",
                "green component must be numeric",
                position.line,
                position.column,
            ));
        }
    };

    let b = match &args[2] {
        Expression::Number { value, .. } => (*value as u8).clamp(0, 255),
        _ => {
            return Err(Error::function_error(
                "rgb",
                "blue component must be numeric",
                position.line,
                position.column,
            ));
        }
    };

    Ok(Expression::Color {
        red: r,
        green: g,
        blue: b,
        alpha: 1.0,
        original: None,
        position: position.clone(),
    })
}

fn rgba_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("rgba", 4, args, position)?;

    let (r, g, b, _) = expression_to_color(&Expression::FunctionCall {
        name: "rgb".to_string(),
        arguments: args[0..3].to_vec(),
        position: position.clone(),
    }, position)?;

    let a = match &args[3] {
        Expression::Number { value, .. } => (*value).clamp(0.0, 1.0),
        _ => {
            return Err(Error::function_error(
                "rgba",
                "alpha component must be numeric",
                position.line,
                position.column,
            ));
        }
    };

    Ok(Expression::Color {
        red: r,
        green: g,
        blue: b,
        alpha: a,
        original: None,
        position: position.clone(),
    })
}

fn hsl_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("hsl", 3, args, position)?;

    let h = match &args[0] {
        Expression::Number { value, .. } => (value % 360.0).abs(),
        _ => {
            return Err(Error::function_error(
                "hsl",
                "hue component must be numeric",
                position.line,
                position.column,
            ));
        }
    };

    let s = expression_to_percentage(&args[1], position)? / 100.0;
    let l = expression_to_percentage(&args[2], position)? / 100.0;

    let (r, g, b) = hsl_to_rgb(h, s, l);

    Ok(Expression::Color {
        red: r,
        green: g,
        blue: b,
        alpha: 1.0,
        original: None,
        position: position.clone(),
    })
}

fn hsla_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("hsla", 4, args, position)?;

    let h = match &args[0] {
        Expression::Number { value, .. } => (value % 360.0).abs(),
        _ => {
            return Err(Error::function_error(
                "hsla",
                "hue component must be numeric",
                position.line,
                position.column,
            ));
        }
    };

    let s = expression_to_percentage(&args[1], position)? / 100.0;
    let l = expression_to_percentage(&args[2], position)? / 100.0;

    let a = match &args[3] {
        Expression::Number { value, .. } => (*value).clamp(0.0, 1.0),
        _ => {
            return Err(Error::function_error(
                "hsla",
                "alpha component must be numeric",
                position.line,
                position.column,
            ));
        }
    };

    let (r, g, b) = hsl_to_rgb(h, s, l);

    Ok(Expression::Color {
        red: r,
        green: g,
        blue: b,
        alpha: a,
        original: None,
        position: position.clone(),
    })
}

// String functions

/// CSS-escape a value: outputs the raw string with quotes stripped.
/// Mirrors less.js `e()`: `new Quoted('"', str.value, true)`.
fn e_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("e", 1, args, position)?;
    let string_val = string_value(&args[0]);

    Ok(Expression::Anonymous(string_val.value, position.clone()))
}

/// Percent-encode a string per less.js `escape()`
/// (tree/functions/string.js):
/// `encodeURI(str.value)` then additionally encoding `= : # ; ( )`.
/// `encodeURI` leaves `A-Za-z0-9 - _ . ! ~ * ' ( ) ; / ? : @ & = + $ , #`
/// unescaped and encodes every other character (incl. space and non-ASCII)
/// as uppercase `%XX` per UTF-8 byte; after the extra replacements the
/// surviving unescaped set is exactly `A-Za-z0-9 - _ . ! ~ * ' / ? @ & + $ ,`.
fn escape_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("escape", 1, args, position)?;
    let input = string_value(&args[0]);
    let value = &input.value;

    let mut encoded = String::new();
    for ch in value.chars() {
        match ch {
            // Characters that encodeURI leaves unescaped...
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '!' | '~' | '*' | '\''
            | '/' | '?' | '@' | '&' | '+' | '$' | ',' => {
                encoded.push(ch);
            }
            // Extra characters that less.js also encodes
            '=' | ':' | '#' | ';' | '(' | ')' | ' ' => {
                for byte in ch.to_string().as_bytes() {
                    encoded.push_str(&format!("%{:02X}", byte));
                }
            }
            // All other characters get percent-encoded as UTF-8 bytes
            _ => {
                for byte in ch.to_string().as_bytes() {
                    encoded.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }

    Ok(Expression::Anonymous(encoded, position.clone()))
}

fn replace_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Error::function_error(
            "replace",
            "expected 2 to 4 arguments",
            position.line,
            position.column,
        ));
    }

    let text = string_value(&args[0]);
    let pattern = string_value(&args[1]);
    let replacement = string_value(&args[2]);
    // Note: flags parameter is parsed but not implemented yet (for regex support)
    let _flags = if args.len() == 4 {
        string_value(&args[3])
    } else {
        StringValue { value: "".to_string(), quoted: true }
    };

    // For now, implement basic string replacement without regex support
    if pattern.value.is_empty() {
        return Err(Error::function_error(
            "replace",
            "pattern must not be empty",
            position.line,
            position.column,
        ));
    }
    let _ = &_flags;
    let result = text.value.replace(&pattern.value, &replacement.value);

    Ok(Expression::String {
        value: result,
        quoted: text.quoted,
        position: position.clone(),
    })
}

/// URL function - creates a URL expression
fn url_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("url", 1, args, position)?;
    let path = string_value(&args[0]);

    Ok(Expression::Url(path.value, position.clone()))
}

struct StringValue {
    value: String,
    quoted: bool,
}

fn ensure_arg_count(
    function: &str,
    expected: usize,
    args: &[Expression],
    position: &Position,
) -> Result<()> {
    if args.len() != expected {
        Err(Error::function_error(
            function,
            format!("expected {} arguments, got {}", expected, args.len()),
            position.line,
            position.column,
        ))
    } else {
        Ok(())
    }
}

fn string_value(expr: &Expression) -> StringValue {
    match expr {
        Expression::String { value, quoted, .. } => StringValue {
            value: value.clone(),
            quoted: *quoted,
        },
        Expression::Anonymous(value, _) => StringValue {
            value: value.clone(),
            quoted: false,
        },
        Expression::Number { value, unit, .. } => {
            let num_str = if let Some(u) = unit {
                format!("{}{}", value, u)
            } else {
                value.to_string()
            };
            StringValue {
                value: num_str,
                quoted: false,
            }
        }
        Expression::Variable(name, _) => StringValue {
            value: format!("@{}", name),
            quoted: false,
        },
        Expression::Color { red, green, blue, alpha, .. } => {
            let color_str = if *alpha < 1.0 {
                format!("#{:02x}{:02x}{:02x}{:02x}", red, green, blue, (*alpha * 255.0) as u8)
            } else {
                format!("#{:02x}{:02x}{:02x}", red, green, blue)
            };
            StringValue {
                value: color_str,
                quoted: false,
            }
        }
        Expression::Percentage(value, _) => StringValue {
            value: format!("{}%", value),
            quoted: false,
        },
        Expression::Boolean(b, _) => StringValue {
            value: b.to_string(),
            quoted: false,
        },
        Expression::List { values, separator, .. } => {
            let sep = match separator {
                ListSeparator::Comma => ", ",
                ListSeparator::Space => " ",
                ListSeparator::Semicolon => "; ",
            };
            let value = values
                .iter()
                .map(|v| string_value(v).value)
                .collect::<Vec<_>>()
                .join(sep);
            StringValue { value, quoted: false }
        }
        Expression::MapLiteral { .. } => StringValue {
            value: "[object Object]".to_string(),
            quoted: false,
        },
        Expression::MapAccess { .. } => StringValue {
            value: "[object Object]".to_string(),
            quoted: false,
        },
        Expression::DetachedRuleset { .. } => StringValue {
            value: "[ruleset]".to_string(),
            quoted: false,
        },
        Expression::TemplateString { parts, .. } => {
            let mut result = String::new();
            for part in parts {
                match part {
                    TemplateStringPart::Text(text) => result.push_str(text),
                    TemplateStringPart::Interpolation(name) => {
                        result.push_str(&format!("@{{{}}}", name));
                    }
                }
            }
            StringValue { value: result, quoted: false }
        }
        Expression::Url(value, _) => StringValue {
            value: format!("url({})", value),
            quoted: false,
        },
        Expression::FunctionCall { name, arguments, .. } => {
            // For functions, evaluate them to their string representation
            if *name == "e" {
                if let Some(arg) = arguments.first() {
                    StringValue {
                        value: string_value(arg).value,
                        quoted: false,
                    }
                } else {
                    StringValue { value: String::new(), quoted: false }
                }
            } else {
                StringValue {
                    value: format!("{}({})", name, arguments.len()),
                    quoted: false,
                }
            }
        }
        Expression::BinaryOp { left, operator, right, .. } => {
            let left_str = string_value(left).value;
            let right_str = string_value(right).value;
            let op_str = match operator {
                BinaryOperator::Add => "+",
                BinaryOperator::Subtract => "-",
                BinaryOperator::Multiply => "*",
                BinaryOperator::Divide => "/",
                BinaryOperator::Modulo => "%",
                BinaryOperator::Equal => "==",
                BinaryOperator::NotEqual => "!=",
                BinaryOperator::GreaterThan => ">",
                BinaryOperator::GreaterThanOrEqual => ">=",
                BinaryOperator::LessThan => "<",
                BinaryOperator::LessThanOrEqual => "<=",
                BinaryOperator::And => "and",
                BinaryOperator::Or => "or",
                BinaryOperator::Concatenate => "~",
            };
            StringValue {
                value: format!("{} {} {}", left_str, op_str, right_str),
                quoted: false,
            }
        }
        Expression::UnaryOp { operator, operand, .. } => {
            let op_str = match operator {
                UnaryOperator::Minus => "-",
                UnaryOperator::Plus => "+",
                UnaryOperator::Not => "not ",
            };
            StringValue {
                value: format!("{}{}", op_str, string_value(operand).value),
                quoted: false,
            }
        }
        Expression::Parenthesized(inner, _) => StringValue {
            value: format!("({})", string_value(inner).value),
            quoted: false,
        },
        Expression::Interpolation(name, _) => StringValue {
            value: format!("@{{{}}}", name),
            quoted: false,
        },
        Expression::Null(_) => StringValue {
            value: "null".to_string(),
            quoted: false,
        },
        // Handle less common expression types
        Expression::PropertyInterpolation(name, _) => StringValue {
            value: format!("@{{{}}}", name),
            quoted: false,
        },
        Expression::SelectorInterpolation(name, _) => StringValue {
            value: format!("@{{{}}}", name),
            quoted: false,
        },
        Expression::Dimension { value, from_unit, .. } => StringValue {
            value: format!("{}{}", value, from_unit),
            quoted: false,
        },
        _ => StringValue {
            value: String::new(),
            quoted: false,
        },
    }
}
fn map_key_string(expr: &Expression) -> String {
    match expr {
        Expression::String { value, quoted, .. } => {
            if *quoted {
                format!("\"{}\"", value)
            } else {
                value.clone()
            }
        }
        Expression::Variable(name, _) => format!("@{}", name),
        Expression::Number { value, unit, .. } => {
            if let Some(u) = unit {
                format!("{}{}", value, u)
            } else {
                value.to_string()
            }
        }
        _ => string_value(expr).value,
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
    let mut current_key = None;

    for (i, key) in path.iter().enumerate() {
        let canonical_key = canonical_map_key(key);
        // Use reverse iteration to implement last-wins semantics for duplicate keys
        let found = current_entries
            .iter()
            .rev()
            .find(|(k, _, _)| map_keys_equal(k, canonical_key));

        match found {
            Some((_, value, _)) => {
                if i == path.len() - 1 {
                    // Last key in path, return the value
                    return Ok(Some(value));
                } else {
                    // Not last key, need to traverse deeper
                    match value {
                        Expression::MapLiteral { entries: sub_entries, .. } => {
                            current_entries = sub_entries;
                            current_key = Some(key);
                            continue
                        }
                        _ => {
                            return Err(format!(
                                "Cannot traverse into non-map value at key '{}'",
                                key
                            ));
                        }
                    }
                }
            }
            None => {
                if let Some(key) = current_key {
                    return Err(format!("Key '{}' not found in nested map", key));
                } else {
                    return Ok(None);
                }
            }
        }
    }

    Ok(None)
}

enum RemoveMapPathResult {
    Ok(Vec<(String, Expression, Position)>),
    KeyNotFound(String),
    #[allow(dead_code)]
    NotMap(String),
}

fn remove_map_path(
    entries: &mut Vec<(String, Expression, Position)>,
    path: &[String],
) -> RemoveMapPathResult {
    if path.is_empty() {
        return RemoveMapPathResult::Ok(Vec::new());
    }

    if path.len() == 1 {
        // Remove the key from current level
        let key = &path[0];
        let canonical_key = canonical_map_key(key);
        let initial_len = entries.len();
        entries.retain(|(k, _, _)| !map_keys_equal(k, canonical_key));

        if entries.len() == initial_len {
            RemoveMapPathResult::KeyNotFound(key.clone())
        } else {
            RemoveMapPathResult::Ok(std::mem::take(entries))
        }
    } else {
        // Navigate to parent of the key to remove
        let last_key = path.last().unwrap();
        let parent_path = &path[..path.len() - 1];

        match map_lookup_path_mut(entries, parent_path) {
            Ok(parent_entries) => {
                let canonical_last_key = canonical_map_key(last_key);
                let initial_len = parent_entries.len();
                parent_entries.retain(|(k, _, _)| !map_keys_equal(k, canonical_last_key));

                if parent_entries.len() == initial_len {
                    RemoveMapPathResult::KeyNotFound(last_key.clone())
                } else {
                    RemoveMapPathResult::Ok(std::mem::take(entries))
                }
            }
            Err(_) => {
                // Path doesn't exist, nothing to remove
                RemoveMapPathResult::KeyNotFound(last_key.clone())
            }
        }
    }
}

fn map_lookup_path_mut<'a>(
    entries: &'a mut [(String, Expression, Position)],
    path: &[String],
) -> std::result::Result<&'a mut Vec<(String, Expression, Position)>, String> {
    let mut current_entries = entries;

    for (i, key) in path.iter().enumerate() {
        let canonical_key = canonical_map_key(key);
        let pos = current_entries
            .iter()
            .position(|(k, _, _)| map_keys_equal(k, canonical_key));

        match pos {
            Some(p) => {
                if i == path.len() - 1 {
                    // Last key, return reference to the entries vector
                    match &mut current_entries[p] {
                        (_, Expression::MapLiteral { entries: sub_entries, .. }, _) => {
                            return Ok(sub_entries);
                        }
                        _ => {
                            return Err(format!(
                                "Cannot traverse into non-map value at key '{}'",
                                key
                            ));
                        }
                    }
                } else {
                    // Not last key, need to traverse deeper
                    match &mut current_entries[p] {
                        (_, Expression::MapLiteral { entries: sub_entries, .. }, _) => {
                            current_entries = sub_entries;
                        }
                        _ => {
                            return Err(format!(
                                "Cannot traverse into non-map value at key '{}'",
                                key
                            ));
                        }
                    }
                }
            }
            None => {
                return Err(format!("Key '{}' not found in map", key));
            }
        }
    }

    Err("Empty path provided".to_string())
}

fn set_map_path(
    entries: &mut Vec<(String, Expression, Position)>,
    path: &[String],
    value: &Expression,
    position: &Position,
) -> std::result::Result<(), String> {
    if path.is_empty() {
        return Err("Empty path provided".to_string());
    }

    let key = &path[0];
    let canonical_key = canonical_map_key(key);

    if path.len() == 1 {
        // Set the final key - use reverse iteration to handle duplicates (last wins)
        if let Some(existing) = entries.iter_mut().rev().find(|(k, _, _)| map_keys_equal(k, canonical_key)) {
            existing.1 = value.clone();
            existing.2 = position.clone();
        } else {
            entries.push((key.clone(), value.clone(), position.clone()));
        }
        Ok(())
    } else {
        // Navigate deeper
        match map_lookup_path_mut(entries, &path[..path.len() - 1]) {
            Ok(parent_entries) => {
                let last_key = &path[path.len() - 1];
                let canonical_last_key = canonical_map_key(last_key);

                if let Some(existing) = parent_entries.iter_mut().rev().find(|(k, _, _)| map_keys_equal(k, canonical_last_key)) {
                    // For existing keys at the target depth, update the value directly
                    existing.1 = value.clone();
                    existing.2 = position.clone();
                    Ok(())
                } else {
                    // Key doesn't exist, create new entry
                    parent_entries.push((
                        last_key.clone(),
                        value.clone(),
                        position.clone(),
                    ));
                    Ok(())
                }
            }
            Err(e) => {
                // Distinguish "intermediate key is not a map" (error) from
                // "key not found" (create the path).
                if e.contains("Cannot traverse into non-map value") {
                    return Err(format!("Intermediate key '{}' is not a map", path[0]));
                }
                // Parent path doesn't exist, create the full path
                if path.len() == 1 {
                    entries.push((key.clone(), value.clone(), position.clone()));
                    Ok(())
                } else {
                    // Create the intermediate path structure using recursion
                    let mut new_map = Vec::new();
                    set_map_path(&mut new_map, &path[1..], value, position)?;

                    entries.push((
                        key.clone(),
                        Expression::MapLiteral {
                            entries: new_map,
                            position: position.clone(),
                        },
                        position.clone(),
                    ));
                    Ok(())
                }
            }
        }
    }
}

fn update_map_path(
    entries: &mut [(String, Expression, Position)],
    path: &[String],
    value: &Expression,
) -> std::result::Result<bool, String> {
    if path.is_empty() {
        return Err("Empty path provided".to_string());
    }

    let key = &path[0];
    let canonical_key = canonical_map_key(key);

    if let Some(existing) = entries.iter_mut().rev().find(|(k, _, _)| map_keys_equal(k, canonical_key)) {
        if path.len() == 1 {
            // Update the value
            existing.1 = value.clone();
            Ok(true)
        } else {
            // Navigate deeper
            match &mut existing.1 {
                Expression::MapLiteral { entries: sub_entries, .. } => {
                    update_map_path(sub_entries, &path[1..], value)
                }
                _ => Err(format!("Intermediate key '{}' is not a map", canonical_key)),
            }
        }
    } else {
        Ok(false) // Key not found, but this is not an error for update
    }
}

// String utility functions

/// Convert an expression to a color, handling hex strings and existing colors
fn expression_to_color(expr: &Expression, position: &Position) -> Result<(u8, u8, u8, f64)> {
    match expr {
        Expression::Color {
            red, green, blue, alpha, ..
        } => Ok((*red, *green, *blue, *alpha)),
        Expression::String { value, quoted, .. } if *quoted => {
            // Handle hex color strings
            if let Ok(color) = crate::ast::Color::from_hex(value, position.clone()) {
                Ok((color.red, color.green, color.blue, color.alpha))
            } else {
                Err(Error::function_error(
                    "color",
                    format!("invalid color string: {}", value),
                    position.line,
                    position.column,
                ))
            }
        }
        Expression::Anonymous(value, _) => {
            // Try to parse as hex color
            if let Ok(color) = crate::ast::Color::from_hex(value, position.clone()) {
                Ok((color.red, color.green, color.blue, color.alpha))
            } else {
                Err(Error::function_error(
                    "color",
                    format!("invalid color string: {}", value),
                    position.line,
                    position.column,
                ))
            }
        }
        Expression::FunctionCall { .. } => {
            // Functions should be evaluated before converting to colors
            // In the compiler, function calls are evaluated first via evaluate_expression
            // This fallback handles cases where direct conversion is attempted
            Err(Error::function_error(
                "color",
                "functions must be evaluated to colors first",
                position.line,
                position.column,
            ))
        }
        _ => Err(Error::function_error(
            "color",
            "expected color or color string",
            position.line,
            position.column,
        )),
    }
}

/// Convert an expression to a percentage value
fn expression_to_percentage(expr: &Expression, position: &Position) -> Result<f64> {
    match expr {
        Expression::Percentage(value, _) => Ok(*value),
        Expression::Number { value, .. } => Ok(*value),
        _ => Err(Error::function_error(
            "percentage",
            "expected numeric or percentage value",
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

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if max == min {
        (0.0, 0.0, l)
    } else {
        let d = max - min;
        let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };

        let h = if max == r {
            (g - b) / d + (if g < b { 6.0 } else { 0.0 })
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };

        (h * 60.0, s, l)
    }
}

/// Convert HSL to RGB color space
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let h = h / 360.0;
    let h = h.rem_euclid(1.0);

    if s == 0.0 {
        let gray = (l * 255.0).round() as u8;
        (gray, gray, gray)
    } else {
        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;

        let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
        let g = hue_to_rgb(p, q, h);
        let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

        (
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8,
        )
    }
}

fn hue_to_rgb(p: f64, q: f64, t: f64) -> f64 {
    let mut t = t;
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }

    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

// String functions

fn uppercase_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("uppercase", 1, args, position)?;
    let s = string_value(&args[0]);
    Ok(Expression::String {
        value: s.value.to_uppercase(),
        quoted: s.quoted,
        position: position.clone(),
    })
}

fn lowercase_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("lowercase", 1, args, position)?;
    let s = string_value(&args[0]);
    Ok(Expression::String {
        value: s.value.to_lowercase(),
        quoted: s.quoted,
        position: position.clone(),
    })
}

fn length_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("length", 1, args, position)?;
    let s = string_value(&args[0]);
    Ok(Expression::Number {
        value: s.value.chars().count() as f64,
        unit: None,
        position: position.clone(),
    })
}

fn extract_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("extract", 2, args, position)?;
    let s = string_value(&args[0]);
    let index = match &args[1] {
        Expression::Number { value, .. } => *value as usize,
        _ => {
            return Err(Error::function_error(
                "extract",
                "expected numeric index",
                position.line,
                position.column,
            ));
        }
    };

    let chars: Vec<char> = s.value.chars().collect();
    if index == 0 || index > chars.len() {
        return Err(Error::function_error(
            "extract",
            "index out of bounds",
            position.line,
            position.column,
        ));
    }

    let char_at = chars[index - 1].to_string();
    Ok(Expression::String {
        value: char_at,
        quoted: s.quoted,
        position: position.clone(),
    })
}

// Map functions

fn map_get_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count_at_least("map-get", 2, args, position)?;
    let entries = match &args[0] {
        Expression::MapLiteral { entries, .. } => entries,
        _ => {
            return Err(Error::function_error(
                "map-get",
                "expected map literal",
                position.line,
                position.column,
            ));
        }
    };

    let key_path = map_path_keys(args, 1);
    match map_lookup_path(entries, &key_path) {
        Ok(Some(value)) => Ok(value.clone()),
        Ok(None) => {
            // Double-check with reverse iteration for duplicate key handling
            // (LESS behavior: last duplicate key wins)
            if !key_path.is_empty() {
                let first_key = &key_path[0];
                let canonical_first = canonical_map_key(first_key);
                
                if let Some((_, value, _)) = entries.iter().rev().find(|(k, _, _)| map_keys_equal(k, canonical_first)) {
                    return Ok(value.clone());
                }
            }
            
            Err(Error::function_error(
                "map-get",
                format!("key '{}' not found in map", key_path.join(".")),
                position.line,
                position.column,
            ))
        },
        Err(msg) => Err(Error::function_error(
            "map-get",
            msg,
            position.line,
            position.column,
        )),
    }
}

fn map_has_key_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count_at_least("map-has-key", 2, args, position)?;
    let entries = match &args[0] {
        Expression::MapLiteral { entries, .. } => entries,
        _ => {
            return Err(Error::function_error(
                "map-has-key",
                "expected map literal",
                position.line,
                position.column,
            ));
        }
    };

    let key_path = map_path_keys(args, 1);
    match map_lookup_path(entries, &key_path) {
        Ok(Some(_)) => Ok(Expression::Boolean(true, position.clone())),
        Ok(None) => Ok(Expression::Boolean(false, position.clone())),
        Err(_) => Ok(Expression::Boolean(false, position.clone())),
    }
}

fn map_keys_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("map-keys", 1, args, position)?;
    let entries = match &args[0] {
        Expression::MapLiteral { entries, .. } => entries,
        _ => {
            return Err(Error::function_error(
                "map-keys",
                "expected map literal",
                position.line,
                position.column,
            ));
        }
    };

    let keys: Vec<Expression> = entries
        .iter()
        .map(|(key, _, _)| Expression::String {
            value: key.clone(),
            quoted: false,
            position: position.clone(),
        })
        .collect();

    Ok(Expression::List {
        values: keys,
        separator: ListSeparator::Comma,
        position: position.clone(),
    })
}

fn map_values_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("map-values", 1, args, position)?;
    let entries = match &args[0] {
        Expression::MapLiteral { entries, .. } => entries,
        _ => {
            return Err(Error::function_error(
                "map-values",
                "expected map literal",
                position.line,
                position.column,
            ));
        }
    };

    let values: Vec<Expression> = entries
        .iter()
        .map(|(_, value, _)| value.clone())
        .collect();

    Ok(Expression::List {
        values,
        separator: ListSeparator::Comma,
        position: position.clone(),
    })
}

fn map_merge_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() < 2 {
        return Err(Error::function_error(
            "map-merge",
            "expected at least 2 arguments",
            position.line,
            position.column,
        ));
    }

    let mut result_entries = Vec::new();

    for (i, arg) in args.iter().enumerate() {
        match arg {
            Expression::MapLiteral { entries, .. } => {
                if i == 0 {
                    // First map: copy all entries
                    result_entries = entries.clone();
                } else {
                    // Subsequent maps: shallow merge (last wins)
                    for (key, value, pos) in entries {
                        // Check if key already exists
                        if let Some(existing) = result_entries.iter_mut().find(|(k, _, _)| map_keys_equal(k, key)) {
                            // Overwrite existing value
                            existing.1 = value.clone();
                            existing.2 = pos.clone();
                        } else {
                            // Add new key
                            result_entries.push((key.clone(), value.clone(), pos.clone()));
                        }
                    }
                }
            }
            _ => {
                return Err(Error::function_error(
                    "map-merge",
                    format!("argument {} is not a map", i + 1),
                    position.line,
                    position.column,
                ));
            }
        }
    }

    Ok(Expression::MapLiteral {
        entries: result_entries,
        position: position.clone(),
    })
}

fn map_deep_merge_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() < 2 {
        return Err(Error::function_error(
            "map-deep-merge",
            "expected at least 2 arguments",
            position.line,
            position.column,
        ));
    }

    let mut result_entries = Vec::new();

    for (i, arg) in args.iter().enumerate() {
        match arg {
            Expression::MapLiteral { entries, .. } => {
                if i == 0 {
                    // First map: copy all entries
                    result_entries = entries.clone();
                } else {
                    // Subsequent maps: deep merge (recursively merge nested maps)
                    deep_merge_entries(&mut result_entries, entries);
                }
            }
            _ => {
                return Err(Error::function_error(
                    "map-deep-merge",
                    format!("argument {} is not a map", i + 1),
                    position.line,
                    position.column,
                ));
            }
        }
    }

    Ok(Expression::MapLiteral {
        entries: result_entries,
        position: position.clone(),
    })
}

fn map_set_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count_at_least("map-set", 3, args, position)?;
    let mut map = match &args[0] {
        Expression::MapLiteral { entries, .. } => Expression::MapLiteral {
            entries: entries.clone(),
            position: position.clone(),
        },
        _ => {
            return Err(Error::function_error(
                "map-set",
                "First argument must be a map",
                position.line,
                position.column,
            ));
        }
    };

    if let Expression::MapLiteral { entries, .. } = &mut map {
        // The last argument is the value; keys are args[1..len-1]
        let key_path: Vec<String> = args[1..args.len() - 1].iter().map(map_key_string).collect();
        let value = &args[args.len() - 1];
        if let Err(e) = set_map_path(entries, &key_path, value, position) {
            // Return map-set specific error for missing intermediate maps
            if key_path.len() > 1 && e.contains("not found") {
                return Err(Error::function_error(
                    "map-set",
                    format!("Intermediate key '{}' is not a map", key_path[0]),
                    position.line,
                    position.column,
                ));
            }
            return Err(Error::function_error("map-set", e, position.line, position.column));
        }
    }

    Ok(map)
}

fn map_update_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count_at_least("map-update", 3, args, position)?;
    let mut map = match &args[0] {
        Expression::MapLiteral { entries, .. } => Expression::MapLiteral {
            entries: entries.clone(),
            position: position.clone(),
        },
        _ => {
            return Err(Error::function_error(
                "map-update",
                "First argument must be a map",
                position.line,
                position.column,
            ));
        }
    };

    if let Expression::MapLiteral { entries, .. } = &mut map {
        // The last argument is the value; keys are args[1..len-1]
        let key_path: Vec<String> = args[1..args.len() - 1].iter().map(map_key_string).collect();
        let value = &args[args.len() - 1];
        match update_map_path(entries, &key_path, value) {
            Ok(true) => Ok(map),
            Ok(false) => {
                // Key not found: for map-update, return error with helpful message
                Err(Error::function_error(
                    "map-update",
                    format!("key '{}' not found in map", key_path.join(".")),
                    position.line,
                    position.column,
                ))
            },
            Err(e) => Err(Error::function_error(
                "map-update",
                e,
                position.line,
                position.column,
            )),
        }
    } else {
        Ok(map)
    }
}

fn map_replace_function(args: &[Expression], position: &Position) -> Result<Expression> {
    // map-replace is an alias for map-update with the same semantics
    map_update_function(args, position)
}

fn map_remove_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count_at_least("map-remove", 2, args, position)?;
    let mut map = match &args[0] {
        Expression::MapLiteral { entries, .. } => Expression::MapLiteral {
            entries: entries.clone(),
            position: position.clone(),
        },
        _ => {
            return Err(Error::function_error(
                "map-remove",
                "First argument must be a map",
                position.line,
                position.column,
            ));
        }
    };

    if let Expression::MapLiteral { entries, .. } = &mut map {
        let key_path = map_path_keys(args, 1);
        match remove_map_path(entries, &key_path) {
            RemoveMapPathResult::Ok(new_entries) => {
                *entries = new_entries;
                Ok(map)
            }
            RemoveMapPathResult::KeyNotFound(_key) => {
                // If key not found, return original map unchanged (LESS behavior)
                Ok(map)
            },
            RemoveMapPathResult::NotMap(key) => Err(Error::function_error(
                "map-remove",
                format!("cannot traverse into non-map value at '{}'", key),
                position.line,
                position.column,
            )),
        }
    } else {
        Ok(map)
    }
}

fn map_deep_remove_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count_at_least("map-deep-remove", 2, args, position)?;
    let mut map = match &args[0] {
        Expression::MapLiteral { entries, .. } => Expression::MapLiteral {
            entries: entries.clone(),
            position: position.clone(),
        },
        _ => {
            return Err(Error::function_error(
                "map-deep-remove",
                "First argument must be a map",
                position.line,
                position.column,
            ));
        }
    };

    if let Expression::MapLiteral { entries, .. } = &mut map {
        let key_path = map_path_keys(args, 1);
        // Validate that every intermediate key along the path is a map
        {
            let mut current: &[(_, _, _)] = entries;
            for key in &key_path[..key_path.len() - 1] {
                let ck = canonical_map_key(key);
                match current.iter().rev().find(|(k, _, _)| map_keys_equal(k, ck)) {
                    Some((_, Expression::MapLiteral { entries: sub, .. }, _)) => current = sub,
                    Some(_) => {
                        return Err(Error::function_error(
                            "map-deep-remove",
                            format!("Intermediate key '{}' is not a map", key),
                            position.line,
                            position.column,
                        ));
                    }
                    None => break,
                }
            }
        }
        match remove_map_path(entries, &key_path) {
            RemoveMapPathResult::Ok(new_entries) => {
                *entries = new_entries;
                // Prune now-empty ancestor maps along the removed path
                prune_empty_path(entries, &key_path[..key_path.len() - 1]);
                Ok(map)
            }
            RemoveMapPathResult::KeyNotFound(_key) => {
                // If key not found, return original map unchanged (LESS behavior)
                Ok(map)
            },
            RemoveMapPathResult::NotMap(key) => Err(Error::function_error(
                "map-deep-remove",
                format!("Intermediate key '{}' is not a map", key),
                position.line,
                position.column,
            )),
        }
    } else {
        Ok(map)
    }
}

/// Recursively remove maps along `path` that became empty after a deep remove.
/// Only prunes ancestors that hold no entries after their child was removed.
fn prune_empty_path(entries: &mut Vec<(String, Expression, Position)>, path: &[String]) {
    if path.is_empty() {
        return;
    }
    let key = &path[0];
    let canonical_key = canonical_map_key(key);
    if let Some(pos) = entries.iter().position(|(k, _, _)| map_keys_equal(k, canonical_key)) {
        if path.len() == 1 {
            // Deepest ancestor: drop it if now empty
            if let Expression::MapLiteral { entries: sub, .. } = &entries[pos].1 {
                if sub.is_empty() {
                    entries.remove(pos);
                }
            }
        } else if let Expression::MapLiteral { entries: sub, .. } = &mut entries[pos].1 {
            prune_empty_path(sub, &path[1..]);
            // After pruning deeper levels, drop this level too if empty
            if sub.is_empty() {
                entries.remove(pos);
            }
        }
    }
}

// Helper functions for map operations

fn ensure_arg_count_at_least(
    function: &str,
    min: usize,
    args: &[Expression],
    position: &Position,
) -> Result<()> {
    if args.len() < min {
        Err(Error::function_error(
            function,
            format!("Expected at least {} arguments, got {}", min, args.len()),
            position.line,
            position.column,
        ))
    } else {
        Ok(())
    }
}

fn deep_merge_entries(
    target: &mut Vec<(String, Expression, Position)>,
    source: &[(String, Expression, Position)],
) {
    for (key, value, pos) in source {
        if let Some(existing) = target.iter_mut().find(|(k, _, _)| map_keys_equal(k, key)) {
            // Key exists, check if both values are maps and recursively merge
            match (&mut existing.1, value) {
                (
                    Expression::MapLiteral { entries: existing_entries, .. },
                    Expression::MapLiteral { entries: source_entries, .. },
                ) => {
                    deep_merge_entries(existing_entries, source_entries);
                }
                _ => {
                    // Not both maps, overwrite with new value
                    existing.1 = value.clone();
                }
            }
        } else {
            // Key doesn't exist, add it
            target.push((key.clone(), value.clone(), pos.clone()));
        }
    }
}

// Type check functions

fn isnumber_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("isnumber", 1, args, position)?;
    let result = matches!(args[0], Expression::Number { .. });
    Ok(Expression::Boolean(result, position.clone()))
}

fn iscolor_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("iscolor", 1, args, position)?;
    let result = matches!(args[0], Expression::Color { .. });
    Ok(Expression::Boolean(result, position.clone()))
}

fn isstring_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("isstring", 1, args, position)?;
    let result = matches!(&args[0], Expression::String { quoted, .. } if *quoted);
    Ok(Expression::Boolean(result, position.clone()))
}

fn iskeyword_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("iskeyword", 1, args, position)?;
    let result = matches!(&args[0], Expression::String { quoted, .. } if !quoted);
    Ok(Expression::Boolean(result, position.clone()))
}

fn isurl_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("isurl", 1, args, position)?;
    let result = matches!(args[0], Expression::Url(..));
    Ok(Expression::Boolean(result, position.clone()))
}

fn ispixel_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("ispixel", 1, args, position)?;
    let result = matches!(
        &args[0],
        Expression::Number {
            value: _,
            unit: Some(unit),
            ..
        } if unit == "px"
    );
    Ok(Expression::Boolean(result, position.clone()))
}

fn isem_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("isem", 1, args, position)?;
    let result = matches!(
        &args[0],
        Expression::Number {
            value: _,
            unit: Some(unit),
            ..
        } if unit == "em"
    );
    Ok(Expression::Boolean(result, position.clone()))
}

fn ispercentage_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("ispercentage", 1, args, position)?;
    let result = matches!(args[0], Expression::Percentage(..));
    Ok(Expression::Boolean(result, position.clone()))
}

fn unit_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("unit", 1, args, position)?;
    match &args[0] {
        Expression::Number { unit, .. } => Ok(Expression::String {
            value: unit.clone().unwrap_or_default(),
            quoted: false,
            position: position.clone(),
        }),
        _ => Ok(Expression::String {
            value: String::new(),
            quoted: false,
            position: position.clone(),
        }),
    }
}

fn get_unit_function(args: &[Expression], position: &Position) -> Result<Expression> {
    unit_function(args, position)
}

// Advanced math functions

fn sqrt_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("sqrt", 1, args, position)?;
    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::Number {
            value: value.sqrt(),
            unit: unit.clone(),
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "sqrt",
            "expected numeric argument",
            position.line,
            position.column,
        )),
    }
}

fn sin_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("sin", 1, args, position)?;
    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::Number {
            value: value.sin(),
            unit: unit.clone(),
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "sin",
            "expected numeric argument",
            position.line,
            position.column,
        )),
    }
}

fn cos_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("cos", 1, args, position)?;
    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::Number {
            value: value.cos(),
            unit: unit.clone(),
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "cos",
            "expected numeric argument",
            position.line,
            position.column,
        )),
    }
}

fn tan_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("tan", 1, args, position)?;
    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::Number {
            value: value.tan(),
            unit: unit.clone(),
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "tan",
            "expected numeric argument",
            position.line,
            position.column,
        )),
    }
}

fn asin_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("asin", 1, args, position)?;
    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::Number {
            value: value.asin(),
            unit: unit.clone(),
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "asin",
            "expected numeric argument",
            position.line,
            position.column,
        )),
    }
}

fn acos_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("acos", 1, args, position)?;
    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::Number {
            value: value.acos(),
            unit: unit.clone(),
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "acos",
            "expected numeric argument",
            position.line,
            position.column,
        )),
    }
}

fn atan_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("atan", 1, args, position)?;
    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::Number {
            value: value.atan(),
            unit: unit.clone(),
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "atan",
            "expected numeric argument",
            position.line,
            position.column,
        )),
    }
}

fn pow_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("pow", 2, args, position)?;
    let base = match &args[0] {
        Expression::Number { value, .. } => *value,
        _ => {
            return Err(Error::function_error(
                "pow",
                "expected numeric base argument",
                position.line,
                position.column,
            ));
        }
    };

    let exponent = match &args[1] {
        Expression::Number { value, .. } => *value,
        _ => {
            return Err(Error::function_error(
                "pow",
                "expected numeric exponent argument",
                position.line,
                position.column,
            ));
        }
    };

    Ok(Expression::Number {
        value: base.powf(exponent),
        unit: None,
        position: position.clone(),
    })
}

fn pi_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("pi", 0, args, position)?;
    Ok(Expression::Number {
        value: std::f64::consts::PI,
        unit: None,
        position: position.clone(),
    })
}

fn mod_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("mod", 2, args, position)?;
    let dividend = match &args[0] {
        Expression::Number { value, unit, .. } => (value, unit),
        _ => {
            return Err(Error::function_error(
                "mod",
                "expected numeric dividend argument",
                position.line,
                position.column,
            ));
        }
    };

    let divisor = match &args[1] {
        Expression::Number { value, .. } => *value,
        _ => {
            return Err(Error::function_error(
                "mod",
                "expected numeric divisor argument",
                position.line,
                position.column,
            ));
        }
    };

    if divisor == 0.0 {
        return Err(Error::division_by_zero(position.line, position.column));
    }

    Ok(Expression::Number {
        value: dividend.0 % divisor,
        unit: dividend.1.clone(),
        position: position.clone(),
    })
}

// Transform functions

fn scale_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.is_empty() || args.len() > 2 {
        return Err(Error::function_error(
            "scale",
            "expected 1 or 2 arguments",
            position.line,
            position.column,
        ));
    }

    let x = match &args[0] {
        Expression::Number { value, .. } => *value,
        _ => {
            return Err(Error::function_error(
                "scale",
                "expected numeric scale factor",
                position.line,
                position.column,
            ));
        }
    };

    let y = if args.len() == 2 {
        match &args[1] {
            Expression::Number { value, .. } => *value,
            _ => {
                return Err(Error::function_error(
                    "scale",
                    "expected numeric scale factor",
                    position.line,
                    position.column,
                ));
            }
        }
    } else {
        x
    };

    Ok(Expression::FunctionCall {
        name: "scale".to_string(),
        arguments: vec![
            Expression::Number {
                value: x,
                unit: None,
                position: position.clone(),
            },
            Expression::Number {
                value: y,
                unit: None,
                position: position.clone(),
            },
        ],
        position: position.clone(),
    })
}

fn translate_x_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("translateX", 1, args, position)?;
    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::FunctionCall {
            name: "translateX".to_string(),
            arguments: vec![Expression::Number {
                value: *value,
                unit: unit.clone(),
                position: position.clone(),
            }],
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "translateX",
            "expected numeric argument",
            position.line,
            position.column,
        )),
    }
}

fn translate_y_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("translateY", 1, args, position)?;
    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::FunctionCall {
            name: "translateY".to_string(),
            arguments: vec![Expression::Number {
                value: *value,
                unit: unit.clone(),
                position: position.clone(),
            }],
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "translateY",
            "expected numeric argument",
            position.line,
            position.column,
        )),
    }
}

fn rotate_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("rotate", 1, args, position)?;
    match &args[0] {
        Expression::Number { value, unit, .. } => Ok(Expression::FunctionCall {
            name: "rotate".to_string(),
            arguments: vec![Expression::Number {
                value: *value,
                unit: unit.clone(),
                position: position.clone(),
            }],
            position: position.clone(),
        }),
        _ => Err(Error::function_error(
            "rotate",
            "expected numeric argument",
            position.line,
            position.column,
        )),
    }
}

// Conditional functions

fn if_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() != 3 {
        return Err(Error::function_error(
            "if",
            "expected 3 arguments (condition, true-value, false-value)",
            position.line,
            position.column,
        ));
    }

    let condition = &args[0];
    let if_true = &args[1];
    let if_false = &args[2];

    // Evaluate condition - accept more types than just boolean
    let condition_result = match condition {
        Expression::Boolean(b, _) => *b,
        Expression::Number { value, .. } => *value != 0.0,
        Expression::String { value, .. } => !value.is_empty() && value != "false",
        Expression::Color { .. } => true,
        Expression::Percentage(value, _) => *value != 0.0,
        _ => {
            return Err(Error::function_error(
                "if",
                "condition must be evaluatable to a boolean",
                position.line,
                position.column,
            ));
        }
    };

    if condition_result {
        Ok(if_true.clone())
    } else {
        Ok(if_false.clone())
    }
}

// List functions

fn range_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.len() < 2 || args.len() > 3 {
        return Err(Error::function_error(
            "range",
            "expected 2 or 3 arguments (start, end, [step])",
            position.line,
            position.column,
        ));
    }

    let start = match &args[0] {
        Expression::Number { value, unit, .. } => (*value, unit.clone()),
        _ => {
            return Err(Error::function_error(
                "range",
                "expected numeric start value",
                position.line,
                position.column,
            ));
        }
    };

    let end = match &args[1] {
        Expression::Number { value, .. } => *value,
        _ => {
            return Err(Error::function_error(
                "range",
                "expected numeric end value",
                position.line,
                position.column,
            ));
        }
    };

    let step = if args.len() == 3 {
        match &args[2] {
            Expression::Number { value, .. } => *value,
            _ => {
                return Err(Error::function_error(
                    "range",
                    "expected numeric step value",
                    position.line,
                    position.column,
                ));
            }
        }
    } else {
        1.0
    };

    if step == 0.0 {
        return Err(Error::function_error(
            "range",
            "step cannot be zero",
            position.line,
            position.column,
        ));
    }

    let mut values = Vec::new();
    let mut current = start.0;
    let ascending = step > 0.0;

    loop {
        if (ascending && current > end) || (!ascending && current < end) {
            break;
        }
        values.push(Expression::Number {
            value: current,
            unit: start.1.clone(),
            position: position.clone(),
        });

        current += step;
    }

    Ok(Expression::List {
        values,
        separator: ListSeparator::Space,
        position: position.clone(),
    })
}

// Unit conversion function

fn convert_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("convert", 2, args, position)?;

    let value = match &args[0] {
        Expression::Number { value, unit, .. } => (value, unit),
        _ => {
            return Err(Error::function_error(
                "convert",
                "expected numeric value",
                position.line,
                position.column,
            ));
        }
    };

    let target_unit = match &args[1] {
        Expression::String { value, .. } => value,
        _ => {
            return Err(Error::function_error(
                "convert",
                "expected string target unit",
                position.line,
                position.column,
            ));
        }
    };

    match &value.1 {
        Some(source_unit) => {
            if let Some(converted_value) = convert_units(*value.0, source_unit, target_unit) {
                Ok(Expression::Number {
                    value: converted_value,
                    unit: Some(target_unit.clone()),
                    position: position.clone(),
                })
            } else {
                Err(Error::function_error(
                    "convert",
                    format!("cannot convert from {} to {}", source_unit, target_unit),
                    position.line,
                    position.column,
                ))
            }
        }
        None => Err(Error::function_error(
            "convert",
            "cannot convert unitless value",
            position.line,
            position.column,
        )),
    }
}

/// Convert a value between compatible units
fn convert_units(value: f64, from: &str, to: &str) -> Option<f64> {
    // Time conversions
    match (from.to_lowercase().as_str(), to.to_lowercase().as_str()) {
        ("s", "ms") | ("sec", "ms") | ("seconds", "ms") => Some(value * 1000.0),
        ("ms", "s") | ("ms", "sec") | ("milliseconds", "s") => Some(value / 1000.0),

        // Length conversions
        ("px", "pt") | ("px", "points") => Some(value * 0.75),
        ("pt", "px") | ("points", "px") => Some(value / 0.75),
        ("px", "in") | ("px", "inches") => Some(value / 96.0),
        ("in", "px") | ("inches", "px") => Some(value * 96.0),
        ("px", "cm") | ("px", "centimeters") => Some(value / 37.79527559055118),
        ("cm", "px") | ("centimeters", "px") => Some(value * 37.79527559055118),
        ("px", "mm") | ("px", "millimeters") => Some(value / 3.7795275590551185),
        ("mm", "px") | ("millimeters", "px") => Some(value * 3.7795275590551185),

        // Angle conversions
        ("deg", "rad") | ("degrees", "radians") => Some(value * std::f64::consts::PI / 180.0),
        ("rad", "deg") | ("radians", "degrees") => Some(value * 180.0 / std::f64::consts::PI),

        // If same unit, return value unchanged
        _ if from.to_lowercase() == to.to_lowercase() => Some(value),

        // Default: cannot convert
        _ => None,
    }
}

// Color channel access functions

fn red_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("red", 1, args, position)?;
    let (r, _, _, _) = expression_to_color(&args[0], position)?;
    Ok(Expression::Number {
        value: r as f64,
        unit: None,
        position: position.clone(),
    })
}

fn green_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("green", 1, args, position)?;
    let (_, g, _, _) = expression_to_color(&args[0], position)?;
    Ok(Expression::Number {
        value: g as f64,
        unit: None,
        position: position.clone(),
    })
}

fn blue_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("blue", 1, args, position)?;
    let (_, _, b, _) = expression_to_color(&args[0], position)?;
    Ok(Expression::Number {
        value: b as f64,
        unit: None,
        position: position.clone(),
    })
}

fn alpha_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("alpha", 1, args, position)?;
    let (_, _, _, a) = expression_to_color(&args[0], position)?;
    Ok(Expression::Number {
        value: a,
        unit: None,
        position: position.clone(),
    })
}

fn hue_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("hue", 1, args, position)?;
    let (r, g, b, _) = expression_to_color(&args[0], position)?;
    let (h, _, _) = rgb_to_hsl(r, g, b);
    Ok(Expression::Number {
        value: h,
        unit: None,
        position: position.clone(),
    })
}

fn saturation_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("saturation", 1, args, position)?;
    let (r, g, b, _) = expression_to_color(&args[0], position)?;
    let (_, s, _) = rgb_to_hsl(r, g, b);
    Ok(Expression::Percentage(s * 100.0, position.clone()))
}

fn lightness_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("lightness", 1, args, position)?;
    let (r, g, b, _) = expression_to_color(&args[0], position)?;
    let (_, _, l) = rgb_to_hsl(r, g, b);
    Ok(Expression::Percentage(l * 100.0, position.clone()))
}

fn luma_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("luma", 1, args, position)?;
    let (r, g, b, _) = expression_to_color(&args[0], position)?;
    let luma = calculate_luma(r, g, b);
    Ok(Expression::Percentage(luma * 100.0, position.clone()))
}

/// Calculate perceptual luma with gamma correction
fn calculate_luma(r: u8, g: u8, b: u8) -> f64 {
    let linearize = |v: u8| {
        let v = v as f64 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };

    let r_lin = linearize(r);
    let g_lin = linearize(g);
    let b_lin = linearize(b);

    0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin
}

fn argb_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("argb", 1, args, position)?;
    let (r, g, b, a) = expression_to_color(&args[0], position)?;
    let argb = format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        (a * 255.0) as u8, r, g, b
    );
    Ok(Expression::String {
        value: argb,
        quoted: false,
        position: position.clone(),
    })
}

// Color blending functions

/// Apply a per-channel blend operation to two colors
fn blend_colors(
    name: &str,
    args: &[Expression],
    position: &Position,
    blend_fn: fn(f64, f64) -> f64,
) -> Result<Expression> {
    ensure_arg_count(name, 2, args, position)?;

    let (r1, g1, b1, a1) = expression_to_color(&args[0], position)?;
    let (r2, g2, b2, a2) = expression_to_color(&args[1], position)?;

    let blend_channel = |c1: u8, c2: u8| -> u8 {
        (blend_fn(c1 as f64 / 255.0, c2 as f64 / 255.0) * 255.0).round() as u8
    };

    Ok(Expression::Color {
        red: blend_channel(r1, r2),
        green: blend_channel(g1, g2),
        blue: blend_channel(b1, b2),
        alpha: (a1 + a2) / 2.0,
        original: None,
        position: position.clone(),
    })
}

fn multiply_function(args: &[Expression], position: &Position) -> Result<Expression> {
    blend_colors("multiply", args, position, |c1, c2| c1 * c2)
}

fn screen_function(args: &[Expression], position: &Position) -> Result<Expression> {
    blend_colors("screen", args, position, |c1, c2| c1 + c2 - c1 * c2)
}

fn overlay_function(args: &[Expression], position: &Position) -> Result<Expression> {
    blend_colors(
        "overlay",
        args,
        position,
        |c1, c2| {
            if c1 < 0.5 {
                c1 * c2 * 2.0
            } else {
                1.0 - 2.0 * (1.0 - c1) * (1.0 - c2)
            }
        },
    )
}

fn softlight_function(args: &[Expression], position: &Position) -> Result<Expression> {
    blend_colors(
        "softlight",
        args,
        position,
        |c1, c2| {
            let d = if c2 <= 0.25 {
                ((16.0 * c2 - 12.0) * c2 + 4.0) * c2
            } else {
                c2.sqrt()
            };
            if c1 <= 0.5 {
                c1 - (1.0 - 2.0 * c2) * c1 * (1.0 - c1)
            } else {
                c1 + (2.0 * c2 - 1.0) * (d - c1)
            }
        },
    )
}

fn hardlight_function(args: &[Expression], position: &Position) -> Result<Expression> {
    blend_colors(
        "hardlight",
        args,
        position,
        |c1, c2| {
            if c2 <= 0.5 {
                c1 * c2 * 2.0
            } else {
                1.0 - 2.0 * (1.0 - c1) * (1.0 - c2)
            }
        },
    )
}

fn difference_function(args: &[Expression], position: &Position) -> Result<Expression> {
    blend_colors("difference", args, position, |c1, c2| (c1 - c2).abs())
}

fn exclusion_function(args: &[Expression], position: &Position) -> Result<Expression> {
    blend_colors(
        "exclusion",
        args,
        position,
        |c1, c2| c1 + c2 - 2.0 * c1 * c2,
    )
}

fn average_function(args: &[Expression], position: &Position) -> Result<Expression> {
    blend_colors("average", args, position, |c1, c2| (c1 + c2) / 2.0)
}

fn negation_function(args: &[Expression], position: &Position) -> Result<Expression> {
    blend_colors(
        "negation",
        args,
        position,
        |c1, c2| 1.0 - (1.0 - c1).abs() - (1.0 - c2).abs(),
    )
}

// Convenience color functions

fn tint_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("tint", 2, args, position)?;
    let color = expression_to_color(&args[0], position)?;
    let amount = expression_to_percentage(&args[1], position)?;

    // Mix with white
    let white = (255u8, 255u8, 255u8, color.3);
    blend_colors(
        "tint",
        &[
            Expression::Color {
                red: color.0,
                green: color.1,
                blue: color.2,
                alpha: color.3,
                original: None,
                position: position.clone(),
            },
            Expression::Color {
                red: white.0,
                green: white.1,
                blue: white.2,
                alpha: white.3,
                original: None,
                position: position.clone(),
            },
            Expression::Percentage(amount, position.clone()),
        ],
        position,
        |c1, c2| c1 + (1.0 - c1) * c2,
    )
}

fn shade_function(args: &[Expression], position: &Position) -> Result<Expression> {
    ensure_arg_count("shade", 2, args, position)?;
    let color = expression_to_color(&args[0], position)?;
    let amount = expression_to_percentage(&args[1], position)?;

    // Mix with black
    let black = (0u8, 0u8, 0u8, color.3);
    blend_colors(
        "shade",
        &[
            Expression::Color {
                red: color.0,
                green: color.1,
                blue: color.2,
                alpha: color.3,
                original: None,
                position: position.clone(),
            },
            Expression::Color {
                red: black.0,
                green: black.1,
                blue: black.2,
                alpha: black.3,
                original: None,
                position: position.clone(),
            },
            Expression::Percentage(amount, position.clone()),
        ],
        position,
        |c1, c2| c1 * (1.0 - c2),
    )
}

fn contrast_function(args: &[Expression], position: &Position) -> Result<Expression> {
    if args.is_empty() || args.len() > 3 {
        return Err(Error::function_error(
            "contrast",
            "expected 1 to 3 arguments (color, [dark], [light])",
            position.line,
            position.column,
        ));
    }

    let (r, g, b, a) = expression_to_color(&args[0], position)?;
    let luma = calculate_luma(r, g, b);

    let dark = if args.len() >= 2 {
        expression_to_color(&args[1], position)?
    } else {
        (0, 0, 0, 1.0) // black
    };

    let light = if args.len() >= 3 {
        expression_to_color(&args[2], position)?
    } else {
        (255, 255, 255, 1.0) // white
    };

    let result = if luma < 0.5 { light } else { dark };

    Ok(Expression::Color {
        red: result.0,
        green: result.1,
        blue: result.2,
        alpha: a,
        original: None,
        position: position.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a test position
    fn pos() -> Position {
        Position::new(1, 1)
    }

    // Helper function to create a map literal for testing
    fn map_literal(entries: Vec<(&str, Expression)>) -> Expression {
        Expression::MapLiteral {
            entries: entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v, pos()))
                .collect(),
            position: pos(),
        }
    }

    // Basic test to ensure the registry works
    #[test]
    fn test_round_function() {
        let registry = FunctionRegistry::new();
        let args = vec![Expression::Number {
            value: 10.6,
            unit: Some("px".to_string()),
            position: pos(),
        }];

        let result = registry.call("round", &args, &pos()).unwrap();

        if let Expression::Number { value, unit, .. } = result {
            assert_eq!(value, 11.0);
            assert_eq!(unit, Some("px".to_string()));
        } else {
            panic!("Expected Number expression");
        }
    }

    #[test]
    fn test_percentage_function() {
        let registry = FunctionRegistry::new();
        let args = vec![Expression::Number {
            value: 0.5,
            unit: None,
            position: pos(),
        }];

        let result = registry.call("percentage", &args, &pos()).unwrap();

        if let Expression::Percentage(value, _) = result {
            assert_eq!(value, 50.0);
        } else {
            panic!("Expected Percentage expression");
        }
    }

    #[test]
    fn test_rgb_function() {
        let registry = FunctionRegistry::new();
        let args = vec![
            Expression::Number {
                value: 255.0,
                unit: None,
                position: pos(),
            },
            Expression::Number {
                value: 0.0,
                unit: None,
                position: pos(),
            },
            Expression::Number {
                value: 0.0,
                unit: None,
                position: pos(),
            },
        ];

        let result = registry.call("rgb", &args, &pos()).unwrap();

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
            panic!("Expected Color expression");
        }
    }

    #[test]
    fn test_min_max_functions() {
        let registry = FunctionRegistry::new();
        let args = vec![
            Expression::Number {
                value: 10.0,
                unit: Some("px".to_string()),
                position: pos(),
            },
            Expression::Number {
                value: 5.0,
                unit: Some("px".to_string()),
                position: pos(),
            },
            Expression::Number {
                value: 15.0,
                unit: Some("px".to_string()),
                position: pos(),
            },
        ];

        let min_result = registry.call("min", &args, &pos()).unwrap();
        let max_result = registry.call("max", &args, &pos()).unwrap();

        if let Expression::Number { value, .. } = min_result {
            assert_eq!(value, 5.0);
        } else {
            panic!("Expected Number expression");
        }

        if let Expression::Number { value, .. } = max_result {
            assert_eq!(value, 15.0);
        } else {
            panic!("Expected Number expression");
        }
    }

    #[test]
    fn test_undefined_function() {
        let registry = FunctionRegistry::new();
        let args = vec![];

        let result = registry.call("nonexistent", &args, &pos());
        assert!(result.is_err());
    }

    #[test]
    fn test_e_function() {
        let registry = FunctionRegistry::new();
        let args = vec![Expression::string("test".to_string(), pos())];

        let result = registry.call("e", &args, &pos()).unwrap();

        if let Expression::Anonymous(value, _) = result {
            assert_eq!(value, "test");
        } else {
            panic!("Expected Anonymous expression");
        }
    }

    #[test]
    fn test_replace_function() {
        let registry = FunctionRegistry::new();
        let args = vec![
            Expression::string("hello world".to_string(), pos()),
            Expression::string("world".to_string(), pos()),
            Expression::string("LESS".to_string(), pos()),
        ];

        let result = registry.call("replace", &args, &pos()).unwrap();

        if let Expression::String { value, .. } = result {
            assert_eq!(value, "hello LESS");
        } else {
            panic!("Expected String expression");
        }
    }

    #[test]
    fn test_map_get_function() {
        let registry = FunctionRegistry::new();
        let map = map_literal(vec![
            ("width", Expression::number_with_unit(10.0, "px", pos())),
            ("height", Expression::number_with_unit(20.0, "px", pos())),
        ]);
        let args = vec![map, Expression::string("height".to_string(), pos())];

        let result = registry.call("map-get", &args, &pos()).unwrap();

        if let Expression::Number { value, unit, .. } = result {
            assert_eq!(value, 20.0);
            assert_eq!(unit, Some("px".to_string()));
        } else {
            panic!("Expected Number expression");
        }
    }

    #[test]
    fn test_map_has_key_function() {
        let registry = FunctionRegistry::new();
        let map = map_literal(vec![
            ("width", Expression::number_with_unit(10.0, "px", pos())),
        ]);
        let args = vec![
            map.clone(),
            Expression::string("width".to_string(), pos()),
        ];

        let result = registry.call("map-has-key", &args, &pos()).unwrap();

        if let Expression::Boolean(value, _) = result {
            assert!(value);
        } else {
            panic!("Expected Boolean expression");
        }

        let args_false = vec![map, Expression::string("height".to_string(), pos())];
        let result_false = registry.call("map-has-key", &args_false, &pos()).unwrap();

        if let Expression::Boolean(value, _) = result_false {
            assert!(!value);
        } else {
            panic!("Expected Boolean expression");
        }
    }

    #[test]
    fn test_map_keys_function() {
        let registry = FunctionRegistry::new();
        let map = map_literal(vec![
            ("width", Expression::number_with_unit(10.0, "px", pos())),
            ("height", Expression::number_with_unit(20.0, "px", pos())),
        ]);

        let result = registry.call("map-keys", std::slice::from_ref(&map), &pos()).unwrap();

        if let Expression::List { values, separator, .. } = result {
            assert_eq!(values.len(), 2);
            assert_eq!(separator, ListSeparator::Comma);

            let keys: Vec<String> = values
                .iter()
                .filter_map(|v| {
                    if let Expression::String { value, .. } = v {
                        Some(value.clone())
                    } else {
                        None
                    }
                })
                .collect();

            assert!(keys.contains(&"width".to_string()));
            assert!(keys.contains(&"height".to_string()));
        } else {
            panic!("Expected List expression");
        }
    }

    #[test]
    fn test_map_values_function() {
        let registry = FunctionRegistry::new();
        let map = map_literal(vec![
            ("width", Expression::number_with_unit(10.0, "px", pos())),
            ("height", Expression::number_with_unit(20.0, "px", pos())),
        ]);

        let result = registry.call("map-values", std::slice::from_ref(&map), &pos()).unwrap();

        if let Expression::List { values, separator, .. } = result {
            assert_eq!(values.len(), 2);
            assert_eq!(separator, ListSeparator::Comma);
        } else {
            panic!("Expected List expression");
        }
    }

    #[test]
    fn test_map_merge_function() {
        let registry = FunctionRegistry::new();
        let map1 = map_literal(vec![
            ("width", Expression::number_with_unit(10.0, "px", pos())),
            ("height", Expression::number_with_unit(20.0, "px", pos())),
        ]);
        let map2 = map_literal(vec![
            ("height", Expression::number_with_unit(30.0, "px", pos())),
            ("margin", Expression::number_with_unit(5.0, "px", pos())),
        ]);

        let result = registry.call("map-merge", &[map1, map2], &pos()).unwrap();

        if let Expression::MapLiteral { entries, .. } = result {
            assert_eq!(entries.len(), 3);

            // Check specific values
            let map: std::collections::HashMap<String, &Expression> = entries
                .iter()
                .map(|(k, v, _)| (k.clone(), v))
                .collect();

            if let Some(Expression::Number { value, .. }) = map.get("width") {
                assert_eq!(*value, 10.0);
            }
            if let Some(Expression::Number { value, .. }) = map.get("height") {
                assert_eq!(*value, 30.0); // Overwritten by second map
            }
            if let Some(Expression::Number { value, .. }) = map.get("margin") {
                assert_eq!(*value, 5.0);
            }
        } else {
            panic!("Expected MapLiteral expression");
        }
    }

    #[test]
    fn test_map_remove_function() {
        let registry = FunctionRegistry::new();
        let map = map_literal(vec![
            ("width", Expression::number_with_unit(10.0, "px", pos())),
            ("height", Expression::number_with_unit(20.0, "px", pos())),
            ("margin", Expression::number_with_unit(5.0, "px", pos())),
        ]);
        let args = vec![
            map,
            Expression::string("height".to_string(), pos()),
        ];

        let result = registry.call("map-remove", &args, &pos()).unwrap();

        if let Expression::MapLiteral { entries, .. } = result {
            assert_eq!(entries.len(), 2);
            assert!(!entries.iter().any(|(k, _, _)| map_keys_equal(k, "height")));
        } else {
            panic!("Expected MapLiteral expression");
        }
    }
}