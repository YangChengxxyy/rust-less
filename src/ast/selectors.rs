//! CSS 选择器表示的选择器类型
//!
//! 此模块定义了用于表示 CSS 选择器的选择器 AST 节点，
//! 包括简单选择器、组合器和伪类/伪元素。

use super::Position;
use std::fmt;

/// 完整的 CSS 选择器（例如 ".class > p:hover"）
#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    /// The parts that make up this selector
    pub parts: Vec<SelectorPart>,
    /// Source position of this selector
    pub position: Position,
}

/// 由组合器分隔的选择器部分
#[derive(Debug, Clone, PartialEq)]
pub struct SelectorPart {
    /// Simple selectors in this part
    pub simple_selectors: Vec<SimpleSelector>,
    /// Optional combinator to next part
    pub combinator: Option<Combinator>,
    /// Source position of this selector part
    pub position: Position,
}

/// 简单选择器组件
#[derive(Debug, Clone, PartialEq)]
pub enum SimpleSelector {
    /// 通用选择器 (*)
    Universal(Position),

    /// 类型选择器 (div, p, span)
    Type {
        /// Element type name
        name: String,
        /// Source position
        position: Position,
    },

    /// Class selector (.class-name)
    Class {
        /// Class name
        name: String,
        /// Source position
        position: Position,
    },

    /// ID selector (#id-name)
    Id {
        /// ID name
        name: String,
        /// Source position
        position: Position,
    },

    /// Attribute selector ([attr], [attr=value], [attr~=value], etc.)
    Attribute {
        /// Attribute name
        name: String,
        /// Optional operator for attribute matching
        operator: Option<AttributeOperator>,
        /// Optional value to match against
        value: Option<String>,
        /// Whether matching should be case insensitive
        case_insensitive: bool,
        /// Source position
        position: Position,
    },

    /// Pseudo-class (:hover, :nth-child(2n+1))
    PseudoClass {
        /// Pseudo-class name
        name: String,
        /// Optional argument for pseudo-class
        argument: Option<String>,
        /// Source position
        position: Position,
    },

    /// Pseudo-element (::before, ::after)
    PseudoElement {
        /// Pseudo-element name
        name: String,
        /// Source position
        position: Position,
    },

    /// Parent selector reference (&)
    Parent(Position),

    /// Interpolated selector (@{variable})
    Interpolation {
        /// Variable name for interpolation
        variable: String,
        /// Source position
        position: Position,
    },
}

/// Attribute selector operators
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeOperator {
    /// [attr=value] - exact match
    Equal,
    /// [attr~=value] - whitespace-separated word match
    Includes,
    /// [attr|=value] - exact match or followed by hyphen
    DashMatch,
    /// [attr^=value] - starts with
    Prefix,
    /// [attr$=value] - ends with
    Suffix,
    /// [attr*=value] - contains substring
    Substring,
}

/// Selector combinators
#[derive(Debug, Clone, PartialEq)]
pub enum Combinator {
    /// Descendant combinator (space)
    Descendant,
    /// Child combinator (>)
    Child,
    /// Adjacent sibling combinator (+)
    AdjacentSibling,
    /// General sibling combinator (~)
    GeneralSibling,
}

impl Selector {
    /// Create a new selector
    pub fn new(parts: Vec<SelectorPart>, position: Position) -> Self {
        Self { parts, position }
    }

