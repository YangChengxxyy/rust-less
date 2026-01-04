use crate::ast::*;
use crate::error::Result;
use super::Compiler;
use super::expression::ExpressionCompiler;
use super::rule::RuleCompiler;

/// At-Rule 编译特性
pub trait AtRuleCompiler {
    /// 编译 At-Rule
    fn compile_at_rule(&mut self, at_rule: &AtRule, is_nested: bool) -> Result<()>;
    
    /// 编译媒体查询
    fn compile_media_query(&mut self, at_rule: &AtRule, is_nested: bool) -> Result<()>;
    
    /// 编译嵌套媒体查询并合并条件
    fn compile_nested_media_query_with_merging(&mut self, at_rule: &AtRule) -> Result<()>;
    
    /// 处理媒体查询中的嵌套规则
    fn process_nested_rule_in_media(
        &self,
        rule: &Rule,
        statements: &mut Vec<Statement>,
    ) -> Result<()>;
    
    /// 编译嵌套媒体查询（简单实现）
    fn compile_nested_media_query_simple(&mut self, at_rule: &AtRule) -> Result<()>;
    
    /// 输出所有挂起的媒体查询
    fn output_pending_media_queries(&mut self) -> Result<()>;
}

impl AtRuleCompiler for Compiler {
    fn compile_at_rule(&mut self, at_rule: &AtRule, is_nested: bool) -> Result<()> {
        // 特殊处理媒体查询
        if at_rule.name == "media" {
            return self.compile_media_query(at_rule, is_nested);
        }

        self.add_indent();
        self.output.push('@');
        self.output.push_str(&at_rule.name);

        if let Some(prelude) = &at_rule.prelude {
            self.add_space();
            self.output.push_str(prelude);
        }

        if let Some(block) = &at_rule.block {
            self.add_space();
            self.output.push('{');
            self.add_newline();

            self.indent_level += 1;
            for statement in block {
                self.compile_statement(statement)?;
            }
            self.indent_level -= 1;

            self.add_indent();
            self.output.push('}');
        } else {
            self.output.push(';');
        }

        self.add_newline();
        Ok(())
    }

    fn compile_media_query(&mut self, at_rule: &AtRule, is_nested: bool) -> Result<()> {
        let current_condition = at_rule.prelude.as_deref().unwrap_or("").trim();

        if is_nested && !self.media_query_stack.is_empty() {
            // 嵌套媒体查询：合并条件
            return self.compile_nested_media_query_with_merging(at_rule);
        }

        if !self.current_selectors.is_empty() {
            // CSS规则内的媒体查询：使用现有逻辑
            return self.compile_nested_media_query_simple(at_rule);
        }

        // 顶级媒体查询：推入上下文栈并正常处理
        self.media_query_stack.push(current_condition.to_string());

        self.add_indent();
        self.output.push_str("@media");
        if !current_condition.is_empty() {
            self.add_space();
            self.output.push_str(current_condition);
        }

        if let Some(block) = &at_rule.block {
            self.add_space();
            self.output.push('{');
            self.add_newline();

            self.indent_level += 1;
            for statement in block {
                self.compile_statement(statement)?;
            }
            self.indent_level -= 1;

            self.add_indent();
            self.output.push('}');
        }

        self.add_newline();

        // 处理完成后从栈中移除
        self.media_query_stack.pop();
        Ok(())
    }

