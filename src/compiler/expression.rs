use super::Compiler;
use crate::ast::*;
use crate::error::{Error, Result};

/// 表达式编译特性
pub trait ExpressionCompiler {
    /// 评估表达式
    fn evaluate_expression(&mut self, expr: &Expression) -> Result<Expression>;

    /// 将表达式评估为字符串（用于插值）
    fn evaluate_expression_to_string(&mut self, expr: &Expression) -> Result<String>;

    /// 解析变量
    fn resolve_variable(&mut self, name: &str) -> Result<Expression>;

    /// 评估二元操作
    fn evaluate_binary_op(
        &self,
        left: &Expression,
        operator: &BinaryOperator,
        right: &Expression,
        position: &Position,
    ) -> Result<Expression>;

    /// 评估一元操作
    fn evaluate_unary_op(
        &self,
        operator: &UnaryOperator,
        operand: &Expression,
        position: &Position,
    ) -> Result<Expression>;

    /// 评估函数调用
    fn evaluate_function_call(
        &self,
        name: &str,
        arguments: &[Expression],
        position: &Position,
    ) -> Result<Expression>;

    /// 检查表达式是否为真值
    fn is_truthy(&self, expr: &Expression) -> bool;
}

impl ExpressionCompiler for Compiler {
    fn evaluate_expression(&mut self, expr: &Expression) -> Result<Expression> {
        match expr {
            Expression::Variable(name, pos) => {
                if let Some(value) = self.current_scope().lookup_variable(name) {
                    Ok(value.clone())
                } else {
                    Err(Error::undefined_variable(name, pos.line, pos.column))
                }
            }
            Expression::Interpolation(name, pos) => {
                // Variable interpolation - resolve variable and return its value
                if let Some(value) = self.current_scope().lookup_variable(name) {
                    Ok(value.clone())
                } else {
                    Err(Error::undefined_variable(name, pos.line, pos.column))
                }
            }
            Expression::BinaryOp {
                left,
                operator,
                right,
                position,
            } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                self.evaluate_binary_op(&left_val, operator, &right_val, position)
            }
            Expression::UnaryOp {
                operator,
                operand,
                position,
            } => {
                let operand_val = self.evaluate_expression(operand)?;
                self.evaluate_unary_op(operator, &operand_val, position)
            }
            Expression::FunctionCall {
                name,
                arguments,
                position,
            } => {
                let mut eval_args = Vec::new();
                for arg in arguments {
                    eval_args.push(self.evaluate_expression(arg)?);
                }
                self.evaluate_function_call(name, &eval_args, position)
            }
            Expression::Parenthesized(inner, _) => self.evaluate_expression(inner),
            Expression::List {
                values,
                separator,
                position,
            } => {
                let mut eval_values = Vec::new();
                for value in values {
                    eval_values.push(self.evaluate_expression(value)?);
                }
                Ok(Expression::list(
                    eval_values,
                    separator.clone(),
                    position.clone(),
                ))
            }
            Expression::TemplateString { parts, position } => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        TemplateStringPart::Text(text) => {
                            result.push_str(text);
                        }
                        TemplateStringPart::Interpolation(var_name) => {
                            let var_value = self.resolve_variable(var_name)?;
                            let interpolated = self.evaluate_expression_to_string(&var_value)?;
                            result.push_str(&interpolated);
                        }
                    }
                }
                Ok(Expression::string(result, position.clone()))
            }
            Expression::MapLiteral { entries, position } => {
                // Evaluate all values in the map
                let mut eval_entries = Vec::new();
                for (key, value) in entries {
                    let eval_value = self.evaluate_expression(value)?;
                    eval_entries.push((key.clone(), eval_value));
                }
                Ok(Expression::MapLiteral {
                    entries: eval_entries,
                    position: position.clone(),
                })
            }
            Expression::MapAccess { map, key, position } => {
                let map_val = self.evaluate_expression(map)?;
                let key_val = self.evaluate_expression(key)?;
                let key_str = normalize_map_key(&key_val);

                if let Expression::MapLiteral { entries, .. } = &map_val {
                    for (k, v) in entries {
                        if k == &key_str {
                            return Ok(v.clone());
                        }
                    }
                    Err(Error::semantic_error(
                        format!("Key '{}' not found in map", key_str),
                        position.line,
                        position.column,
                    ))
                } else {
                    Err(Error::semantic_error(
                        "Cannot access property on non-map value",
                        position.line,
                        position.column,
                    ))
                }
            }
            // Detached rulesets are opaque values; pass through without evaluation.
            // They are only expanded when invoked via `@var()`.
            Expression::DetachedRuleset { .. } => Ok(expr.clone()),
            _ => Ok(expr.clone()), // Literals don't need evaluation
        }
    }

    fn evaluate_expression_to_string(&mut self, expr: &Expression) -> Result<String> {
        let evaluated = self.evaluate_expression(expr)?;
        Ok(match evaluated {
            Expression::String { value, .. } => value,
            Expression::Number { value, unit, .. } => {
                if let Some(u) = unit {
                    format!("{}{}", value, u)
                } else {
                    value.to_string()
                }
            }
            Expression::Color {
                red,
                green,
                blue,
                alpha,
                ..
            } => {
                if alpha == 1.0 {
                    format!("#{:02x}{:02x}{:02x}", red, green, blue)
                } else {
                    format!("rgba({}, {}, {}, {})", red, green, blue, alpha)
                }
            }
            Expression::Boolean(b, _) => b.to_string(),
            _ => {
                // For other types, format as string
                format!("{:?}", evaluated)
            }
        })
    }

    fn resolve_variable(&mut self, name: &str) -> Result<Expression> {
        if let Some(value) = self.current_scope().lookup_variable(name) {
            Ok(value.clone())
        } else {
            Err(Error::undefined_variable(name, 0, 0))
        }
    }

    fn evaluate_binary_op(
        &self,
        left: &Expression,
        operator: &BinaryOperator,
        right: &Expression,
        position: &Position,
    ) -> Result<Expression> {
        match (left, right) {
            (
                Expression::Number {
                    value: left_val,
                    unit: left_unit,
                    ..
                },
                Expression::Number {
                    value: right_val,
                    unit: right_unit,
                    ..
                },
            ) => match operator {
                BinaryOperator::Add => {
                    let result_unit = merge_units(left_unit, right_unit);
                    Ok(Expression::Number {
                        value: left_val + right_val,
                        unit: result_unit,
                        position: position.clone(),
                    })
                }
                BinaryOperator::Subtract => {
                    let result_unit = merge_units(left_unit, right_unit);
                    Ok(Expression::Number {
                        value: left_val - right_val,
                        unit: result_unit,
                        position: position.clone(),
                    })
                }
                BinaryOperator::Multiply => {
                    // Multiplication: if one side is unitless, use the other's unit
                    let result_unit = match (left_unit, right_unit) {
                        (Some(l), None) => Some(l.clone()),
                        (None, Some(r)) => Some(r.clone()),
                        (Some(l), Some(_)) => Some(l.clone()), // left takes precedence
                        (None, None) => None,
                    };
                    Ok(Expression::Number {
                        value: left_val * right_val,
                        unit: result_unit,
                        position: position.clone(),
                    })
                }
                BinaryOperator::Divide => {
                    if *right_val == 0.0 {
                        return Err(Error::division_by_zero(position.line, position.column));
                    }
                    // Division: same units cancel out; otherwise left unit preserved
                    let result_unit = match (left_unit, right_unit) {
                        (Some(l), Some(r)) if l == r => None, // units cancel
                        (Some(l), None) => Some(l.clone()),
                        (None, _) => None,
                        (Some(l), Some(_)) => Some(l.clone()), // incompatible, keep left
                    };
                    Ok(Expression::Number {
                        value: left_val / right_val,
                        unit: result_unit,
                        position: position.clone(),
                    })
                }
                BinaryOperator::Modulo => {
                    if *right_val == 0.0 {
                        return Err(Error::division_by_zero(position.line, position.column));
                    }
                    let result_unit = merge_units(left_unit, right_unit);
                    Ok(Expression::Number {
                        value: left_val % right_val,
                        unit: result_unit,
                        position: position.clone(),
                    })
                }
                BinaryOperator::GreaterThan => {
                    Ok(Expression::Boolean(left_val > right_val, position.clone()))
                }
                BinaryOperator::LessThan => {
                    Ok(Expression::Boolean(left_val < right_val, position.clone()))
                }
                BinaryOperator::GreaterThanOrEqual => {
                    Ok(Expression::Boolean(left_val >= right_val, position.clone()))
                }
                BinaryOperator::LessThanOrEqual => {
                    Ok(Expression::Boolean(left_val <= right_val, position.clone()))
                }
                BinaryOperator::Equal => Ok(Expression::Boolean(
                    (left_val - right_val).abs() < f64::EPSILON,
                    position.clone(),
                )),
                BinaryOperator::NotEqual => Ok(Expression::Boolean(
                    (left_val - right_val).abs() >= f64::EPSILON,
                    position.clone(),
                )),
                _ => Err(Error::type_mismatch(
                    "numeric operation",
                    "unsupported operator",
                    position.line,
                    position.column,
                )),
            },
            // String concatenation with Add operator
            (
                Expression::String {
                    value: left_val,
                    quoted: left_quoted,
                    ..
                },
                right_expr,
            ) if matches!(operator, BinaryOperator::Add) => {
                let right_str = match right_expr {
                    Expression::String { value, .. } => value.clone(),
                    _ => right_expr.to_css(),
                };
                Ok(Expression::String {
                    value: format!("{}{}", left_val, right_str),
                    quoted: *left_quoted,
                    position: position.clone(),
                })
            }
            (
                left_expr,
                Expression::String {
                    value: right_val, ..
                },
            ) if matches!(operator, BinaryOperator::Add) => {
                let left_str = match left_expr {
                    Expression::String { value, .. } => value.clone(),
                    _ => left_expr.to_css(),
                };
                Ok(Expression::String {
                    value: format!("{}{}", left_str, right_val),
                    quoted: false,
                    position: position.clone(),
                })
            }
            // Color + Color arithmetic
            (
                Expression::Color {
                    red: r1,
                    green: g1,
                    blue: b1,
                    alpha: a1,
                    ..
                },
                Expression::Color {
                    red: r2,
                    green: g2,
                    blue: b2,
                    alpha: a2,
                    ..
                },
            ) => match operator {
                BinaryOperator::Add => Ok(Expression::Color {
                    red: (*r1 as u16 + *r2 as u16).min(255) as u8,
                    green: (*g1 as u16 + *g2 as u16).min(255) as u8,
                    blue: (*b1 as u16 + *b2 as u16).min(255) as u8,
                    alpha: *a1,
                    position: position.clone(),
                }),
                BinaryOperator::Subtract => Ok(Expression::Color {
                    red: (*r1 as i16 - *r2 as i16).max(0) as u8,
                    green: (*g1 as i16 - *g2 as i16).max(0) as u8,
                    blue: (*b1 as i16 - *b2 as i16).max(0) as u8,
                    alpha: *a1,
                    position: position.clone(),
                }),
                BinaryOperator::Multiply => Ok(Expression::Color {
                    red: ((*r1 as u16) * (*r2 as u16) / 255).min(255) as u8,
                    green: ((*g1 as u16) * (*g2 as u16) / 255).min(255) as u8,
                    blue: ((*b1 as u16) * (*b2 as u16) / 255).min(255) as u8,
                    alpha: *a1,
                    position: position.clone(),
                }),
                BinaryOperator::Equal => {
                    let equal = r1 == r2 && g1 == g2 && b1 == b2 && (a1 - a2).abs() < f64::EPSILON;
                    Ok(Expression::Boolean(equal, position.clone()))
                }
                BinaryOperator::NotEqual => {
                    let equal = r1 == r2 && g1 == g2 && b1 == b2 && (a1 - a2).abs() < f64::EPSILON;
                    Ok(Expression::Boolean(!equal, position.clone()))
                }
                _ => Ok(Expression::binary_op(
                    left.clone(),
                    operator.clone(),
                    right.clone(),
                    position.clone(),
                )),
            },
            // Color * Number, Color / Number, Color + Number, Color - Number
            (
                Expression::Color {
                    red,
                    green,
                    blue,
                    alpha,
                    ..
                },
                Expression::Number { value: num, .. },
            ) => match operator {
                BinaryOperator::Multiply => Ok(Expression::Color {
                    red: ((*red as f64) * num).round().clamp(0.0, 255.0) as u8,
                    green: ((*green as f64) * num).round().clamp(0.0, 255.0) as u8,
                    blue: ((*blue as f64) * num).round().clamp(0.0, 255.0) as u8,
                    alpha: *alpha,
                    position: position.clone(),
                }),
                BinaryOperator::Divide => {
                    if *num == 0.0 {
                        return Err(Error::division_by_zero(position.line, position.column));
                    }
                    Ok(Expression::Color {
                        red: ((*red as f64) / num).round().clamp(0.0, 255.0) as u8,
                        green: ((*green as f64) / num).round().clamp(0.0, 255.0) as u8,
                        blue: ((*blue as f64) / num).round().clamp(0.0, 255.0) as u8,
                        alpha: *alpha,
                        position: position.clone(),
                    })
                }
                BinaryOperator::Add => Ok(Expression::Color {
                    red: ((*red as f64) + num).round().clamp(0.0, 255.0) as u8,
                    green: ((*green as f64) + num).round().clamp(0.0, 255.0) as u8,
                    blue: ((*blue as f64) + num).round().clamp(0.0, 255.0) as u8,
                    alpha: *alpha,
                    position: position.clone(),
                }),
                BinaryOperator::Subtract => Ok(Expression::Color {
                    red: ((*red as f64) - num).round().clamp(0.0, 255.0) as u8,
                    green: ((*green as f64) - num).round().clamp(0.0, 255.0) as u8,
                    blue: ((*blue as f64) - num).round().clamp(0.0, 255.0) as u8,
                    alpha: *alpha,
                    position: position.clone(),
                }),
                _ => Ok(Expression::binary_op(
                    left.clone(),
                    operator.clone(),
                    right.clone(),
                    position.clone(),
                )),
            },
            // Number * Color
            (
                Expression::Number { value: num, .. },
                Expression::Color {
                    red,
                    green,
                    blue,
                    alpha,
                    ..
                },
            ) if matches!(operator, BinaryOperator::Multiply) => Ok(Expression::Color {
                red: ((*red as f64) * num).round().clamp(0.0, 255.0) as u8,
                green: ((*green as f64) * num).round().clamp(0.0, 255.0) as u8,
                blue: ((*blue as f64) * num).round().clamp(0.0, 255.0) as u8,
                alpha: *alpha,
                position: position.clone(),
            }),
            // Boolean logical operators
            (left_expr, right_expr)
                if matches!(operator, BinaryOperator::And | BinaryOperator::Or) =>
            {
                let left_truthy = self.is_truthy(left_expr);
                let right_truthy = self.is_truthy(right_expr);
                match operator {
                    BinaryOperator::And => Ok(Expression::Boolean(
                        left_truthy && right_truthy,
                        position.clone(),
                    )),
                    BinaryOperator::Or => Ok(Expression::Boolean(
                        left_truthy || right_truthy,
                        position.clone(),
                    )),
                    _ => unreachable!(),
                }
            }
            _ => {
                // For now, just return the original binary operation
                Ok(Expression::binary_op(
                    left.clone(),
                    operator.clone(),
                    right.clone(),
                    position.clone(),
                ))
            }
        }
    }

    fn evaluate_unary_op(
        &self,
        operator: &UnaryOperator,
        operand: &Expression,
        position: &Position,
    ) -> Result<Expression> {
        match operand {
            Expression::Number { value, unit, .. } => match operator {
                UnaryOperator::Minus => Ok(Expression::Number {
                    value: -value,
                    unit: unit.clone(),
                    position: position.clone(),
                }),
                UnaryOperator::Plus => Ok(operand.clone()),
                UnaryOperator::Not => Ok(Expression::Boolean(*value == 0.0, position.clone())),
            },
            Expression::Boolean(val, _) => match operator {
                UnaryOperator::Not => Ok(Expression::Boolean(!val, position.clone())),
                _ => Err(Error::type_mismatch(
                    "boolean",
                    "numeric operator",
                    position.line,
                    position.column,
                )),
            },
            _ => {
                // Return the original unary operation
                Ok(Expression::unary_op(
                    operator.clone(),
                    operand.clone(),
                    position.clone(),
                ))
            }
        }
    }

    fn evaluate_function_call(
        &self,
        name: &str,
        arguments: &[Expression],
        position: &Position,
    ) -> Result<Expression> {
        // Use the function registry to call the function
        self.function_registry.call(name, arguments, position)
    }

    fn is_truthy(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Boolean(b, _) => *b,
            Expression::Number { value, .. } => *value != 0.0,
            Expression::String { value, .. } => !value.is_empty(),
            _ => false,
        }
    }
}