    /// Create a simple selector with a single part
    pub fn simple(selector_text: String, position: Position) -> Self {
        // Handle complex selectors starting with &
        if selector_text.starts_with('&') && selector_text.len() > 1 {
            let mut simple_selectors = vec![SimpleSelector::Parent(position.clone())];
            let remaining = &selector_text[1..];

            // Parse the rest of the selector
            if remaining.starts_with(':') {
                // Pseudo-class like &:hover
                simple_selectors.push(SimpleSelector::PseudoClass {
                    name: remaining[1..].to_string(),
                    argument: None,
                    position: position.clone(),
                });
            } else if remaining.starts_with('.') {
                // Class like &.active
                simple_selectors.push(SimpleSelector::Class {
                    name: remaining[1..].to_string(),
                    position: position.clone(),
                });
            } else {
                // Direct concatenation like &-large
                simple_selectors.push(SimpleSelector::Type {
                    name: remaining.to_string(),
                    position: position.clone(),
                });
            }

            let part = SelectorPart {
                simple_selectors,
                combinator: None,
                position: position.clone(),
            };

            return Self::new(vec![part], position);
        }

        let simple_selector = if selector_text.starts_with('.') {
            SimpleSelector::Class {
                name: selector_text[1..].to_string(),
                position: position.clone(),
            }
        } else if selector_text.starts_with('#') {
            SimpleSelector::Id {
                name: selector_text[1..].to_string(),
                position: position.clone(),
            }
        } else if selector_text == "*" {
            SimpleSelector::Universal(position.clone())
        } else if selector_text == "&" {
            SimpleSelector::Parent(position.clone())
        } else {
            SimpleSelector::Type {
                name: selector_text,
                position: position.clone(),
            }
        };

        let part = SelectorPart {
            simple_selectors: vec![simple_selector],
            combinator: None,
            position: position.clone(),
        };

        Self::new(vec![part], position)
    }

    /// Get the specificity of this selector
    pub fn specificity(&self) -> (u32, u32, u32) {
        let mut ids = 0;
        let mut classes = 0;
        let mut elements = 0;

        for part in &self.parts {
            for simple in &part.simple_selectors {
                match simple {
                    SimpleSelector::Id { .. } => ids += 1,
                    SimpleSelector::Class { .. }
                    | SimpleSelector::Attribute { .. }
                    | SimpleSelector::PseudoClass { .. } => classes += 1,
                    SimpleSelector::Type { .. } | SimpleSelector::PseudoElement { .. } => {
                        elements += 1
                    }
                    SimpleSelector::Universal(_)
                    | SimpleSelector::Parent(_)
                    | SimpleSelector::Interpolation { .. } => {} // No specificity
                }
            }
        }

        (ids, classes, elements)
    }

    /// Check if this selector contains a parent reference (&)
    pub fn has_parent_reference(&self) -> bool {
        self.parts.iter().any(|part| {
            part.simple_selectors
                .iter()
                .any(|simple| matches!(simple, SimpleSelector::Parent(_)))
        })
    }

    /// Convert to CSS string
    pub fn to_css(&self) -> String {
        self.parts
            .iter()
            .enumerate()
            .map(|(i, part)| {
                let result = part
                    .simple_selectors
                    .iter()
                    .map(|s| s.to_css())
                    .collect::<String>();

                if i < self.parts.len() - 1 {
                    if let Some(combinator) = &part.combinator {
                        format!("{} {} ", result, combinator.to_css())
                    } else {
                        format!("{} ", result)
                    }
                } else {
                    result
                }
            })
            .collect()
    }
}

impl SelectorPart {
    /// Create a new selector part
    pub fn new(simple_selectors: Vec<SimpleSelector>, position: Position) -> Self {
        Self {
            simple_selectors,
            combinator: None,
            position,
        }
    }

    /// Add a combinator to this part
    pub fn with_combinator(mut self, combinator: Combinator) -> Self {
        self.combinator = Some(combinator);
        self
    }
}

impl SimpleSelector {
    /// Get the position of this simple selector
    pub fn position(&self) -> &Position {
        match self {
            SimpleSelector::Universal(pos)
            | SimpleSelector::Type { position: pos, .. }
            | SimpleSelector::Class { position: pos, .. }
            | SimpleSelector::Id { position: pos, .. }
            | SimpleSelector::Attribute { position: pos, .. }
            | SimpleSelector::PseudoClass { position: pos, .. }
            | SimpleSelector::PseudoElement { position: pos, .. }
            | SimpleSelector::Parent(pos)
            | SimpleSelector::Interpolation { position: pos, .. } => pos,
        }
    }

