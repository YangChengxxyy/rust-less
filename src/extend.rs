//! Extend functionality module
//! 
//! This module handles the Less `:extend` pseudo-class and statement functionality.
//! It provides a registry for storing extend relationships and a collector for
//! finding extend statements in the AST.

use crate::ast::{Selector, Statement, Stylesheet, SimpleSelector};
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
        self.extends.entry(key).or_default().push((extending_selectors, all));
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
    /// Only returns matches if the extend has 'all' set to true
    pub fn find_partial_matches(&self, selector_str: &str) -> Vec<String> {
        let mut result = Vec::new();
        // println!("DEBUG: find_partial_matches for '{}'", selector_str);
        
        for (target, list) in &self.extends {
            // println!("DEBUG: Checking target '{}'", target);
            if let Some(_) = selector_str.find(target) {
                for (extenders, all) in list {
                    if *all {
                        for extender in extenders {
                             let new_selector = selector_str.replace(target, extender);
                             if new_selector != selector_str {
                                 result.push(new_selector);
                             }
                        }
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
pub struct ExtendCollector {
    /// The registry being populated
    pub registry: ExtendRegistry,
}

impl ExtendCollector {
    /// Create a new collector
    pub fn new() -> Self {
        Self {
            registry: ExtendRegistry::new(),
        }
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
                    self.registry.add_extend(target, parent_selectors.to_vec(), extend.all);
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
                                     self.registry.add_extend(&target, vec![resolved.clone()], all);
                                 }
                             }
                             Err(_) => {
                                 // Failed to parse extend arg
                             }
                         }
                     }
                 }
             }
        }
    }

    fn parse_extend_arg(&self, arg: &str) -> crate::error::Result<(Vec<Selector>, bool)> {
        // We can reuse Parser to parse selectors from string
        // But argument might contain "all" keyword at the end
        // Parser::parse_extend handles "all" by custom logic.
        // Here we just have the content.
        // E.g. ".b" or ".b all" or ".b, .c"
        
        // Quick hack: Wrap in dummy extend statement to use existing parser? 
        // Or create new parser method.
        // Creating new parser instance for the string.
        let mut parser = Parser::from_string(arg.to_string())?;
        
        // We need to parse selectors until "all" or end.
        // Parser::parse_selectors parses comma separated selectors.
        // But "all" is not a selector.
        // However, "all" is valid tag name.
        // If we use parse_selectors, it might consume "all" as tag selector.
        
        // Manual parsing similar to parse_extend loop but simpler because we are inside parens context
        // actually arg string is just what was inside parens.
        
        // Let's rely on standard selector parsing and check last simple selector?
        // If last selector is "all" (tag), treat as keyword?
        // But ".b all" -> ".b" (class) "all" (descendant tag).
        // Less syntax is special here.
        
        // Let's implement a simplified parse logic for extend args
        // reusing Parser public API if possible.
        // parser.parse_selectors() returns Result<Vec<Selector>>.
        
        let selectors = parser.parse_selectors()?;
        
        // Check the last selector to see if it ends with " all"
        // This is tricky if parsed as "all" tag.
        
        // Alternative: Check string suffix?
        let arg_trim = arg.trim();
        let (arg_clean, has_all) = if arg_trim.ends_with(" all") {
            (&arg_trim[0..arg_trim.len()-4], true)
        } else {
            (arg_trim, false)
        };
        
        // Re-parse without "all" if found
        let (targets, all) = if has_all {
             let mut p2 = Parser::from_string(arg_clean.to_string())?;
             (p2.parse_selectors()?, true)
        } else {
             (selectors, false)
        };
        
        Ok((targets, all))
    }
    
    fn resolve_selectors(&self, selectors: &[Selector], parent_selectors: &[String]) -> Vec<String> {
        let mut result = Vec::new();
        
        for selector in selectors {
            let selector_str = selector.to_css();
            
            if parent_selectors.is_empty() {
                 result.push(selector_str);
            } else {
                 // If the selector has a parent reference (&), we need to replace it
                 if selector.has_parent_reference() {
                     for parent in parent_selectors {
                         // Simple string replacement for &
                         // This handles both prefix (e.g. &:hover) and inline (e.g. .a & .b)
                         result.push(selector_str.replace('&', parent));
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
