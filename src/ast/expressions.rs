//! LESS 值和操作的表达式类型
//!
//! 此模块定义了可以在 LESS 中出现的所有表达式类型，
//! 包括字面量、变量、函数调用和操作。

use super::Position;
use std::fmt;

/// LESS 中的核心表达式类型
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// 字符串字面量: "hello", 'world'
    String {
        /// The string content
        value: String,
        /// Whether the string was originally quoted
        quoted: bool,
        /// Source position of this string
        position: Position,
    },

    /// 带可选单位的数字: 10, 10px, 1.5em
    Number {
        /// The numeric value
        value: f64,
        /// Optional unit (px, em, %, etc.)
        unit: Option<String>,
        /// Source position of this number
        position: Position,
    },

    /// 颜色值: #fff, rgb(255,0,0), hsl(0,100%,50%)
    Color {
        /// Red component (0-255)
        red: u8,
        /// Green component (0-255)
        green: u8,
        /// Blue component (0-255)
        blue: u8,
        /// Alpha component (0.0-1.0)
        alpha: f64,
        /// Source position of this color
        position: Position,
    },

    /// Boolean literal: true, false
    Boolean(bool, Position),

    /// Variable reference: @variable
    Variable(String, Position),

    /// Binary operation: @a + @b, @width * 2
    BinaryOp {
        /// Left operand
        left: Box<Expression>,
        /// Binary operator
        operator: BinaryOperator,
        /// Right operand
        right: Box<Expression>,
        /// Source position of this operation
        position: Position,
    },

    /// Unary operation: -@value, not @condition
    UnaryOp {
        /// Unary operator
        operator: UnaryOperator,
        /// The operand
        operand: Box<Expression>,
        /// Source position of this operation
        position: Position,
    },

    /// Function call: lighten(@color, 20%), round(10.6)
    FunctionCall {
        /// Function name
        name: String,
        /// Function arguments
        arguments: Vec<Expression>,
        /// Source position of this function call
        position: Position,
    },

    /// Parenthesized expression: (@a + @b)
    Parenthesized(Box<Expression>, Position),

    /// List of values: 10px 20px, Arial, sans-serif
    List {
        /// List items
        values: Vec<Expression>,
        /// Separator type (space, comma, etc.)
        separator: ListSeparator,
        /// Source position of this list
        position: Position,
    },

    /// Variable interpolation: @{variable}
    Interpolation(String, Position),

    /// Property interpolation: @{property}: value
    PropertyInterpolation(String, Position),

    /// Selector interpolation: .@{selector}
    SelectorInterpolation(String, Position),

    /// URL expression: url("image.png")
    Url(String, Position),

    /// Template string with interpolation: "prefix@{variable}suffix"
    TemplateString {
        /// Template string parts (text and expressions)
        parts: Vec<TemplateStringPart>,
        /// Source position of this template string
        position: Position,
    },

    /// Dimension with explicit unit conversion
    Dimension {
        /// Numeric value
        value: f64,
        /// Source unit
        from_unit: String,
        /// Target unit
        to_unit: String,
        /// Source position of this dimension
        position: Position,
    },

    /// Percentage value: 50%
    Percentage(f64, Position),

    /// Null/undefined value
    Null(Position),

    /// Map access: @map[key]
    MapAccess {
        /// The map expression
        map: Box<Expression>,
        /// The key expression
        key: Box<Expression>,
        /// Source position of this map access
        position: Position,
    },

    /// Conditional expression: if(@condition, @true-value, @false-value)
    Conditional {
        /// Condition expression
        condition: Box<Expression>,
        /// Value if condition is true
        true_value: Box<Expression>,
        /// Value if condition is false
        false_value: Box<Expression>,
        /// Source position of this conditional
        position: Position,
    },

    /// Property access: @object.property
    PropertyAccess {
        /// The object expression
        object: Box<Expression>,
        /// Property name
        property: String,
        /// Source position of this property access
        position: Position,
    },

    /// Escaped value: ~"literal CSS"
    Escaped(String, Position),

    /// Anonymous value: e(expression)
    Anonymous(String, Position),

    /// JavaScript evaluation: `expression`
    JavaScript(String, Position),

    /// Map literal: { key: value; key2: value2; }
    MapLiteral {
        /// Key-value pairs
        entries: Vec<(String, Expression)>,
        /// Source position
        position: Position,
    },

    /// Detached ruleset: { color: red; .inner { ... } }
    /// A block of statements stored as a value, invoked with @var()
    DetachedRuleset {
        /// The statements inside the detached ruleset
        body: Vec<super::Statement>,
        /// Source position
        position: Position,
    },
}