    /// Convert to CSS string
    pub fn to_css(&self) -> String {
        match self {
            SimpleSelector::Universal(_) => "*".to_string(),
            SimpleSelector::Type { name, .. } => {
                // Handle direct concatenation with parent selector
                if name.starts_with('-')
                    || name
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
                {
                    name.clone()
                } else {
                    name.clone()
                }
            }
            SimpleSelector::Class { name, .. } => format!(".{}", name),
            SimpleSelector::Id { name, .. } => format!("#{}", name),
            SimpleSelector::Attribute {
                name,
                operator,
                value,
                case_insensitive,
                ..
            } => {
                let mut result = format!("[{}", name);
                if let Some(op) = operator {
                    if let Some(val) = value {
                        result.push_str(&format!("{}\"{}\"", op.to_css(), val));
                        if *case_insensitive {
                            result.push_str(" i");
                        }
                    }
                }
                result.push(']');
                result
            }
            SimpleSelector::PseudoClass { name, argument, .. } => {
                if let Some(arg) = argument {
                    format!(":{}({})", name, arg)
                } else {
                    format!(":{}", name)
                }
            }
            SimpleSelector::PseudoElement { name, .. } => format!("::{}", name),
            SimpleSelector::Parent(_) => "&".to_string(),
            SimpleSelector::Interpolation { variable, .. } => format!("@{{{}}}", variable),
        }
    }

    /// Check if this is a structural pseudo-class (affects layout)
    pub fn is_structural(&self) -> bool {
        match self {
            SimpleSelector::PseudoClass { name, .. } => {
                matches!(
                    name.as_str(),
                    "first-child"
                        | "last-child"
                        | "nth-child"
                        | "nth-last-child"
                        | "first-of-type"
                        | "last-of-type"
                        | "nth-of-type"
                        | "nth-last-of-type"
                        | "only-child"
                        | "only-of-type"
                        | "empty"
                )
            }
            _ => false,
        }
    }
}

impl AttributeOperator {
    /// Convert to CSS string
    pub fn to_css(&self) -> &'static str {
        match self {
            AttributeOperator::Equal => "=",
            AttributeOperator::Includes => "~=",
            AttributeOperator::DashMatch => "|=",
            AttributeOperator::Prefix => "^=",
            AttributeOperator::Suffix => "$=",
            AttributeOperator::Substring => "*=",
        }
    }
}

impl Combinator {
    /// Convert to CSS string
    pub fn to_css(&self) -> &'static str {
        match self {
            Combinator::Descendant => " ",
            Combinator::Child => ">",
            Combinator::AdjacentSibling => "+",
            Combinator::GeneralSibling => "~",
        }
    }

    /// Get the precedence of this combinator (higher = binds tighter)
    pub fn precedence(&self) -> u8 {
        match self {
            Combinator::Descendant => 1,
            Combinator::Child => 2,
            Combinator::AdjacentSibling => 3,
            Combinator::GeneralSibling => 3,
        }
    }
}

impl fmt::Display for AttributeOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_css())
    }
}

impl fmt::Display for Combinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_css())
    }
}

// Implement visitor pattern
use super::{Visitable, Visitor};

impl Visitable for Selector {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_selector(self);
        for part in &self.parts {
            part.accept(visitor);
        }
    }
}

impl Visitable for SelectorPart {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        for simple in &self.simple_selectors {
            simple.accept(visitor);
        }
    }
}

impl Visitable for SimpleSelector {
    fn accept<V: Visitor>(&self, _visitor: &mut V) {
        // Simple selectors are leaf nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_class_selector() {
        let pos = Position::new(1, 1);
        let selector = Selector::simple(".test".to_string(), pos);

        assert_eq!(selector.to_css(), ".test");
        assert_eq!(selector.specificity(), (0, 1, 0));
    }

    #[test]
    fn test_simple_id_selector() {
        let pos = Position::new(1, 1);
        let selector = Selector::simple("#header".to_string(), pos);

        assert_eq!(selector.to_css(), "#header");
        assert_eq!(selector.specificity(), (1, 0, 0));
    }