    fn compile_nested_media_query_with_merging(&mut self, at_rule: &AtRule) -> Result<()> {
        let current_condition = at_rule.prelude.as_deref().unwrap_or("").trim();

        // 合并所有父级媒体查询条件
        let mut combined_conditions = self.media_query_stack.clone();
        if !current_condition.is_empty() {
            combined_conditions.push(current_condition.to_string());
        }

        // 使用 "and" 连接所有条件
        let merged_condition = combined_conditions
            .iter()
            .filter(|c| !c.is_empty())
            .map(|c| {
                format!(
                    "({})",
                    c.trim().trim_start_matches('(').trim_end_matches(')')
                )
            })
            .collect::<Vec<_>>()
            .join(" and ");

        if let Some(block) = &at_rule.block {
            let media_query = if merged_condition.is_empty() {
                "@media".to_string()
            } else {
                format!("@media {}", merged_condition)
            };

            // 处理块内容
            let mut transformed_statements = Vec::new();
            let mut current_declarations = Vec::new();

            for statement in block {
                match statement {
                    Statement::Variable(var) => {
                        // Manually compile variable declaration
                        let value = self.evaluate_expression(&var.value)?;
                        self.current_scope().define_variable(var.name.clone(), value);
                    }
                    Statement::Declaration(decl) => {
                        current_declarations.push(decl.clone());
                    }
                    Statement::Rule(rule) => {
                        // 处理待处理的声明
                        if !current_declarations.is_empty() {
                            self.create_rule_for_current_selectors(
                                &mut transformed_statements,
                                &current_declarations,
                                &at_rule.position,
                            )?;
                            current_declarations.clear();
                        }

                        // 处理嵌套规则
                        self.process_nested_rule_in_media(rule, &mut transformed_statements)?;
                    }
                    Statement::AtRule(nested_at_rule) if nested_at_rule.name == "media" => {
                        // 处理嵌套的媒体查询 - 递归合并条件
                        let nested_condition =
                            nested_at_rule.prelude.as_deref().unwrap_or("").trim();

                        // 创建新的合并条件
                        let mut nested_combined_conditions = combined_conditions.clone();
                        if !nested_condition.is_empty() {
                            nested_combined_conditions.push(nested_condition.to_string());
                        }

                        let nested_merged_condition = nested_combined_conditions
                            .iter()
                            .filter(|c| !c.is_empty())
                            .map(|c| {
                                format!(
                                    "({})",
                                    c.trim().trim_start_matches('(').trim_end_matches(')')
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(" and ");

                        if let Some(nested_block) = &nested_at_rule.block {
                            let nested_media_query = if nested_merged_condition.is_empty() {
                                "@media".to_string()
                            } else {
                                format!("@media {}", nested_merged_condition)
                            };

                            // 处理嵌套媒体查询的内容
                            let mut nested_transformed_statements = Vec::new();
                            for nested_statement in nested_block {
                                match nested_statement {
                                    Statement::Declaration(decl) => {
                                        self.create_rule_for_current_selectors(
                                            &mut nested_transformed_statements,
                                            &[decl.clone()],
                                            &nested_at_rule.position,
                                        )?;
                                    }
                                    Statement::Rule(nested_rule) => {
                                        self.process_nested_rule_in_media(
                                            nested_rule,
                                            &mut nested_transformed_statements,
                                        )?;
                                    }
                                    _ => {
                                        nested_transformed_statements
                                            .push(nested_statement.clone());
                                    }
                                }
                            }

                            self.pending_media_queries
                                .push((nested_media_query, nested_transformed_statements));
                        }
                    }
                    _ => {
                        transformed_statements.push(statement.clone());
                    }
                }
            }

            // 处理剩余的声明
            if !current_declarations.is_empty() {
                self.create_rule_for_current_selectors(
                    &mut transformed_statements,
                    &current_declarations,
                    &at_rule.position,
                )?;
            }

            self.pending_media_queries
                .push((media_query, transformed_statements));
        }

        Ok(())
    }

    fn process_nested_rule_in_media(
        &self,
        rule: &Rule,
        statements: &mut Vec<Statement>,
    ) -> Result<()> {
        let rule_sel_strings: Vec<String> = rule.selectors.iter().map(|s| s.to_css()).collect();

        if self.current_selectors.is_empty() {
            // 没有当前选择器上下文，直接使用规则选择器
            statements.push(Statement::Rule(rule.clone()));
        } else {
            // 有当前选择器上下文，需要组合选择器
            for current_sel in &self.current_selectors {
                for rule_sel_str in &rule_sel_strings {
                    let combined_selector = format!("{} {}", current_sel, rule_sel_str);

                    use crate::ast::selectors::*;
                    let combined_selector_obj = Selector {
                        parts: vec![SelectorPart {
                            simple_selectors: vec![SimpleSelector::Type {
                                name: combined_selector,
                                position: rule.position.clone(),
                            }],
                            combinator: None,
                            position: rule.position.clone(),
                        }],
                        position: rule.position.clone(),
                    };

                    let new_rule = Rule::new(vec![combined_selector_obj], rule.position.clone())
                        .with_declarations(rule.declarations.clone())
                        .with_nested_rules(rule.nested_rules.clone());

                    statements.push(Statement::Rule(new_rule));
                }
            }
        }
        Ok(())
    }

    fn compile_nested_media_query_simple(&mut self, at_rule: &AtRule) -> Result<()> {
        if let Some(block) = &at_rule.block {
            let mut media_query = String::from("@media");
            if let Some(prelude) = &at_rule.prelude {
                media_query.push(' ');
                media_query.push_str(prelude);
            }

            // Transform nested content to work with current selector context
            let mut transformed_statements = Vec::new();

            // Collect any declarations and wrap them in rules with current selectors
            let mut current_declarations = Vec::new();

            for statement in block {
                match statement {
                    Statement::Variable(var) => {
                        // Process variables in scope but don't add to output
                        // Use evaluate_expression directly as we don't have compile_variable_declaration
                         let value = self.evaluate_expression(&var.value)?;
                         self.current_scope().define_variable(var.name.clone(), value);
                    }
                    Statement::Declaration(decl) => {
                        // Collect declarations to be wrapped with current selectors
                        current_declarations.push(decl.clone());
                    }
                    Statement::Rule(rule) => {
                        // First, finish any pending declarations
                        if !current_declarations.is_empty() {
                            self.create_rule_for_current_selectors(
                                &mut transformed_statements,
                                &current_declarations,
                                &at_rule.position,
                            )?;
                            current_declarations.clear();
                        }

                        // Handle nested rules by combining selectors
                        // Pre-compute selector strings to avoid borrowing issues
                        let mut rule_sel_strings = Vec::new();
                        for rule_sel in &rule.selectors {
                            rule_sel_strings.push(rule_sel.to_css());
                        }

                        for current_sel in &self.current_selectors {
                            for rule_sel_str in &rule_sel_strings {
                                let combined_selector = format!("{} {}", current_sel, rule_sel_str);

                                // Create selector for combined path
                                use crate::ast::selectors::*;
                                let combined_selector_obj = Selector {
                                    parts: vec![SelectorPart {
                                        simple_selectors: vec![SimpleSelector::Type {
                                            name: combined_selector,
                                            position: rule.position.clone(),
                                        }],
                                        combinator: None,
                                        position: rule.position.clone(),
                                    }],
                                    position: rule.position.clone(),
                                };

                                let new_rule =
                                    Rule::new(vec![combined_selector_obj], rule.position.clone())
                                        .with_declarations(rule.declarations.clone())
                                        .with_nested_rules(rule.nested_rules.clone());

                                transformed_statements.push(Statement::Rule(new_rule));
                            }
                        }
                    }
                    _ => {
                        // Check if this is a declaration at the rule level
                        if let Statement::Rule(_rule) = statement {
                            // This case is already handled above
                        } else {
                            // For direct declarations in the media query, we need to extract them
                            // This requires walking through the rule structure to find declarations
                            // For now, add the statement as-is and handle in the parser
                            transformed_statements.push(statement.clone());
                        }
                    }
                }
            }

            // Handle any remaining declarations
            if !current_declarations.is_empty() {
                self.create_rule_for_current_selectors(
                    &mut transformed_statements,
                    &current_declarations,
                    &at_rule.position,
                )?;
            }

            self.pending_media_queries
                .push((media_query, transformed_statements));
        }
        Ok(())
    }

    fn output_pending_media_queries(&mut self) -> Result<()> {
        let pending = std::mem::take(&mut self.pending_media_queries);
        for (media_query, statements) in pending {
            self.output.push_str(&media_query);
            self.add_space();
            self.output.push('{');
            self.add_newline();

            self.indent_level += 1;
            for statement in &statements {
                self.compile_statement(statement)?;
            }
            self.indent_level -= 1;

            self.add_indent();
            self.output.push('}');
            self.add_newline();
        }
        Ok(())
    }
}
