//! Extend functionality module
//!
//! This module handles the Less `:extend` pseudo-class and statement functionality.
//! It provides a registry for storing extend relationships and a collector for
//! finding extend statements in the AST.

use crate::ast::{Selector, SimpleSelector, Statement, Stylesheet};
use crate::parser::Parser;
use std::collections::HashMap;

/// Registry for managing extend relationships
#[derive(Debug, Default, Clone)]
pub struct ExtendRegistry {
    /// Map from target selector string to list of extending selectors
    /// Key: The selector being extended (e.g., ".b" in .a:extend(.b))
    /// Value: List of (extending_selectors, is_all)
    /// extending_selectors are fully resolved selector strings (e.g. ".parent .child")
    extends: HashMap<String, Vec<(Vec<String>, bool)>>,
}

impl ExtendRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            extends: HashMap::new(),
        }
    }

    /// Register an extend rule
    /// target: The selector being extended (from the :extend() argument)
    /// extending_selectors: The fully resolved selectors that are doing the extending
    /// all: Whether the 'all' keyword was used
    pub fn add_extend(&mut self, target: &Selector, extending_selectors: Vec<String>, all: bool) {
        let key = target.to_css().trim().to_string();
        self.extends
            .entry(key)
            .or_default()
            .push((extending_selectors, all));
    }

    /// Find selectors that extend the given selector
    /// For exact matching primarily
    pub fn find_exact_matches(&self, selector_str: &str) -> Vec<String> {
        let key = selector_str.trim();

        let mut result = Vec::new();

        if let Some(list) = self.extends.get(key) {
            for (extenders, _) in list {
                result.extend(extenders.clone());
            }
        }

        result
    }

    /// Find selectors that extend the given selector (partial matching)
    /// Only returns matches if the extend has 'all' set to true.
    /// Uses structural matching to avoid false positives like .b matching .button.
    /// Handles both:
    /// - `.a` in `.c .a` (descendant) → `.c .b`
    /// - `.a` in `.a:hover` (compound) → `.b:hover`
    pub fn find_partial_matches(&self, selector_str: &str) -> Vec<String> {
        let mut result = Vec::new();

        for (target, list) in &self.extends {
            for (extenders, all) in list {
                if !*all {
                    continue;
                }

                let target_trimmed = target.trim();
                if !selector_contains_target(selector_str, target_trimmed) {
                    continue;
                }

                for extender in extenders {
                    let new_selector =
                        replace_selector_target(selector_str, target_trimmed, extender);
                    if new_selector != selector_str {
                        result.push(new_selector);
                    }
                }
            }
        }

        result
    }

    /// Merge another registry into this one
    pub fn merge(&mut self, other: ExtendRegistry) {
        for (target, list) in other.extends {
            let entry = self.extends.entry(target).or_default();
            for item in list {
                entry.push(item);
            }
        }
    }
}

/// Collector to find and register extend statements
#[derive(Default)]
pub struct ExtendCollector {
    /// The registry being populated
    pub registry: ExtendRegistry,
}

impl ExtendCollector {
    /// Create a new collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Collect extends from a stylesheet
    pub fn collect(&mut self, stylesheet: &Stylesheet) {
        for statement in &stylesheet.statements {
            self.collect_statement(statement, &[]);
        }
    }

    fn collect_statement(&mut self, statement: &Statement, parent_selectors: &[String]) {
        match statement {
            Statement::Rule(rule) => {
                // Check for selector-attached extends in rule.selectors
                self.check_selector_extends(&rule.selectors, parent_selectors);

                // Resolve selectors for the current rule
                let current_selectors = self.resolve_selectors(&rule.selectors, parent_selectors);

                // Recursively collect from nested rules
                for nested in &rule.nested_rules {
                    self.collect_statement(nested, &current_selectors);
                }
            }
            Statement::Extend(extend) => {
                for target in &extend.selectors {
                    self.registry
                        .add_extend(target, parent_selectors.to_vec(), extend.all);
                }
            }
            Statement::AtRule(at_rule) => {
                if let Some(block) = &at_rule.block {
                    for stmt in block {
                        self.collect_statement(stmt, parent_selectors);
                    }
                }
            }
            // Ignore others
            _ => {}
        }
    }