    #[test]
    fn test_simple_type_selector() {
        let pos = Position::new(1, 1);
        let selector = Selector::simple("div".to_string(), pos);

        assert_eq!(selector.to_css(), "div");
        assert_eq!(selector.specificity(), (0, 0, 1));
    }

    #[test]
    fn test_universal_selector() {
        let pos = Position::new(1, 1);
        let selector = Selector::simple("*".to_string(), pos);

        assert_eq!(selector.to_css(), "*");
        assert_eq!(selector.specificity(), (0, 0, 0));
    }

    #[test]
    fn test_parent_selector() {
        let pos = Position::new(1, 1);
        let selector = Selector::simple("&".to_string(), pos.clone());

        assert_eq!(selector.to_css(), "&");
        assert!(selector.has_parent_reference());
    }

    #[test]
    fn test_attribute_selector() {
        let pos = Position::new(1, 1);
        let attr = SimpleSelector::Attribute {
            name: "href".to_string(),
            operator: Some(AttributeOperator::Equal),
            value: Some("test".to_string()),
            case_insensitive: false,
            position: pos,
        };

        assert_eq!(attr.to_css(), "[href=\"test\"]");
    }

    #[test]
    fn test_attribute_selector_case_insensitive() {
        let pos = Position::new(1, 1);
        let attr = SimpleSelector::Attribute {
            name: "href".to_string(),
            operator: Some(AttributeOperator::Equal),
            value: Some("test".to_string()),
            case_insensitive: true,
            position: pos,
        };

        assert_eq!(attr.to_css(), "[href=\"test\" i]");
    }

    #[test]
    fn test_pseudo_class() {
        let pos = Position::new(1, 1);
        let pseudo = SimpleSelector::PseudoClass {
            name: "hover".to_string(),
            argument: None,
            position: pos,
        };

        assert_eq!(pseudo.to_css(), ":hover");
    }

    #[test]
    fn test_pseudo_class_with_argument() {
        let pos = Position::new(1, 1);
        let pseudo = SimpleSelector::PseudoClass {
            name: "nth-child".to_string(),
            argument: Some("2n+1".to_string()),
            position: pos,
        };

        assert_eq!(pseudo.to_css(), ":nth-child(2n+1)");
    }

    #[test]
    fn test_pseudo_element() {
        let pos = Position::new(1, 1);
        let pseudo = SimpleSelector::PseudoElement {
            name: "before".to_string(),
            position: pos,
        };

        assert_eq!(pseudo.to_css(), "::before");
    }

    #[test]
    fn test_combinator_css() {
        assert_eq!(Combinator::Child.to_css(), ">");
        assert_eq!(Combinator::AdjacentSibling.to_css(), "+");
        assert_eq!(Combinator::GeneralSibling.to_css(), "~");
    }

    #[test]
    fn test_attribute_operator_css() {
        assert_eq!(AttributeOperator::Equal.to_css(), "=");
        assert_eq!(AttributeOperator::Includes.to_css(), "~=");
        assert_eq!(AttributeOperator::Prefix.to_css(), "^=");
        assert_eq!(AttributeOperator::Suffix.to_css(), "$=");
        assert_eq!(AttributeOperator::Substring.to_css(), "*=");
    }

    #[test]
    fn test_structural_pseudo_classes() {
        let pos = Position::new(1, 1);
        let first_child = SimpleSelector::PseudoClass {
            name: "first-child".to_string(),
            argument: None,
            position: pos.clone(),
        };

        let hover = SimpleSelector::PseudoClass {
            name: "hover".to_string(),
            argument: None,
            position: pos,
        };

        assert!(first_child.is_structural());
        assert!(!hover.is_structural());
    }

    #[test]
    fn test_interpolation() {
        let pos = Position::new(1, 1);
        let interpolation = SimpleSelector::Interpolation {
            variable: "selector".to_string(),
            position: pos,
        };

        assert_eq!(interpolation.to_css(), "@{selector}");
    }

    #[test]
    fn test_combinator_precedence() {
        assert!(Combinator::AdjacentSibling.precedence() > Combinator::Child.precedence());
        assert!(Combinator::Child.precedence() > Combinator::Descendant.precedence());
    }
}
