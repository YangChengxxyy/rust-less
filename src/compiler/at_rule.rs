use super::expression::ExpressionCompiler;
use super::mixin::MixinCompiler;
use super::rule::RuleCompiler;
use super::{Compiler, PendingAtRuleMapping};
use crate::ast::*;
use crate::error::{Error, Result};

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
            'a' if depth == 0
                && idx + 2 < chars.len()
                && chars[idx + 1] == 'n'
                && chars[idx + 2] == 'd' =>
            {
                let prev_ok = idx == 0 || chars[idx - 1].is_whitespace();
                let next_ok = idx + 3 >= chars.len() || chars[idx + 3].is_whitespace();
                if prev_ok && next_ok {
                    return Some(idx);
                }
            }
            _ => {}
        }
        idx += 1;
    }

    None
}

/// Scan a media prelude for (generated, source) column delta pairs (offsets
/// from the `@media` keyword start), following less.js header mapping rules.
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
                    // Top-level media feature: generated position right after
                    // '(' maps to the current item separator or trailing end.
                    let next_separator = find_top_level_separator(&chars, i + 1);
                    let source_delta = next_separator
                        .map(|separator_idx| 7 + separator_idx)
                        .unwrap_or_else(|| 8 + len);
                    mappings.push((8 + i, source_delta));
                } else if i > 0 && is_identifier_char(chars[i - 1]) {
                    // Function call inside media feature (e.g. calc(...)):
                    // less.js maps most function-name starts to themselves.
                    let mut start = i - 1;
                    while start > 0 && is_identifier_char(chars[start - 1]) {
                        start -= 1;
                    }
                    let function_name: String = chars[start..i].iter().collect();
                    if function_name.eq_ignore_ascii_case("url") {
                        mappings.push((8 + i, 8 + i));
                    } else {
                        mappings.push((7 + start, 7 + start));
                    }
                }
                depth += 1;
            }
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }

    mappings
}

/// Build (generated, source) column deltas when the emitted prelude differs
/// from the source prelude (variables resolved). Generated offsets track the
/// emitted text; source offsets track the original text.
fn media_feature_mappings_resolved(emitted: &str, source: &str) -> Vec<(usize, usize)> {
    if emitted == source {
        return media_feature_mappings(emitted);
    }
    let generated = media_feature_mappings(emitted);
    let original = media_feature_mappings(source);
    generated
        .into_iter()
        .zip(original)
        .map(|((gen, _), (_, src))| (gen, src))
        .collect()
}

fn should_bubble_conditional_at_rule(name: &str) -> bool {
    name == "supports"
}

impl Compiler {
    /// Resolve LESS variables (`@var`, `@{var}`, `@map[key][nested]`) inside an
    /// at-rule prelude, matching less.js behavior of evaluating variables in
    /// media/supports conditions before emission.
    fn resolve_prelude_variables(&mut self, prelude: &str) -> Result<String> {
        if !prelude.contains('@') {
            return Ok(prelude.to_string());
        }

        let chars: Vec<char> = prelude.chars().collect();
        let len = chars.len();
        let mut out = String::new();
        let mut i = 0;

        while i < len {
            if chars[i] != '@' {
                out.push(chars[i]);
                i += 1;
                continue;
            }

            // Interpolation: @{name}
            if i + 1 < len && chars[i + 1] == '{' {
                let mut j = i + 2;
                while j < len && chars[j] != '}' {
                    j += 1;
                }
                if j >= len {
                    // Unterminated interpolation: emit literally
                    out.push('@');
                    i += 1;
                    continue;
                }
                let name: String = chars[i + 2..j].iter().collect();
                let value = self.resolve_variable(name.trim())?;
                out.push_str(&self.evaluate_expression_to_string(&value)?);
                i = j + 1;
                continue;
            }

            // Plain variable: @name
            let mut j = i + 1;
            while j < len && is_identifier_char(chars[j]) {
                j += 1;
            }
            if j == i + 1 {
                out.push('@');
                i += 1;
                continue;
            }
            let name: String = chars[i + 1..j].iter().collect();
            let mut expr = Expression::variable(name, Position::default());

            // Map access chain: [key][nested]...
            while j < len && chars[j] == '[' {
                let mut k = j + 1;
                let mut quote: Option<char> = None;
                while k < len {
                    if let Some(q) = quote {
                        if chars[k] == q {
                            quote = None;
                        }
                    } else if chars[k] == '"' || chars[k] == '\'' {
                        quote = Some(chars[k]);
                    } else if chars[k] == ']' {
                        break;
                    }
                    k += 1;
                }
                if k >= len {
                    break; // Unterminated bracket: emit what we have
                }
                let raw_key: String = chars[j + 1..k].iter().collect();
                let trimmed = raw_key.trim();
                let key_expr = if (trimmed.starts_with('"')
                    && trimmed.ends_with('"')
                    && trimmed.len() >= 2)
                    || (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2)
                {
                    Expression::string(
                        trimmed[1..trimmed.len() - 1].to_string(),
                        Position::default(),
                    )
                } else {
                    Expression::identifier(trimmed.to_string(), Position::default())
                };
                expr = Expression::MapAccess {
                    map: Box::new(expr),
                    key: Box::new(key_expr),
                    position: Position::default(),
                };
                j = k + 1;
            }

            let resolved = self.evaluate_expression_to_string(&expr)?;
            out.push_str(&resolved);
            i = j;
        }

        Ok(out)
    }

