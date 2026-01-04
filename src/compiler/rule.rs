use crate::ast::*;
use crate::error::{Error, Result};
use super::Compiler;
use super::expression::ExpressionCompiler;
use super::mixin::MixinCompiler;
use super::at_rule::AtRuleCompiler;

/// 规则编译特性
pub trait RuleCompiler {
    /// 编译规则
    fn compile_rule(&mut self, rule: &Rule, parent_selectors: &[String]) -> Result<()>;
    
    /// 编译选择器
    fn compile_selector(&mut self, selector: &Selector, parent_selectors: &[String]) -> Result<String>;
    
    /// 编译声明
    fn compile_declaration(&mut self, declaration: &Declaration) -> Result<()>;
    
    /// 编译注释
    fn compile_comment(&mut self, comment: &Comment) -> Result<()>;
    
    /// 为当前选择器创建规则
    fn create_rule_for_current_selectors(
        &self,
        statements: &mut Vec<Statement>,
        declarations: &[Declaration],
        position: &Position,
    ) -> Result<()>;
}

impl RuleCompiler for Compiler {
    fn compile_rule(&mut self, rule: &Rule, parent_selectors: &[String]) -> Result<()> {
        // Register this rule as a mixin in the current scope (Implicit Mixin)
        self.register_rule_as_mixin(rule);

        // Push a new scope for this rule
        self.push_scope();

        // First, process any nested variable declarations to establish local scope
        for nested in &rule.nested_rules {
            if let Statement::Variable(var) = nested {
                // TODO: compile_variable_declaration should be accessible or moved
                // Currently in mod.rs, but we can call compile_statement for it or just duplicate simple logic
                // But better to call compile_statement if it was available here.
                // Since compile_statement is in mod.rs and Compiler implements it... wait.
                // Circular dependency: RuleCompiler -> Compiler -> compile_statement -> RuleCompiler.
                // We can't call methods on Compiler that are defined in mod.rs unless we define a trait for them or they are inherent.
                // But inherent methods are visible.
                // However, compile_statement is in mod.rs and calls compile_rule.
                
                // For now, let's assume we can call `compile_variable_declaration` if it's public enough or just replicate logic.
                // Replicating variable compilation logic here for simplicity as it's small.
                let value = self.evaluate_expression(&var.value)?;
                self.current_scope().define_variable(var.name.clone(), value);
            }
        }

        // Convert selectors to strings
        let mut current_selectors = Vec::new();
        for selector in &rule.selectors {
            let selector_str = self.compile_selector(selector, parent_selectors)?;
            
            // Check for extends (Phase 2 & 3)
            let exact_matches = self.extend_registry.find_exact_matches(&selector_str);
            let partial_matches = self.extend_registry.find_partial_matches(&selector_str);
            
            current_selectors.push(selector_str);
            
            // Add unique matches
            for m in exact_matches.into_iter().chain(partial_matches.into_iter()) {
                if !current_selectors.contains(&m) {
                    current_selectors.push(m);
                }
            }
        }

        // Save previous selectors and update current selectors in compiler state
        let previous_selectors = self.current_selectors.clone();
        self.current_selectors = current_selectors.clone();

        // Check if we need to output this rule (has declarations, mixin calls, or comments)
        let has_declarations = !rule.declarations.is_empty();
        let has_mixin_calls = rule
            .nested_rules
            .iter()
            .any(|stmt| matches!(stmt, Statement::MixinCall(_)));
        let has_comments = rule
            .nested_rules
            .iter()
            .any(|stmt| matches!(stmt, Statement::Comment(_)));

        if has_declarations || has_mixin_calls || has_comments {
            self.add_indent();
            self.output.push_str(&current_selectors.join(", "));
            self.add_space();
            self.output.push('{');
            self.add_newline();

            self.indent_level += 1;

            // Create a enum to represent items to process
            #[derive(Clone)]
            enum RuleItem {
                Declaration(Declaration),
                MixinCall(MixinCall),
                Comment(Comment),
            }

            // Collect all declarations, mixin calls, and comments with their positions
            let mut items: Vec<(usize, usize, RuleItem)> = Vec::new();

            // Add declarations
            for declaration in &rule.declarations {
                items.push((
                    declaration.position.line,
                    declaration.position.column,
                    RuleItem::Declaration(declaration.clone()),
                ));
            }

            // Add mixin calls and comments
            for nested in &rule.nested_rules {
                match nested {
                    Statement::MixinCall(call) => {
                        items.push((
                            call.position.line,
                            call.position.column,
                            RuleItem::MixinCall(call.clone()),
                        ));
                    }
                    Statement::Comment(comment) => {
                        items.push((
                            comment.position.line,
                            comment.position.column,
                            RuleItem::Comment(comment.clone()),
                        ));
                    }
                    _ => {}
                }
            }

            // Sort by line number and column
            items.sort_by_key(|(line, column, _)| (*line, *column));
            
            for (_, _, item) in items {
                match item {
                    RuleItem::Declaration(decl) => self.compile_declaration(&decl)?,
                    RuleItem::MixinCall(call) => self.compile_mixin_call(&call)?,
                    RuleItem::Comment(comment) => self.compile_comment(&comment)?,
                }
            }

            self.indent_level -= 1;

            self.add_indent();
            self.output.push('}');
            self.add_newline();
        }

        // Compile nested rules (excluding variable declarations, mixin calls, and comments which were already processed)
        for nested in &rule.nested_rules {
            match nested {
                Statement::Rule(nested_rule) => {
                    self.compile_rule(nested_rule, &current_selectors)?;
                }
                Statement::Variable(_) => {
                    // Already processed above
                    continue;
                }
                Statement::MixinCall(_) => {
                    // Already processed above
                    continue;
                }
                Statement::Comment(_) => {
                    // Already processed above inside the block
                    continue;
                }
                Statement::AtRule(at_rule) => {
                    self.compile_at_rule(at_rule, true)?;
                }
                _ => {
                    // Fallback for other statements
                    // We can't easily call compile_statement here without defining it in a trait.
                    // But usually nested items are limited.
                    // Import? MixinDefinition?
                    match nested {
                        Statement::MixinDefinition(mixin) => self.compile_mixin_definition(mixin)?,
                        Statement::Import(_) => {
                             // Imports inside rules are allowed in LESS but often bubble up.
                             // For now, ignore or implement if needed.
                             // Compiler::compile_statement handles it.
                             // We need to call back to something that handles all statements.
                             // But we can't see compile_statement.
                             // Let's assume we implement a `StatementCompiler` trait later or expose it.
                             // For now, if we encounter unknown, we might fail or just warn.
                             // But wait, compile_statement IS needed.
                        }
                        _ => {}
                    }
                }
            }
        }

        // Restore previous selectors
        self.current_selectors = previous_selectors;

        // Pop the scope when done with this rule
        self.pop_scope();

        Ok(())
    }

