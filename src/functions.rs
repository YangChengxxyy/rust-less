//! LESS 内置函数
//!
//! 此模块提供 LESS 内置函数的实现，用于颜色操作、
//! 数学运算、字符串操作和其他实用功能。

use crate::ast::{Expression, Position};
use crate::error::{Error, Result};

/// LESS 内置函数注册表
pub struct FunctionRegistry {
    functions: std::collections::HashMap<
        String,
        Box<dyn Fn(&[Expression], &Position) -> Result<Expression>>,
    >,
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

        // URL function
        self.register("url", Box::new(url_function));

        // Transform functions
        self.register("scale", Box::new(scale_function));
        self.register("translateX", Box::new(translate_x_function));
        self.register("translateY", Box::new(translate_y_function));
        self.register("rotate", Box::new(rotate_function));
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
        alpha: alpha.max(0.0).min(1.0),
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
        _ => return Err(Error::function_error("spin", "Angle must be a number", position.line, position.column)),
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

    // Extract RGB values
    let mut rgb_values = Vec::new();
    for arg in args {
        match arg {
            Expression::Number { value, .. } => {
                rgb_values.push(*value as u8);
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

    // Extract RGBA values
    let mut rgb_values = Vec::new();
    let mut alpha = 1.0;

    for (i, arg) in args.iter().enumerate() {
        match arg {
            Expression::Number { value, .. } => {
                if i < 3 {
                    rgb_values.push(*value as u8);
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
        _ => return Err(Error::function_error("hsl", "Hue must be a number", position.line, position.column)),
    };
    
    let s = expression_to_percentage(&args[1], position)? / 100.0;
    let l = expression_to_percentage(&args[2], position)? / 100.0;

    let (red, green, blue) = hsl_to_rgb(h, s, l);

    Ok(Expression::Color {
        red,
        green,
        blue,
        alpha: 1.0,
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
        _ => return Err(Error::function_error("hsla", "Hue must be a number", position.line, position.column)),
    };
    
    let s = expression_to_percentage(&args[1], position)? / 100.0;
    let l = expression_to_percentage(&args[2], position)? / 100.0;
    
    let alpha = match &args[3] {
        Expression::Number { value, .. } => *value,
        Expression::Percentage(value, _) => value / 100.0,
        _ => return Err(Error::function_error("hsla", "Alpha must be a number", position.line, position.column)),
    };

    let (red, green, blue) = hsl_to_rgb(h, s, l);

    Ok(Expression::Color {
        red,
        green,
        blue,
        alpha,
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
    ensure_arg_count("replace", args, 3, position)?;

    let input = string_value(&args[0]);
    let pattern = string_value(&args[1]).value;
    let replacement = string_value(&args[2]).value;

    if pattern.is_empty() {
        return Err(Error::function_error(
            "replace",
            "Argument 2 (pattern) must not be empty",
            position.line,
            position.column,
        ));
    }

    let result = input.value.replace(&pattern, &replacement);
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
            format!("Expected {} argument{}, got {}", expected, if expected == 1 { "" } else { "s" }, args.len()),
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

    Ok(Expression::string(scale_str, position.clone()))
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
            Ok(Expression::string(translate_str, position.clone()))
        }
        Expression::String { value, .. } => {
            let translate_str = format!("translateX({})", value);
            Ok(Expression::string(translate_str, position.clone()))
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
            Ok(Expression::string(translate_str, position.clone()))
        }
        Expression::String { value, .. } => {
            let translate_str = format!("translateY({})", value);
            Ok(Expression::string(translate_str, position.clone()))
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
            let unit_str = unit.as_deref().unwrap_or("deg");
            let rotate_str = format!("rotate({}{})", value, unit_str);
            Ok(Expression::string(rotate_str, position.clone()))
        }
        Expression::String { value, .. } => {
            let rotate_str = format!("rotate({})", value);
            Ok(Expression::string(rotate_str, position.clone()))
        }
        _ => Err(Error::function_error(
            "rotate",
            "argument must be a number",
            position.line,
            position.column,
        )),
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