    /// Lower a statement for deferred media/supports emission: variables and
    /// declarations are evaluated eagerly (so mixin parameters stay bound),
    /// rules are lowered recursively, other statements pass through cloned.
    fn lower_statement_for_media(
        &mut self,
        statement: &Statement,
        out: &mut Vec<Statement>,
        important: bool,
        source_file: Option<&str>,
    ) -> Result<()> {
        match statement {
            Statement::Variable(var) => {
                let value = self.evaluate_expression(&var.value)?;
                self.current_scope()
                    .define_variable(var.name.clone(), value);
            }
            Statement::Declaration(decl) => {
                let mut lowered = decl.clone();
                lowered.value = self.evaluate_expression(&decl.value)?;
                if let Some(f) = source_file {
                    lowered.source_file = Some(f.to_string());
                }
                if important {
                    lowered.important = true;
                }
                out.push(Statement::Declaration(lowered));
            }
            Statement::Rule(rule) => {
                out.push(Statement::Rule(self.lower_rule_for_media(
                    rule,
                    important,
                    source_file,
                )?));
            }
            Statement::MixinCall(call) => {
                // Already lowered with the mixin's source file attached
                for stmt in self.expand_mixin_call_to_statements(call, important)? {
                    out.push(stmt);
                }
            }
            Statement::DetachedRulesetCall(call) => {
                for stmt in self.expand_detached_ruleset_to_statements(call, important)? {
                    out.push(stmt);
                }
            }
            Statement::EachCall(each_call) => {
                for stmt in self.expand_each_call_to_statements(each_call, important)? {
                    out.push(stmt);
                }
            }
            other => out.push(other.clone()),
        }
        Ok(())
    }

    /// Lower a rule for deferred media emission: declaration values are
    /// evaluated eagerly and nested rules are lowered recursively.
    fn lower_rule_for_media(
        &mut self,
        rule: &Rule,
        important: bool,
        source_file: Option<&str>,
    ) -> Result<Rule> {
        let mut declarations = Vec::new();
        for decl in &rule.declarations {
            let mut lowered = decl.clone();
            lowered.value = self.evaluate_expression(&decl.value)?;
            if let Some(f) = source_file {
                lowered.source_file = Some(f.to_string());
            }
            if important {
                lowered.important = true;
            }
            declarations.push(lowered);
        }

        let mut nested_rules = Vec::new();
        for nested in &rule.nested_rules {
            self.lower_statement_for_media(nested, &mut nested_rules, important, source_file)?;
        }

        Ok(Rule::new(rule.selectors.clone(), rule.position.clone())
            .with_declarations(declarations)
            .with_nested_rules(nested_rules))
    }