/// Parts of a template string
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateStringPart {
    /// Literal text
    Text(String),
    /// Variable interpolation
    Interpolation(String),
}

/// Binary operators for expressions
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // Arithmetic
    /// Addition operator (+)
    Add,
    /// Subtraction operator (-)
    Subtract,
    /// Multiplication operator (*)
    Multiply,
    /// Division operator (/)
    Divide,
    /// Modulo operator (%)
    Modulo,

    // Comparison
    /// Equality operator (==)
    Equal,
    /// Inequality operator (!=)
    NotEqual,
    /// Less than operator (<)
    LessThan,
    /// Less than or equal operator (<=)
    LessThanOrEqual,
    /// Greater than operator (>)
    GreaterThan,
    /// Greater than or equal operator (>=)
    GreaterThanOrEqual,

    // Logical
    /// Logical AND operator (&&)
    And,
    /// Logical OR operator (||)
    Or,

    // String
    /// String concatenation
    Concatenate,
}

/// Unary operators for expressions
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    /// Unary minus (-)
    Minus,
    /// Unary plus (+)
    Plus,
    /// Logical NOT (!)
    Not,
}

/// List separator types
#[derive(Debug, Clone, PartialEq)]
pub enum ListSeparator {
    /// Space-separated list
    Space,
    /// Comma-separated list
    Comma,
    /// Semicolon-separated list
    Semicolon,
}

impl Expression {
    /// Create a string expression
    pub fn string(value: String, position: Position) -> Self {
        Expression::String {
            value,
            quoted: true,
            position,
        }
    }

    /// Create an unquoted identifier expression
    pub fn identifier(value: String, position: Position) -> Self {
        Expression::String {
            value,
            quoted: false,
            position,
        }
    }

    /// Create a number expression without unit
    pub fn number(value: f64, position: Position) -> Self {
        Expression::Number {
            value,
            unit: None,
            position,
        }
    }

    /// Create a number expression with unit
    pub fn number_with_unit(value: f64, unit: impl Into<String>, position: Position) -> Self {
        Expression::Number {
            value,
            unit: Some(unit.into()),
            position,
        }
    }

    /// Create a color expression from RGB values
    pub fn color_rgb(red: u8, green: u8, blue: u8, position: Position) -> Self {
        Expression::Color {
            red,
            green,
            blue,
            alpha: 1.0,
            position,
        }
    }

    /// Create a color expression from RGBA values
    pub fn color_rgba(red: u8, green: u8, blue: u8, alpha: f64, position: Position) -> Self {
        Expression::Color {
            red,
            green,
            blue,
            alpha,
            position,
        }
    }

    /// Create a color from hex string
    pub fn color_hex(hex: &str, position: Position) -> Result<Self, &'static str> {
        let hex = hex.trim_start_matches('#');