    fn check_selector_extends(&mut self, selectors: &[Selector], parent_selectors: &[String]) {
        for selector in selectors {
            let mut extends_found = Vec::new();
            let mut clean_selector = selector.clone();

            for part in &mut clean_selector.parts {
                part.simple_selectors.retain(|simple| {
                    if let SimpleSelector::PseudoClass { name, argument, .. } = simple {
                        if name == "extend" {
                            if let Some(arg) = argument {
                                extends_found.push(arg.clone());
                            }
                            return false;
                        }
                    }
                    true
                });
            }

            if !extends_found.is_empty() {
                let clean_selectors_vec = vec![clean_selector];
                let resolved_list = self.resolve_selectors(&clean_selectors_vec, parent_selectors);

                if let Some(resolved) = resolved_list.first() {
                    for extend_arg in extends_found {
                        match self.parse_extend_arg(&extend_arg) {
                            Ok((targets, all)) => {
                                for target in targets {
                                    self.registry
                                        .add_extend(&target, vec![resolved.clone()], all);
                                }
                            }
                            Err(err) => {
                                eprintln!(
                                    "rust-less: ignoring unparseable :extend argument {:?}: {}",
                                    extend_arg, err
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// Parse extend argument string to extract target selectors and 'all' flag.
    ///
    /// Handles formats like:
    /// - ".selector" -> ([.selector], false)
    /// - ".selector all" -> ([.selector], true)
    /// - ".a, .b all" -> ([.a, .b], true)
    fn parse_extend_arg(&self, arg: &str) -> crate::error::Result<(Vec<Selector>, bool)> {
        let arg_trim = arg.trim();

        // Check for 'all' keyword at the end (must be preceded by whitespace)
        let (selector_str, has_all) = if let Some(stripped) = arg_trim.strip_suffix(" all") {
            (stripped, true)
        } else {
            (arg_trim, false)
        };

        // Parse the selector string
        let mut parser = Parser::from_string(selector_str.to_string())?;
        let selectors = parser.parse_selectors()?;

        Ok((selectors, has_all))
    }

    fn resolve_selectors(
        &self,
        selectors: &[Selector],
        parent_selectors: &[String],
    ) -> Vec<String> {
        let mut result = Vec::new();

        for selector in selectors {
            let selector_str = selector.to_css();

            if parent_selectors.is_empty() {
                result.push(selector_str);
            } else {
                // If the selector has a parent reference (&), we need to replace it
                if selector.has_parent_reference() {
                    for parent in parent_selectors {
                        // String-aware replacement for & (skips quoted strings
                        // and attribute selectors); handles both prefix
                        // (e.g. &:hover) and inline (e.g. .a & .b)
                        result.push(replace_parent_refs(&selector_str, parent));
                    }
                } else {
                    // Standard nesting (descendant combinator)
                    for parent in parent_selectors {
                        result.push(format!("{} {}", parent, selector_str));
                    }
                }
            }
        }
        result
    }
}

/// Replace `&` (parent reference) in a selector string with `parent`,
/// skipping occurrences inside double-quoted strings, single-quoted strings,
/// and CSS attribute selectors `[...]`.
///
/// Escape policy: `\\&` outside a string is treated as a literal `&` — the
/// backslash is preserved and no substitution happens (matches how an escaped
/// ampersand avoids being a parent reference). Inside quoted strings a
/// trailing backslash escapes the next character (including the closing
/// quote) per CSS string rules.
fn replace_parent_refs(selector: &str, parent: &str) -> String {
    let mut out = String::with_capacity(selector.len());
    let mut chars = selector.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' | '\'' => {
                out.push(c);
                // Consume the quoted string; backslash escapes the next char.
                while let Some(&next) = chars.peek() {
                    chars.next();
                    out.push(next);
                    if next == '\\' {
                        if let Some(&escaped) = chars.peek() {
                            chars.next();
                            out.push(escaped);
                        }
                    } else if next == c {
                        break;
                    }
                }
            }
            '[' => {
                out.push(c);
                // Attribute selector: copy verbatim until the matching ']'.
                // Quotes inside are consumed as part of the literal.
                let mut quote: Option<char> = None;
                for next in chars.by_ref() {
                    out.push(next);
                    match (quote, next) {
                        (None, '"' | '\'') => quote = Some(next),
                        (Some(q @ ('"' | '\'')), _) if q == next => quote = None,
                        (None, ']') => break,
                        _ => {}
                    }
                }
            }
            '\\' => {
                // Escaped character (e.g. \&): keep both chars verbatim.
                out.push(c);
                if let Some(&next) = chars.peek() {
                    chars.next();
                    out.push(next);
                }
            }
            '&' => out.push_str(parent),
            _ => out.push(c),
        }
    }
    out
}

/// Check if a selector string structurally contains a target selector.
/// This checks at selector-part boundaries:
/// - `.a` matches as a standalone compound part in `.c .a` (separated by combinators)
/// - `.a` matches as the beginning of a compound selector like `.a:hover` or `.a.b`
/// - `.b` does NOT match inside `.button` (not at a boundary)
fn selector_contains_target(selector: &str, target: &str) -> bool {
    // Split selector by combinator boundaries (space, >, +, ~)
    for compound in split_by_combinators(selector) {
        let compound = compound.trim();
        if compound.is_empty() {
            continue;
        }
        // Check if this compound selector starts with or equals the target
        if compound_contains_simple(compound, target) {
            return true;
        }
    }
    false
}

/// Check if a compound selector (like ".a:hover" or ".a.b") contains
/// the target as a structurally valid sub-selector.
fn compound_contains_simple(compound: &str, target: &str) -> bool {
    if compound == target {
        return true;
    }
    // Check if compound starts with target followed by a selector boundary
    // Boundaries: '.', '#', ':', '[', or end-of-string
    if let Some(rest) = compound.strip_prefix(target) {
        if rest.is_empty() {
            return true;
        }
        if let Some(next_char) = rest.chars().next() {
            if next_char == '.' || next_char == '#' || next_char == ':' || next_char == '[' {
                return true;
            }
        }
    }
    false
}

/// Split a selector string by combinators (space, >, +, ~)
fn split_by_combinators(selector: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut last = 0;

    let bytes = selector.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b' ' || c == b'>' || c == b'+' || c == b'~' {
            let part = &selector[last..i];
            if !part.trim().is_empty() {
                parts.push(part.trim());
            }
            // Skip consecutive combinator chars and spaces
            while i < bytes.len()
                && (bytes[i] == b' ' || bytes[i] == b'>' || bytes[i] == b'+' || bytes[i] == b'~')
            {
                i += 1;
            }
            last = i;
            continue;
        }
        i += 1;
    }

    let remaining = selector[last..].trim();
    if !remaining.is_empty() {
        parts.push(remaining);
    }

    parts
}

/// Replace the target selector within a full selector string, respecting structure.
/// For each compound part that contains the target, replace the target portion.
fn replace_selector_target(selector: &str, target: &str, replacement: &str) -> String {
    let mut result = String::new();
    let mut last = 0;

    let bytes = selector.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b' ' || c == b'>' || c == b'+' || c == b'~' {
            // Process the compound part before this combinator
            let part = &selector[last..i];
            let trimmed = part.trim();
            if !trimmed.is_empty() && compound_contains_simple(trimmed, target) {
                // Preserve any leading whitespace from the original
                let leading = &part[..part.len() - part.trim_start().len()];
                result.push_str(leading);
                result.push_str(&trimmed.replacen(target, replacement, 1));
            } else {
                result.push_str(part);
            }

            // Copy combinators/spaces as-is
            let comb_start = i;
            while i < bytes.len()
                && (bytes[i] == b' ' || bytes[i] == b'>' || bytes[i] == b'+' || bytes[i] == b'~')
            {
                i += 1;
            }
            result.push_str(&selector[comb_start..i]);
            last = i;
            continue;
        }
        i += 1;
    }

    // Process final part
    let part = &selector[last..];
    let trimmed = part.trim();
    if !trimmed.is_empty() && compound_contains_simple(trimmed, target) {
        let leading = &part[..part.len() - part.trim_start().len()];
        result.push_str(leading);
        result.push_str(&trimmed.replacen(target, replacement, 1));
        // Preserve trailing whitespace
        let trailing = &part[part.trim_end().len()..];
        if !trailing.is_empty() {
            // Only add if the replacement didn't already include it
        }
    } else {
        result.push_str(part);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::replace_parent_refs;

    #[test]
    fn replaces_plain_parent_refs() {
        assert_eq!(replace_parent_refs("&:hover", ".a"), ".a:hover");
        assert_eq!(replace_parent_refs(".a & .b", ".p"), ".a .p .b");
        assert_eq!(replace_parent_refs("a&", ".p"), "a.p");
        assert_eq!(replace_parent_refs("&#id", ".p"), ".p#id");
    }

    #[test]
    fn skips_ampersand_in_double_quoted_string() {
        assert_eq!(replace_parent_refs("[data-x=\"a&b\"]", ".p"), "[data-x=\"a&b\"]");
        assert_eq!(replace_parent_refs("\"a&b\" &", ".p"), "\"a&b\" .p");
    }

    #[test]
    fn skips_ampersand_in_single_quoted_string() {
        assert_eq!(replace_parent_refs("'a&b'&:hover", ".p"), "'a&b'.p:hover");
    }

    #[test]
    fn skips_ampersand_in_attribute_selector() {
        assert_eq!(
            replace_parent_refs("&[data-x=\"a&b\"]", ".p"),
            ".p[data-x=\"a&b\"]"
        );
        assert_eq!(
            replace_parent_refs("&[title='c&d']", ".p"),
            ".p[title='c&d']"
        );
    }

    #[test]
    fn escaped_ampersand_is_literal() {
        // Policy: \& keeps its backslash and is not a parent reference.
        assert_eq!(replace_parent_refs("\\& &", ".p"), "\\& .p");
    }

    #[test]
    fn string_escape_handling() {
        // Escaped quote inside string must not end the string; the & after
        // the real closing quote is replaced.
        assert_eq!(
            replace_parent_refs("\"a\\\"&b\" &", ".p"),
            "\"a\\\"&b\" .p"
        );
    }
}