    /// Expand a mixin call into lowered statements (parameters bound in scope).
    /// Declarations are tagged with the mixin definition's source file for
    /// source-map attribution.
    fn expand_mixin_call_to_statements(
        &mut self,
        call: &MixinCall,
        important: bool,
    ) -> Result<Vec<Statement>> {
        let mixins = self.resolve_mixin_path(&call.name)?;
        if mixins.is_empty() {
            return Err(Error::undefined_mixin(
                &call.name,
                call.position.line,
                call.position.column,
            ));
        }
        let mixin_def = self.find_matching_mixin(&mixins, call)?.ok_or_else(|| {
            Error::semantic_error(
                format!("No matching guard for mixin '{}'", call.name),
                call.position.line,
                call.position.column,
            )
        })?;

        if self.recursion_depth >= self.max_recursion_depth {
            return Err(Error::infinite_recursion(
                format!("Mixin '{}'", call.name),
                call.position.line,
                call.position.column,
            ));
        }
        self.recursion_depth += 1;

        let result = (|| -> Result<Vec<Statement>> {
            let parent = self.current_scope().clone();
            let mut mixin_scope = Scope::with_parent(parent);
            self.bind_mixin_arguments(&mixin_def, call, &mut mixin_scope)?;
            self.scope_stack.push(mixin_scope);
            self.pre_scan_variables(&mixin_def.body);

            let body = mixin_def.body.clone();
            let source_file = mixin_def.source_file.clone();
            let important = important || call.important;
            let mut out = Vec::new();
            let expand_result = (|| -> Result<()> {
                for stmt in &body {
                    self.lower_statement_for_media(
                        stmt,
                        &mut out,
                        important,
                        source_file.as_deref(),
                    )?;
                }
                Ok(())
            })();
            self.pop_scope();
            expand_result?;
            Ok(out)
        })();

        self.recursion_depth -= 1;
        result
    }

    /// Expand a detached ruleset call into lowered statements.
    fn expand_detached_ruleset_to_statements(
        &mut self,
        call: &DetachedRulesetCall,
        important: bool,
    ) -> Result<Vec<Statement>> {
        // Follow alias chains to the terminal deferred value (@dr() works
        // through variable aliases in less.js).
        let value = self
            .resolve_deferred_value(&call.name)
            .ok_or_else(|| {
                Error::undefined_variable(&call.name, call.position.line, call.position.column)
            })?;

        let body = match &value {
            Expression::DetachedRuleset { body, .. } => body.clone(),
            // A map literal invoked as `@map()` expands its entries as
            // declarations, matching less.js where maps are rulesets.
            Expression::MapLiteral {
                entries, position, ..
            } => entries
                .iter()
                .map(|(key, value, key_position)| {
                    let _ = position;
                    Statement::Declaration(Declaration::new(
                        key.clone(),
                        value.clone(),
                        key_position.clone(),
                    ))
                })
                .collect(),
            _ => {
                return Err(Error::semantic_error(
                    format!("@{}() is not a detached ruleset", call.name),
                    call.position.line,
                    call.position.column,
                ))
            }
        };

        self.push_scope();
        let source_file = self
            .current_scope()
            .lookup_variable_file(&call.name)
            .cloned();
        let mut out = Vec::new();
        let result = (|| -> Result<()> {
            for stmt in &body {
                self.lower_statement_for_media(stmt, &mut out, important, source_file.as_deref())?;
            }
            Ok(())
        })();
        self.pop_scope();
        result?;
        Ok(out)
    }