    fn compile_selector(
        &mut self,
        selector: &Selector,
        parent_selectors: &[String],
    ) -> Result<String> {
        // Clone and strip :extend pseudo-classes
        let mut clean_selector = selector.clone();
        for part in &mut clean_selector.parts {
            part.simple_selectors.retain(|s| {
                !matches!(s, SimpleSelector::PseudoClass { name, .. } if name == "extend")
            });
        }
        
        // Remove empty parts
        clean_selector.parts.retain(|part| !part.simple_selectors.is_empty());
        
        if clean_selector.parts.is_empty() {
            return Ok(String::new());
        }

        let mut selector_str = String::new();

        // Compile each part of the selector, handling interpolation
        for (part_idx, part) in clean_selector.parts.iter().enumerate() {
            for (sel_idx, simple_selector) in part.simple_selectors.iter().enumerate() {
                if sel_idx > 0 {
                    // No space between simple selectors like .class#id
                }

                match simple_selector {
                    SimpleSelector::Interpolation { variable, .. } => {
                        // Resolve variable interpolation
                        let var_value = self.resolve_variable(variable)?;
                        let interpolated_value = self.evaluate_expression_to_string(&var_value)?;
                        selector_str.push_str(&interpolated_value);
                    }
                    _ => {
                        selector_str.push_str(&simple_selector.to_css());
                    }
                }
            }

            // Append combinator if not last part
            if part_idx < clean_selector.parts.len() - 1 {
                if let Some(combinator) = &part.combinator {
                    if matches!(combinator, Combinator::Descendant) {
                        selector_str.push(' ');
                    } else {
                        selector_str.push_str(&format!(" {} ", combinator.to_css()));
                    }
                } else {
                    selector_str.push(' ');
                }
            }
        }

        if clean_selector.has_parent_reference() {
            // Handle parent selector (&)
            if parent_selectors.is_empty() {
                return Err(Error::compilation_error(
                    "Parent selector (&) used without parent context",
                ));
            }

            let mut result = String::new();
            for (i, parent) in parent_selectors.iter().enumerate() {
                if i > 0 {
                    result.push_str(", ");
                }

                // Replace & with parent selector
                if selector_str.starts_with('&') {
                    // Direct parent replacement: &:hover -> .button:hover
                    let rest = &selector_str[1..];
                    result.push_str(&format!("{}{}", parent, rest));
                } else {
                    // & in middle of selector
                    result.push_str(&selector_str.replace('&', parent));
                }
            }
            Ok(result)
        } else if parent_selectors.is_empty() {
            Ok(selector_str)
        } else {
            // Nested selector without &
            let mut result = String::new();
            for (i, parent) in parent_selectors.iter().enumerate() {
                if i > 0 {
                    result.push_str(", ");
                }
                result.push_str(&format!("{} {}", parent, selector_str));
            }
            Ok(result)
        }
    }

    fn compile_declaration(&mut self, declaration: &Declaration) -> Result<()> {
        self.add_indent();
        self.output.push_str(&declaration.property);
        self.output.push(':');
        self.add_space();

        let value = self.evaluate_expression(&declaration.value)?;
        self.output.push_str(&value.to_css());

        if declaration.important {
            self.add_space();
            self.output.push_str("!important");
        }

        self.output.push(';');
        self.add_newline();

        Ok(())
    }

    fn compile_comment(&mut self, comment: &Comment) -> Result<()> {
        match comment.comment_type {
            CommentType::Block => {
                self.add_indent();
                self.output.push_str("/*");
                self.output.push_str(&comment.content);
                self.output.push_str("*/");
                self.add_newline();
            }
            CommentType::Line => {
                // Line comments are not preserved in CSS output
            }
        }
        Ok(())
    }

    fn create_rule_for_current_selectors(
        &self,
        statements: &mut Vec<Statement>,
        declarations: &[Declaration],
        position: &Position,
    ) -> Result<()> {
        for current_sel in &self.current_selectors {
            use crate::ast::selectors::*;
            let selector_obj = Selector {
                parts: vec![SelectorPart {
                    simple_selectors: vec![SimpleSelector::Type {
                        name: current_sel.clone(),
                        position: position.clone(),
                    }],
                    combinator: None,
                    position: position.clone(),
                }],
                position: position.clone(),
            };

            let rule = Rule::new(vec![selector_obj], position.clone())
                .with_declarations(declarations.to_vec());

            statements.push(Statement::Rule(rule));
        }
        Ok(())
    }
}
