use super::expression::ExpressionCompiler;
use super::rule::RuleCompiler;
use super::{Compiler, PendingAtRuleMapping};
use crate::ast::*;
use crate::error::Result;

fn at_rule_mapping_name(at_rule: &AtRule) -> String {
    let mut name = format!("@{}", at_rule.name);
    if let Some(prelude) = &at_rule.prelude {
        let prelude = prelude.trim();
        if !prelude.is_empty() {
            name.push(' ');
            name.push_str(prelude);
        }
    }
    name
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'
}

fn find_top_level_separator(chars: &[char], from: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut idx = from;

    while idx < chars.len() {
        match chars[idx] {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return Some(idx),
            'a' if depth == 0 && idx + 2 < chars.len() => {
                if chars[idx + 1] == 'n' && chars[idx + 2] == 'd' {
                    let prev_ok = idx == 0 || chars[idx - 1].is_whitespace();
                    let next_ok = idx + 3 >= chars.len() || chars[idx + 3].is_whitespace();
                    if prev_ok && next_ok {
                        return Some(idx);
                    }
                }
            }
            _ => {}
        }
        idx += 1;
    }

    None
}

fn media_feature_mappings(prelude: &str) -> Vec<(usize, usize)> {
    let normalized = prelude.trim();
    if normalized.is_empty() {
        return Vec::new();
    }

    let chars: Vec<char> = normalized.chars().collect();
    let len = chars.len();
    let mut mappings = Vec::new();
    let mut depth = 0usize;

    for i in 0..len {
        match chars[i] {
            '(' => {
                if depth == 0 {
                    // Top-level media feature: map generated position right after '('
                    // to the current item separator (',' or top-level 'and') or trailing '{'.
                    let next_separator = find_top_level_separator(&chars, i + 1);

                    let generated_col_delta = 8 + i;
                    let source_col_delta = next_separator
                        .map(|separator_idx| 7 + separator_idx)
                        .unwrap_or_else(|| 8 + len);
                    mappings.push((generated_col_delta, source_col_delta));
                } else if i > 0 && is_identifier_char(chars[i - 1]) {
                    // Function call inside media feature (e.g. calc(...)):
                    // less.js maps most function-name starts to themselves.
                    // Special-case: url(...) maps to the first character after '('.
                    let mut start = i - 1;
                    while start > 0 && is_identifier_char(chars[start - 1]) {
                        start -= 1;
                    }
                    let function_name: String = chars[start..i].iter().collect();
                    let (generated_col_delta, source_col_delta) =
                        if function_name.eq_ignore_ascii_case("url") {
                            (8 + i, 8 + i)
                        } else {
                            (7 + start, 7 + start)
                        };
                    mappings.push((generated_col_delta, source_col_delta));
                }
                depth += 1;
            }
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }

    mappings
}

fn should_bubble_conditional_at_rule(name: &str) -> bool {
    name == "supports"
}

impl Compiler {
    fn transform_conditional_at_rule_for_current_selectors(
        &mut self,
        at_rule: &AtRule,
    ) -> Result<AtRule> {
        let Some(block) = &at_rule.block else {
            return Ok(at_rule.clone());
        };

        let mut transformed_statements = Vec::new();
        let mut current_declarations = Vec::new();

        for statement in block {
            match statement {
                Statement::Variable(var) => {
                    let value = self.evaluate_expression(&var.value)?;
                    self.current_scope()
                        .define_variable(var.name.clone(), value);
                }
                Statement::Declaration(decl) => {
                    current_declarations.push(decl.clone());
                }
                Statement::Rule(rule) => {
                    if !current_declarations.is_empty() {
                        self.create_rule_for_current_selectors(
                            &mut transformed_statements,
                            &current_declarations,
                            &at_rule.position,
                        )?;
                        current_declarations.clear();
                    }
                    self.process_nested_rule_in_media(rule, &mut transformed_statements)?;
                }
                Statement::AtRule(nested_at_rule)
                    if should_bubble_conditional_at_rule(&nested_at_rule.name) =>
                {
                    if !current_declarations.is_empty() {
                        self.create_rule_for_current_selectors(
                            &mut transformed_statements,
                            &current_declarations,
                            &at_rule.position,
                        )?;
                        current_declarations.clear();
                    }
                    let transformed_nested =
                        self.transform_conditional_at_rule_for_current_selectors(nested_at_rule)?;
                    transformed_statements.push(Statement::AtRule(transformed_nested));
                }
                _ => {
                    transformed_statements.push(statement.clone());
                }
            }
        }

        if !current_declarations.is_empty() {
            self.create_rule_for_current_selectors(
                &mut transformed_statements,
                &current_declarations,
                &at_rule.position,
            )?;
        }

        let mut transformed_at_rule = at_rule.clone();
        transformed_at_rule.block = Some(transformed_statements);
        Ok(transformed_at_rule)
    }
}

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

    /// 编译嵌套在规则内的条件性 at-rule（@supports 等），实现选择器冒泡
    fn compile_nested_conditional_at_rule(&mut self, at_rule: &AtRule) -> Result<()>;

    /// 输出所有挂起的媒体查询
    fn output_pending_media_queries(&mut self) -> Result<()>;
}