/// Merge units for add/subtract operations.
/// If both have units: left takes precedence (LESS behavior).
/// If one is unitless: use the other's unit.
fn merge_units(left: &Option<String>, right: &Option<String>) -> Option<String> {
    match (left, right) {
        (Some(l), _) => Some(l.clone()),
        (None, Some(r)) => Some(r.clone()),
        (None, None) => None,
    }
}

fn normalize_map_key(expr: &Expression) -> String {
    match expr {
        Expression::String { value, quoted, .. } => {
            if *quoted {
                format!("\"{}\"", value)
            } else {
                value.clone()
            }
        }
        Expression::Number { value, unit, .. } => match unit {
            Some(unit) => format!("{}{}", value, unit),
            None => value.to_string(),
        },
        Expression::Percentage(value, _) => format!("{}%", value),
        Expression::Parenthesized(inner, _) => normalize_map_key(inner),
        _ => expr.to_css(),
    }
}

/// Check if two unit strings belong to the same unit group.
/// Used for compatibility checks (length, angle, time, frequency, resolution).
#[allow(dead_code)]
fn units_compatible(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let group = |u: &str| -> u8 {
        match u {
            "px" | "em" | "rem" | "ex" | "ch" | "vw" | "vh" | "vmin" | "vmax" | "cm" | "mm"
            | "in" | "pt" | "pc" | "q" => 1, // length
            "deg" | "grad" | "rad" | "turn" => 2, // angle
            "s" | "ms" => 3,                      // time
            "hz" | "khz" => 4,                    // frequency
            "dpi" | "dpcm" | "dppx" => 5,         // resolution
            "%" => 6,                             // percentage
            _ => 0,                               // unknown
        }
    };
    let ga = group(a);
    let gb = group(b);
    ga != 0 && ga == gb
}