    /// Expand an each() call into lowered statements (one copy of the template
    /// body per item, with @value/@key/@index bound).
    fn expand_each_call_to_statements(
        &mut self,
        each_call: &EachCall,
        important: bool,
    ) -> Result<Vec<Statement>> {
        if self.recursion_depth >= self.max_recursion_depth {
            return Err(Error::infinite_recursion(
                "each() call",
                each_call.position.line,
                each_call.position.column,
            ));
        }
        self.recursion_depth += 1;

        let result = (|| -> Result<Vec<Statement>> {
            let list = self.evaluate_expression(&each_call.list)?;

            let iterations: Vec<(Expression, Expression)> = match &list {
                Expression::MapLiteral { entries, .. } => entries
                    .iter()
                    .map(|(key, value, _)| {
                        (
                            Expression::identifier(key.clone(), each_call.position.clone()),
                            value.clone(),
                        )
                    })
                    .collect(),
                Expression::List { values, .. } => values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        (
                            Expression::number((index + 1) as f64, each_call.position.clone()),
                            value.clone(),
                        )
                    })
                    .collect(),
                other => vec![(
                    Expression::number(1.0, each_call.position.clone()),
                    other.clone(),
                )],
            };

            let mut out = Vec::new();
            for (index, (key_expr, value_expr)) in iterations.into_iter().enumerate() {
                self.push_scope();
                self.current_scope()
                    .define_variable("value".to_string(), value_expr);
                self.current_scope()
                    .define_variable("key".to_string(), key_expr);
                self.current_scope().define_variable(
                    "index".to_string(),
                    Expression::number((index + 1) as f64, each_call.position.clone()),
                );

                let iter_result = (|| -> Result<()> {
                    for stmt in &each_call.body {
                        self.lower_statement_for_media(stmt, &mut out, important, None)?;
                    }
                    Ok(())
                })();
                self.pop_scope();
                iter_result?;
            }
            Ok(out)
        })();

        self.recursion_depth -= 1;
        result
    }

    /// Process one statement of a nested media/supports block: declarations
    /// (and expansions of mixin/DR/each calls) are collected for wrapping with
    /// the current selectors; rules and conditional at-rules are transformed.
    fn collect_media_block_item(
        &mut self,
        statement: &Statement,
        transformed_statements: &mut Vec<Statement>,
        current_declarations: &mut Vec<Declaration>,
        position: &Position,
    ) -> Result<()> {
        match statement {
            Statement::Variable(var) => {
                let value = self.evaluate_expression(&var.value)?;
                self.current_scope()
                    .define_variable(var.name.clone(), value);
            }
            Statement::Declaration(decl) => {
                current_declarations.push(decl.clone());
            }
            Statement::MixinCall(_)
            | Statement::DetachedRulesetCall(_)
            | Statement::EachCall(_) => {
                let mut lowered = Vec::new();
                self.lower_statement_for_media(statement, &mut lowered, false, None)?;
                for stmt in lowered {
                    self.collect_media_block_item(
                        &stmt,
                        transformed_statements,
                        current_declarations,
                        position,
                    )?;
                }
            }
            Statement::Rule(rule) => {
                if !current_declarations.is_empty() {
                    self.create_rule_for_current_selectors(
                        transformed_statements,
                        current_declarations,
                        position,
                    )?;
                    current_declarations.clear();
                }
                self.process_nested_rule_in_media(rule, transformed_statements)?;
            }
            Statement::AtRule(nested_at_rule)
                if should_bubble_conditional_at_rule(&nested_at_rule.name) =>
            {
                if !current_declarations.is_empty() {
                    self.create_rule_for_current_selectors(
                        transformed_statements,
                        current_declarations,
                        position,
                    )?;
                    current_declarations.clear();
                }
                let transformed =
                    self.transform_conditional_at_rule_for_current_selectors(nested_at_rule)?;
                transformed_statements.push(Statement::AtRule(transformed));
            }
            other => {
                transformed_statements.push(other.clone());
            }
        }
        Ok(())
    }

    /// Process one statement of a nested media block inside a merging media
    /// query: each declaration is wrapped immediately, mixin/DR/each calls are
    /// expanded and processed recursively.
    fn collect_nested_media_item(
        &mut self,
        statement: &Statement,
        out: &mut Vec<Statement>,
        position: &Position,
    ) -> Result<()> {
        match statement {
            Statement::Declaration(decl) => {
                self.create_rule_for_current_selectors(out, std::slice::from_ref(decl), position)?;
            }
            Statement::Rule(rule) => {
                self.process_nested_rule_in_media(rule, out)?;
            }
            Statement::AtRule(nested_conditional)
                if should_bubble_conditional_at_rule(&nested_conditional.name) =>
            {
                let transformed =
                    self.transform_conditional_at_rule_for_current_selectors(nested_conditional)?;
                out.push(Statement::AtRule(transformed));
            }
            Statement::MixinCall(_)
            | Statement::DetachedRulesetCall(_)
            | Statement::EachCall(_) => {
                let mut lowered = Vec::new();
                self.lower_statement_for_media(statement, &mut lowered, false, None)?;
                for stmt in lowered {
                    self.collect_nested_media_item(&stmt, out, position)?;
                }
            }
            other => {
                out.push(other.clone());
            }
        }
        Ok(())
    }

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
            self.collect_media_block_item(
                statement,
                &mut transformed_statements,
                &mut current_declarations,
                &at_rule.position,
            )?;
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
    fn compile_at_rule_resolved(&mut self, at_rule: &AtRule, is_nested: bool) -> Result<()> {
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
            // Space between at-rule name and prelude is significant even in
            // compressed output (matches less.js).
            self.write_str(" ");
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

    /// Media feature mappings for the at-rule currently being compiled;
    /// anchors source columns to the original prelude when variables were
    /// resolved into the emitted text.
    fn prelude_media_mappings(&self, emitted: &str) -> Vec<(usize, usize)> {
        match &self.original_at_rule_prelude {
            Some(source) => media_feature_mappings_resolved(emitted, source),
            None => media_feature_mappings(emitted),
        }
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
        // Evaluate LESS variables in the prelude (media/supports conditions),
        // matching less.js; downstream merging/mapping sees resolved text.
        let resolved_at_rule;
        let mut original_prelude = None;
        let at_rule = match &at_rule.prelude {
            Some(prelude) if prelude.contains('@') => {
                let mut cloned = at_rule.clone();
                cloned.prelude = Some(self.resolve_prelude_variables(prelude)?);
                original_prelude = Some(prelude.clone());
                resolved_at_rule = cloned;
                &resolved_at_rule
            }
            _ => at_rule,
        };

        // While compiling this at-rule, keep the source prelude available so
        // source-map columns stay anchored to the original text when the
        // emitted prelude had variables resolved.
        let saved_original_prelude = self.original_at_rule_prelude.take();
        self.original_at_rule_prelude = original_prelude;

        let dispatch_result = self.compile_at_rule_resolved(at_rule, is_nested);

        self.original_at_rule_prelude = saved_original_prelude;
        dispatch_result
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
            for (generated_col_delta, source_col_delta) in
                self.prelude_media_mappings(current_condition)
            {
                let mut source_pos = at_rule.position.clone();
                source_pos.column = source_pos.column.saturating_add(source_col_delta);
                let gen_col = self.current_col.saturating_add(generated_col_delta as u32);
                self.add_mapping_at_generated_col(&source_pos, Some(&mapping_name), gen_col);
            }
        }
        self.write_str("@media");
        if !current_condition.is_empty() {
            // Space between at-rule name and prelude is significant even in
            // compressed output (less.js keeps `@media (...)`).
            self.write_str(" ");
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
                if !matches!(statement, Statement::AtRule(nested) if nested.name == "media") {
                    self.collect_media_block_item(
                        statement,
                        &mut transformed_statements,
                        &mut current_declarations,
                        &at_rule.position,
                    )?;
                    continue;
                }
                match statement {
                    Statement::AtRule(nested_at_rule) if nested_at_rule.name == "media" => {
                        // 处理嵌套的媒体查询 - 递归合并条件
                        let raw_nested_condition =
                            nested_at_rule.prelude.as_deref().unwrap_or("").trim();
                        let resolved_nested_condition;
                        let nested_condition = if raw_nested_condition.contains('@') {
                            resolved_nested_condition =
                                self.resolve_prelude_variables(raw_nested_condition)?;
                            resolved_nested_condition.trim()
                        } else {
                            raw_nested_condition
                        };

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
                                self.collect_nested_media_item(
                                    nested_statement,
                                    &mut nested_transformed_statements,
                                    &nested_at_rule.position,
                                )?;
                            }

                            let nested_mapping = PendingAtRuleMapping {
                                source_file: self.current_file.clone(),
                                position: nested_at_rule.position.clone(),
                                name: at_rule_mapping_name(nested_at_rule),
                                media_feature_mappings: if raw_nested_condition.contains('@') {
                                    media_feature_mappings_resolved(
                                        nested_condition,
                                        raw_nested_condition,
                                    )
                                } else {
                                    nested_at_rule
                                        .prelude
                                        .as_ref()
                                        .map(|p| media_feature_mappings(p))
                                        .unwrap_or_default()
                                },
                            };
                            self.queue_pending_media_query(
                                nested_media_query,
                                nested_transformed_statements,
                                Some(nested_mapping),
                            );
                        }
                    }
                    _ => unreachable!("non-media statements handled above"),
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
                    .map(|p| self.prelude_media_mappings(p))
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
                self.collect_media_block_item(
                    statement,
                    &mut transformed_statements,
                    &mut current_declarations,
                    &at_rule.position,
                )?;
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
                    .map(|p| self.prelude_media_mappings(p))
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