impl AtRuleCompiler for Compiler {
    fn compile_at_rule(&mut self, at_rule: &AtRule, is_nested: bool) -> Result<()> {
        // 特殊处理媒体查询
        if at_rule.name == "media" {
            return self.compile_media_query(at_rule, is_nested);
        }

        // Conditional at-rules (e.g. @supports) get bubble-up treatment when nested in rules.
        if should_bubble_conditional_at_rule(&at_rule.name) && !self.current_selectors.is_empty() {
            return self.compile_nested_conditional_at_rule(at_rule);
        }

        self.add_indent();
        // Source map: map at-rule header to source position with stable token name.
        let mapping_name = at_rule_mapping_name(at_rule);
        self.add_mapping(&at_rule.position, Some(&mapping_name));
        self.write_char('@');
        self.write_str(&at_rule.name);

        if let Some(prelude) = &at_rule.prelude {
            self.add_space();
            self.write_str(prelude);
        }

        if let Some(block) = &at_rule.block {
            self.add_space();
            self.write_char('{');
            self.add_newline();

            self.indent_level += 1;
            for statement in block {
                self.compile_statement(statement)?;
            }
            self.indent_level -= 1;

            self.add_indent();
            self.write_char('}');
        } else {
            self.write_char(';');
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
        let mapping_name = at_rule_mapping_name(at_rule);
        self.add_mapping(&at_rule.position, Some(&mapping_name));
        if self.source_map_lessjs_compat && !current_condition.is_empty() {
            for (generated_col_delta, source_col_delta) in media_feature_mappings(current_condition) {
                let mut source_pos = at_rule.position.clone();
                source_pos.column = source_pos.column.saturating_add(source_col_delta);
                let gen_col = self.current_col.saturating_add(generated_col_delta as u32);
                self.add_mapping_at_generated_col(&source_pos, Some(&mapping_name), gen_col);
            }
        }
        self.write_str("@media");
        if !current_condition.is_empty() {
            self.add_space();
            self.write_str(current_condition);
        }

        if let Some(block) = &at_rule.block {
            self.add_space();
            self.write_char('{');
            self.add_newline();

            self.indent_level += 1;
            for statement in block {
                self.compile_statement(statement)?;
            }
            self.indent_level -= 1;

            self.add_indent();
            self.write_char('}');
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
                        self.current_scope()
                            .define_variable(var.name.clone(), value);
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
                                            std::slice::from_ref(decl),
                                            &nested_at_rule.position,
                                        )?;
                                    }
                                    Statement::Rule(nested_rule) => {
                                        self.process_nested_rule_in_media(
                                            nested_rule,
                                            &mut nested_transformed_statements,
                                        )?;
                                    }
                                    Statement::AtRule(nested_conditional)
                                        if should_bubble_conditional_at_rule(
                                            &nested_conditional.name,
                                        ) =>
                                    {
                                        let transformed_conditional = self
                                            .transform_conditional_at_rule_for_current_selectors(
                                                nested_conditional,
                                            )?;
                                        nested_transformed_statements
                                            .push(Statement::AtRule(transformed_conditional));
                                    }
                                    _ => {
                                        nested_transformed_statements
                                            .push(nested_statement.clone());
                                    }
                                }
                            }

                            let nested_mapping = PendingAtRuleMapping {
                                source_file: self.current_file.clone(),
                                position: nested_at_rule.position.clone(),
                                name: at_rule_mapping_name(nested_at_rule),
                                media_feature_mappings: nested_at_rule
                                    .prelude
                                    .as_ref()
                                    .map(|p| media_feature_mappings(p))
                                    .unwrap_or_default(),
                            };
                            self.queue_pending_media_query(
                                nested_media_query,
                                nested_transformed_statements,
                                Some(nested_mapping),
                            );
                        }
                    }
                    Statement::AtRule(nested_at_rule)
                        if should_bubble_conditional_at_rule(&nested_at_rule.name) =>
                    {
                        if !current_declarations.is_empty() {
                            self.create_rule_for_current_selectors(
                                &mut transformed_statements,
                                &current_declarations,
                                &at_rule.position,
                            )?;
                            current_declarations.clear();
                        }
                        let transformed_nested = self
                            .transform_conditional_at_rule_for_current_selectors(nested_at_rule)?;
                        transformed_statements.push(Statement::AtRule(transformed_nested));
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

            let mapping = PendingAtRuleMapping {
                source_file: self.current_file.clone(),
                position: at_rule.position.clone(),
                name: at_rule_mapping_name(at_rule),
                media_feature_mappings: at_rule
                    .prelude
                    .as_ref()
                    .map(|p| media_feature_mappings(p))
                    .unwrap_or_default(),
            };
            self.queue_pending_media_query(media_query, transformed_statements, Some(mapping));
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
                        self.current_scope()
                            .define_variable(var.name.clone(), value);
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
                    Statement::AtRule(nested_at_rule)
                        if should_bubble_conditional_at_rule(&nested_at_rule.name) =>
                    {
                        if !current_declarations.is_empty() {
                            self.create_rule_for_current_selectors(
                                &mut transformed_statements,
                                &current_declarations,
                                &at_rule.position,
                            )?;
                            current_declarations.clear();
                        }
                        let transformed_nested = self
                            .transform_conditional_at_rule_for_current_selectors(nested_at_rule)?;
                        transformed_statements.push(Statement::AtRule(transformed_nested));
                    }
                    _ => {
                        transformed_statements.push(statement.clone());
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

            let mapping = PendingAtRuleMapping {
                source_file: self.current_file.clone(),
                position: at_rule.position.clone(),
                name: at_rule_mapping_name(at_rule),
                media_feature_mappings: at_rule
                    .prelude
                    .as_ref()
                    .map(|p| media_feature_mappings(p))
                    .unwrap_or_default(),
            };
            self.queue_pending_media_query(media_query, transformed_statements, Some(mapping));
        }
        Ok(())
    }

    fn compile_nested_conditional_at_rule(&mut self, at_rule: &AtRule) -> Result<()> {
        // Similar to compile_nested_media_query_simple but for @supports and similar
        // conditional at-rules. Bubbles the selector into the at-rule block.
        if let Some(block) = &at_rule.block {
            let mut at_rule_header = format!("@{}", at_rule.name);
            if let Some(prelude) = &at_rule.prelude {
                at_rule_header.push(' ');
                at_rule_header.push_str(prelude);
            }

            let mut transformed_statements = Vec::new();
            let mut current_declarations = Vec::new();

            for statement in block {
                match statement {
                    Statement::Variable(var) => {
                        let value = self.evaluate_expression(&var.value)?;
                        self.current_scope()
                            .define_variable(var.name.clone(), value);
                    }
                    Statement::Declaration(decl) => {
                        current_declarations.push(decl.clone());
                    }
                    Statement::Rule(rule) => {
                        if !current_declarations.is_empty() {
                            self.create_rule_for_current_selectors(
                                &mut transformed_statements,
                                &current_declarations,
                                &at_rule.position,
                            )?;
                            current_declarations.clear();
                        }
                        self.process_nested_rule_in_media(rule, &mut transformed_statements)?;
                    }
                    _ => {
                        transformed_statements.push(statement.clone());
                    }
                }
            }

            if !current_declarations.is_empty() {
                self.create_rule_for_current_selectors(
                    &mut transformed_statements,
                    &current_declarations,
                    &at_rule.position,
                )?;
            }

            // Use pending_media_queries to defer output (same mechanism as @media)
            let mapping = PendingAtRuleMapping {
                source_file: self.current_file.clone(),
                position: at_rule.position.clone(),
                name: at_rule_mapping_name(at_rule),
                media_feature_mappings: Vec::new(),
            };
            self.queue_pending_media_query(at_rule_header, transformed_statements, Some(mapping));
        }
        Ok(())
    }

    fn output_pending_media_queries(&mut self) -> Result<()> {
        let pending = std::mem::take(&mut self.pending_media_queries);
        for query in pending {
            self.emit_pending_media_query(query)?;
        }
        Ok(())
    }
}