        let (red, green, blue, alpha) = match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")?;
                (r, g, b, 1.0)
            }
            4 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")?;
                let a = u8::from_str_radix(&hex[3..4].repeat(2), 16)
                    .map_err(|_| "Invalid hex color")? as f64
                    / 255.0;
                (r, g, b, a)
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex color")?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex color")?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex color")?;
                (r, g, b, 1.0)
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex color")?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex color")?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex color")?;
                let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| "Invalid hex color")? as f64
                    / 255.0;
                (r, g, b, a)
            }
            _ => return Err("Invalid hex color length"),
        };

        Ok(Expression::Color {
            red,
            green,
            blue,
            alpha,
            position,
        })
    }

    /// Create a boolean expression
    pub fn boolean(value: bool, position: Position) -> Self {
        Expression::Boolean(value, position)
    }

    /// Create a variable reference
    pub fn variable(name: impl Into<String>, position: Position) -> Self {
        Expression::Variable(name.into(), position)
    }

    /// Create a binary operation
    pub fn binary_op(
        left: Expression,
        operator: BinaryOperator,
        right: Expression,
        position: Position,
    ) -> Self {
        Expression::BinaryOp {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            position,
        }
    }

    /// Create a unary operation
    pub fn unary_op(operator: UnaryOperator, operand: Expression, position: Position) -> Self {
        Expression::UnaryOp {
            operator,
            operand: Box::new(operand),
            position,
        }
    }

    /// Create a function call
    pub fn function_call(
        name: impl Into<String>,
        arguments: Vec<Expression>,
        position: Position,
    ) -> Self {
        Expression::FunctionCall {
            name: name.into(),
            arguments,
            position,
        }
    }

    /// Create a parenthesized expression
    pub fn parenthesized(expr: Expression, position: Position) -> Self {
        Expression::Parenthesized(Box::new(expr), position)
    }

    /// Create a list expression
    pub fn list(values: Vec<Expression>, separator: ListSeparator, position: Position) -> Self {
        Expression::List {
            values,
            separator,
            position,
        }
    }

    /// Create an interpolation expression
    pub fn interpolation(variable: impl Into<String>, position: Position) -> Self {
        Expression::Interpolation(variable.into(), position)
    }

    /// Create a URL expression
    pub fn url(url: impl Into<String>, position: Position) -> Self {
        Expression::Url(url.into(), position)
    }

    /// Create a percentage expression
    pub fn percentage(value: f64, position: Position) -> Self {
        Expression::Percentage(value, position)
    }

    /// Create a null expression
    pub fn null(position: Position) -> Self {
        Expression::Null(position)
    }

    /// Get the position of this expression
    pub fn position(&self) -> &Position {
        match self {
            Expression::String { position: pos, .. }
            | Expression::Number { position: pos, .. }
            | Expression::Color { position: pos, .. }
            | Expression::Boolean(_, pos)
            | Expression::Variable(_, pos)
            | Expression::BinaryOp { position: pos, .. }
            | Expression::UnaryOp { position: pos, .. }
            | Expression::FunctionCall { position: pos, .. }
            | Expression::Parenthesized(_, pos)
            | Expression::List { position: pos, .. }
            | Expression::Interpolation(_, pos)
            | Expression::PropertyInterpolation(_, pos)
            | Expression::SelectorInterpolation(_, pos)
            | Expression::Url(_, pos)
            | Expression::TemplateString { position: pos, .. }
            | Expression::Dimension { position: pos, .. }
            | Expression::Percentage(_, pos)
            | Expression::Null(pos)
            | Expression::MapAccess { position: pos, .. }
            | Expression::Conditional { position: pos, .. }
            | Expression::PropertyAccess { position: pos, .. }
            | Expression::Escaped(_, pos)
            | Expression::Anonymous(_, pos)
            | Expression::JavaScript(_, pos)
            | Expression::MapLiteral { position: pos, .. }
            | Expression::DetachedRuleset { position: pos, .. } => pos,
        }
    }

    /// Check if this expression is a literal value
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Expression::String { .. }
                | Expression::Number { .. }
                | Expression::Color { .. }
                | Expression::Boolean(_, _)
                | Expression::Percentage(_, _)
                | Expression::Null(_)
        )
    }

    /// Check if this expression is a variable reference
    pub fn is_variable(&self) -> bool {
        matches!(self, Expression::Variable(_, _))
    }

    /// Check if this expression requires evaluation
    pub fn needs_evaluation(&self) -> bool {
        match self {
            Expression::Variable(_, _)
            | Expression::BinaryOp { .. }
            | Expression::UnaryOp { .. }
            | Expression::FunctionCall { .. }
            | Expression::Interpolation(_, _)
            | Expression::MapAccess { .. }
            | Expression::Conditional { .. }
            | Expression::PropertyAccess { .. } => true,
            Expression::Parenthesized(expr, _) => expr.needs_evaluation(),
            Expression::List { values, .. } => values.iter().any(|v| v.needs_evaluation()),
            _ => false,
        }
    }

    /// Convert to CSS string representation
    pub fn to_css(&self) -> String {
        match self {
            Expression::String { value, quoted, .. } => {
                if *quoted {
                    format!("\"{}\"", value)
                } else {
                    value.clone()
                }
            }
            Expression::Number { value, unit, .. } => {
                if let Some(unit) = unit {
                    format!("{}{}", value, unit)
                } else {
                    value.to_string()
                }
            }
            Expression::Color {
                red,
                green,
                blue,
                alpha,
                position,
            } => {
                use crate::ast::values::Color;
                let color = Color {
                    red: *red,
                    green: *green,
                    blue: *blue,
                    alpha: *alpha,
                    position: position.clone(),
                };
                color.to_css()
            }
            Expression::Boolean(b, _) => b.to_string(),
            Expression::Variable(name, _) => format!("@{}", name),
            Expression::Parenthesized(expr, _) => format!("({})", expr.to_css()),
            Expression::List {
                values, separator, ..
            } => {
                let sep = match separator {
                    ListSeparator::Space => " ",
                    ListSeparator::Comma => ", ",
                    ListSeparator::Semicolon => "; ",
                };
                values
                    .iter()
                    .map(|v| v.to_css())
                    .collect::<Vec<_>>()
                    .join(sep)
            }
            Expression::Url(url, _) => format!("url({})", url),
            Expression::Percentage(value, _) => format!("{}%", value),
            Expression::Null(_) => "null".to_string(),
            Expression::Escaped(value, _) => value.clone(),
            Expression::Anonymous(value, _) => value.clone(),
            Expression::DetachedRuleset { .. } => "[detached ruleset]".to_string(),
            _ => format!("{:?}", self), // Fallback for complex expressions
        }
    }
}

