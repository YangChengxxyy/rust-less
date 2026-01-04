use crate::ast::*;
use crate::error::{Error, Result};
use super::Compiler;

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
                    let result_unit = left_unit.clone().or_else(|| right_unit.clone());
                    Ok(Expression::Number {
                        value: left_val + right_val,
                        unit: result_unit,
                        position: position.clone(),
                    })
                }
                BinaryOperator::Subtract => {
                    let result_unit = left_unit.clone().or_else(|| right_unit.clone());
                    Ok(Expression::Number {
                        value: left_val - right_val,
                        unit: result_unit,
                        position: position.clone(),
                    })
                }
                BinaryOperator::Multiply => {
                    let result_unit = left_unit.clone().or_else(|| right_unit.clone());
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
                    let result_unit = if left_unit == right_unit {
                        None
                    } else {
                        left_unit.clone()
                    };
                    Ok(Expression::Number {
                        value: left_val / right_val,
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