impl BinaryOperator {
    /// Get the precedence of this operator (higher = binds tighter)
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOperator::Or => 1,
            BinaryOperator::And => 2,
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual => 3,
            BinaryOperator::Add | BinaryOperator::Subtract => 4,
            BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulo => 5,
            BinaryOperator::Concatenate => 6,
        }
    }

    /// Check if this operator is left-associative
    pub fn is_left_associative(&self) -> bool {
        true // All our operators are left-associative
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Subtract => write!(f, "-"),
            BinaryOperator::Multiply => write!(f, "*"),
            BinaryOperator::Divide => write!(f, "/"),
            BinaryOperator::Modulo => write!(f, "%"),
            BinaryOperator::Equal => write!(f, "="),
            BinaryOperator::NotEqual => write!(f, "!="),
            BinaryOperator::LessThan => write!(f, "<"),
            BinaryOperator::LessThanOrEqual => write!(f, "<="),
            BinaryOperator::GreaterThan => write!(f, ">"),
            BinaryOperator::GreaterThanOrEqual => write!(f, ">="),
            BinaryOperator::And => write!(f, "and"),
            BinaryOperator::Or => write!(f, "or"),
            BinaryOperator::Concatenate => write!(f, "++"),
        }
    }
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperator::Minus => write!(f, "-"),
            UnaryOperator::Plus => write!(f, "+"),
            UnaryOperator::Not => write!(f, "not"),
        }
    }
}

impl fmt::Display for ListSeparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ListSeparator::Space => write!(f, "space"),
            ListSeparator::Comma => write!(f, "comma"),
            ListSeparator::Semicolon => write!(f, "semicolon"),
        }
    }
}

// Implement visitor pattern for expressions
use super::Visitor;

impl super::Visitable for Expression {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_expression(self);
        match self {
            Expression::BinaryOp { left, right, .. } => {
                left.accept(visitor);
                right.accept(visitor);
            }
            Expression::UnaryOp { operand, .. } => {
                operand.accept(visitor);
            }
            Expression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    arg.accept(visitor);
                }
            }
            Expression::Parenthesized(expr, _) => {
                expr.accept(visitor);
            }
            Expression::List { values, .. } => {
                for value in values {
                    value.accept(visitor);
                }
            }
            Expression::MapAccess { map, key, .. } => {
                map.accept(visitor);
                key.accept(visitor);
            }
            Expression::Conditional {
                condition,
                true_value,
                false_value,
                ..
            } => {
                condition.accept(visitor);
                true_value.accept(visitor);
                false_value.accept(visitor);
            }
            Expression::PropertyAccess { object, .. } => {
                object.accept(visitor);
            }
            Expression::MapLiteral { entries, .. } => {
                for (_, value) in entries {
                    value.accept(visitor);
                }
            }
            Expression::DetachedRuleset { body, .. } => {
                for statement in body {
                    statement.accept(visitor);
                }
            }
            _ => {} // Leaf expressions don't need to visit children
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_expression() {
        let pos = Position::new(1, 1);
        let expr = Expression::string("hello".to_string(), pos.clone());

        assert_eq!(expr.to_css(), "\"hello\"");
        assert_eq!(expr.position(), &pos);
        assert!(expr.is_literal());
        assert!(!expr.needs_evaluation());
    }

    #[test]
    fn test_number_expression() {
        let pos = Position::new(1, 1);
        let expr = Expression::number(42.0, pos.clone());

        assert_eq!(expr.to_css(), "42");
        assert!(expr.is_literal());
    }

    #[test]
    fn test_number_with_unit() {
        let pos = Position::new(1, 1);
        let expr = Expression::number_with_unit(10.0, "px", pos);

        assert_eq!(expr.to_css(), "10px");
    }

    #[test]
    fn test_color_rgb() {
        let pos = Position::new(1, 1);
        let expr = Expression::color_rgb(255, 0, 0, pos);

        assert_eq!(expr.to_css(), "#f00");
        assert!(expr.is_literal());
    }

    #[test]
    fn test_color_rgba() {
        let pos = Position::new(1, 1);
        let expr = Expression::color_rgba(255, 0, 0, 0.5, pos);

        assert_eq!(expr.to_css(), "rgba(255, 0, 0, 0.5)");
    }

    #[test]
    fn test_color_hex_3() {
        let pos = Position::new(1, 1);
        let expr = Expression::color_hex("#f00", pos).unwrap();

        if let Expression::Color {
            red,
            green,
            blue,
            alpha,
            ..
        } = expr
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
    fn test_color_hex_6() {
        let pos = Position::new(1, 1);
        let expr = Expression::color_hex("#ff0000", pos).unwrap();

        if let Expression::Color {
            red,
            green,
            blue,
            alpha,
            ..
        } = expr
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
    fn test_invalid_hex_color() {
        let pos = Position::new(1, 1);
        let result = Expression::color_hex("#gg", pos);
        assert!(result.is_err());
    }

    #[test]
    fn test_variable_expression() {
        let pos = Position::new(1, 1);
        let expr = Expression::variable("color", pos);

        assert_eq!(expr.to_css(), "@color");
        assert!(expr.is_variable());
        assert!(expr.needs_evaluation());
        assert!(!expr.is_literal());
    }

    #[test]
    fn test_binary_operation() {
        let pos = Position::new(1, 1);
        let left = Expression::number(10.0, pos.clone());
        let right = Expression::number(5.0, pos.clone());
        let expr = Expression::binary_op(left, BinaryOperator::Add, right, pos);

        assert!(expr.needs_evaluation());
        assert!(!expr.is_literal());
    }

    #[test]
    fn test_function_call() {
        let pos = Position::new(1, 1);
        let arg = Expression::number(10.0, pos.clone());
        let expr = Expression::function_call("round", vec![arg], pos);

        assert!(expr.needs_evaluation());
    }

    #[test]
    fn test_list_expression() {
        let pos = Position::new(1, 1);
        let values = vec![
            Expression::number(10.0, pos.clone()),
            Expression::number(20.0, pos.clone()),
        ];
        let expr = Expression::list(values, ListSeparator::Space, pos);

        assert_eq!(expr.to_css(), "10 20");
    }

    #[test]
    fn test_percentage_expression() {
        let pos = Position::new(1, 1);
        let expr = Expression::percentage(50.0, pos);

        assert_eq!(expr.to_css(), "50%");
        assert!(expr.is_literal());
    }

    #[test]
    fn test_operator_precedence() {
        assert!(BinaryOperator::Multiply.precedence() > BinaryOperator::Add.precedence());
        assert!(BinaryOperator::Add.precedence() > BinaryOperator::Equal.precedence());
        assert!(BinaryOperator::Equal.precedence() > BinaryOperator::And.precedence());
        assert!(BinaryOperator::And.precedence() > BinaryOperator::Or.precedence());
    }

    #[test]
    fn test_operator_display() {
        assert_eq!(format!("{}", BinaryOperator::Add), "+");
        assert_eq!(format!("{}", BinaryOperator::Equal), "=");
        assert_eq!(format!("{}", UnaryOperator::Minus), "-");
        assert_eq!(format!("{}", UnaryOperator::Not), "not");
    }

    #[test]
    fn test_list_separator_display() {
        assert_eq!(format!("{}", ListSeparator::Space), "space");
        assert_eq!(format!("{}", ListSeparator::Comma), "comma");
        assert_eq!(format!("{}", ListSeparator::Semicolon), "semicolon");
    }

    #[test]
    fn test_parenthesized_expression() {
        let pos = Position::new(1, 1);
        let inner = Expression::number(42.0, pos.clone());
        let expr = Expression::parenthesized(inner, pos);

        assert_eq!(expr.to_css(), "(42)");
        assert!(!expr.needs_evaluation()); // Inner doesn't need evaluation
    }

    #[test]
    fn test_url_expression() {
        let pos = Position::new(1, 1);
        let expr = Expression::url("image.png", pos);

        assert_eq!(expr.to_css(), "url(image.png)");
        assert!(!expr.needs_evaluation());
    }
}
