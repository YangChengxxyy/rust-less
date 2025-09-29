//! LESS 到 CSS 编译器
//!
//! 此模块提供将 LESS AST 转换为 CSS 输出的主要编译逻辑。

use crate::ast::*;
use crate::error::{Error, Result};
use crate::functions::FunctionRegistry;
use crate::parser::Parser;

/// LESS 到 CSS 转换的主编译器
pub struct Compiler {
    scope_stack: Vec<Scope>,
    output: String,
    indent_level: usize,
    compressed: bool,
    function_registry: FunctionRegistry,
    current_selectors: Vec<String>,
    pending_media_queries: Vec<(String, Vec<Statement>)>,
    // 新增：媒体查询上下文栈，用于追踪嵌套的媒体查询条件
    media_query_stack: Vec<String>,
}

impl Compiler {
    /// 创建新的编译器实例
    pub fn new() -> Self {
        Self {
            scope_stack: vec![Scope::new()],
            output: String::new(),
            indent_level: 0,
            compressed: false,
            function_registry: FunctionRegistry::new(),
            current_selectors: Vec::new(),
            pending_media_queries: Vec::new(),
            media_query_stack: Vec::new(),
        }
    }

    /// 创建启用压缩的新编译器
    /// 创建压缩模式的编译器
    pub fn compressed() -> Self {
        Self {
            scope_stack: vec![Scope::new()],
            output: String::new(),
            indent_level: 0,
            compressed: true,
            function_registry: FunctionRegistry::new(),
            current_selectors: Vec::new(),
            pending_media_queries: Vec::new(),
            media_query_stack: Vec::new(),
        }
    }

    /// 将 LESS 源代码编译为 CSS
    pub fn compile(&mut self, input: &str) -> Result<String> {
        let mut parser = Parser::from_string(input.to_string())?;
        let stylesheet = parser.parse()?;

        self.output.clear();
        self.indent_level = 0;

        self.compile_stylesheet(&stylesheet)?;

        // Output any pending media queries
        self.output_pending_media_queries()?;

        Ok(self.output.clone())
    }

    /// Get the current scope
    fn current_scope(&mut self) -> &mut Scope {
        self.scope_stack.last_mut().unwrap()
    }

    /// Push a new scope
    fn push_scope(&mut self) {
        let parent = self.scope_stack.last().unwrap().clone();
        self.scope_stack.push(Scope::with_parent(parent));
    }

    /// Pop the current scope
    fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    /// Add indentation to output
    fn add_indent(&mut self) {
        if !self.compressed {
            for _ in 0..self.indent_level {
                self.output.push_str("  ");
            }
        }
    }

    /// Add newline to output
    fn add_newline(&mut self) {
        if !self.compressed {
            self.output.push('\n');
        }
    }

    /// Add space to output
    fn add_space(&mut self) {
        if !self.compressed {
            self.output.push(' ');
        }
    }

    /// Compile a stylesheet
    fn compile_stylesheet(&mut self, stylesheet: &Stylesheet) -> Result<()> {
        for statement in &stylesheet.statements {
            self.compile_statement(statement)?;
        }
        Ok(())
    }

    /// Compile a statement
    fn compile_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Variable(var) => self.compile_variable_declaration(var),
            Statement::Rule(rule) => self.compile_rule(rule, &[]),
            Statement::Declaration(decl) => self.compile_declaration(decl),
            Statement::MixinDefinition(mixin) => self.compile_mixin_definition(mixin),
            Statement::MixinCall(call) => self.compile_mixin_call(call),
            Statement::Import(import) => self.compile_import(import),
            Statement::AtRule(at_rule) => {
                // 检查是否在媒体查询上下文中
                let is_nested = !self.media_query_stack.is_empty();
                self.compile_at_rule(at_rule, is_nested)
            }
            Statement::Comment(comment) => self.compile_comment(comment),
        }
    }

    /// Compile a variable declaration
    fn compile_variable_declaration(&mut self, var: &VariableDeclaration) -> Result<()> {
        let value = self.evaluate_expression(&var.value)?;
        self.current_scope()
            .define_variable(var.name.clone(), value);
        // Variables don't produce CSS output
        Ok(())
    }

    /// Compile a CSS rule
    fn compile_rule(&mut self, rule: &Rule, parent_selectors: &[String]) -> Result<()> {
        // Push a new scope for this rule
        self.push_scope();

        // First, process any nested variable declarations to establish local scope
        for nested in &rule.nested_rules {
            if let Statement::Variable(var) = nested {
                self.compile_variable_declaration(var)?;
            }
        }

        // Convert selectors to strings
        let mut current_selectors = Vec::new();
        for selector in &rule.selectors {
            let selector_str = self.compile_selector(selector, parent_selectors)?;
            current_selectors.push(selector_str);
        }

        // Check if we need to output this rule (has declarations or mixin calls)
        let has_declarations = !rule.declarations.is_empty();
        let has_mixin_calls = rule
            .nested_rules
            .iter()
            .any(|stmt| matches!(stmt, Statement::MixinCall(_)));

        if has_declarations || has_mixin_calls {
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
            }

            // Collect all declarations and mixin calls with their positions
            let mut items: Vec<(usize, RuleItem)> = Vec::new();

            // Add declarations
            for declaration in &rule.declarations {
                items.push((
                    declaration.position.line,
                    RuleItem::Declaration(declaration.clone()),
                ));
            }

            // Add mixin calls
            for nested in &rule.nested_rules {
                if let Statement::MixinCall(call) = nested {
                    items.push((call.position.line, RuleItem::MixinCall(call.clone())));
                }
            }

            // Sort by line number and process
            items.sort_by_key(|(line, _)| *line);
            for (_, item) in items {
                match item {
                    RuleItem::Declaration(decl) => self.compile_declaration(&decl)?,
                    RuleItem::MixinCall(call) => self.compile_mixin_call(&call)?,
                }
            }

            self.indent_level -= 1;

            self.add_indent();
            self.output.push('}');
            self.add_newline();
        }

        // Compile nested rules (excluding variable declarations and mixin calls which were already processed)
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
                Statement::AtRule(at_rule) => {
                    // Save current selector context
                    let old_selectors = self.current_selectors.clone();
                    self.current_selectors = current_selectors.clone();

                    self.compile_at_rule(at_rule, true)?;

                    // Restore selector context
                    self.current_selectors = old_selectors;
                }
                _ => {
                    self.compile_statement(nested)?;
                }
            }
        }

        // Pop the scope when done with this rule
        self.pop_scope();

        Ok(())
    }

    /// Compile a selector
    fn compile_selector(
        &mut self,
        selector: &Selector,
        parent_selectors: &[String],
    ) -> Result<String> {
        let mut selector_str = String::new();

        // Compile each part of the selector, handling interpolation
        for (part_idx, part) in selector.parts.iter().enumerate() {
            if part_idx > 0 {
                if let Some(combinator) = &part.combinator {
                    selector_str.push_str(&format!(" {} ", combinator.to_css()));
                } else {
                    selector_str.push(' ');
                }
            }

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
        }

        if selector.has_parent_reference() {
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

    /// Compile a declaration
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

    /// Compile a mixin definition
    fn compile_mixin_definition(&mut self, mixin: &MixinDefinition) -> Result<()> {
        // Store mixin definition in current scope
        self.current_scope()
            .define_mixin(mixin.name.clone(), mixin.clone());
        // Mixins don't produce CSS output directly
        Ok(())
    }

    /// Compile a mixin call
    fn compile_mixin_call(&mut self, call: &MixinCall) -> Result<()> {
        // Look up the mixin definitions
        let mixins = self.current_scope().lookup_mixin(&call.name).cloned();

        match mixins {
            Some(mixin_defs) => {
                // Find the first mixin whose guard evaluates to true
                let matching_mixin = self.find_matching_mixin(&mixin_defs, call)?;

                match matching_mixin {
                    Some(mixin_def) => {
                        // Create a new scope for the mixin expansion
                        let parent = self.current_scope().clone();
                        let mut mixin_scope = Scope::with_parent(parent);

                        // Bind arguments to parameters
                        self.bind_mixin_arguments(&mixin_def, call, &mut mixin_scope)?;

                        // Push the mixin scope
                        self.scope_stack.push(mixin_scope);

                        // Expand mixin body
                        for statement in &mixin_def.body {
                            self.compile_statement(statement)?;
                        }

                        // Pop the mixin scope
                        self.pop_scope();

                        Ok(())
                    }
                    None => Err(Error::semantic_error(
                        &format!("No matching guard for mixin '{}'", call.name),
                        call.position.line,
                        call.position.column,
                    )),
                }
            }
            None => Err(Error::semantic_error(
                &format!("Undefined mixin '{}'", call.name),
                call.position.line,
                call.position.column,
            )),
        }
    }

    /// Find the first mixin definition whose guard matches the call arguments
    fn find_matching_mixin(
        &mut self,
        mixin_defs: &[MixinDefinition],
        call: &MixinCall,
    ) -> Result<Option<MixinDefinition>> {
        for mixin_def in mixin_defs {
            if let Some(guard) = &mixin_def.guard {
                // Create a temporary scope with the arguments bound
                let parent = self.current_scope().clone();
                let mut temp_scope = Scope::with_parent(parent);

                // Bind arguments to parameters for guard evaluation
                self.bind_mixin_arguments_to_scope(mixin_def, call, &mut temp_scope)?;

                // Push the temporary scope
                self.scope_stack.push(temp_scope);

                // Evaluate the guard expression
                let guard_result = self.evaluate_expression(guard)?;

                // Pop the temporary scope
                self.pop_scope();

                // Check if the guard evaluates to true
                if self.is_truthy(&guard_result) {
                    return Ok(Some(mixin_def.clone()));
                }
            } else {
                // No guard means this mixin matches unconditionally
                return Ok(Some(mixin_def.clone()));
            }
        }
        Ok(None)
    }

    /// Check if an expression evaluates to a truthy value
    fn is_truthy(&self, expr: &expressions::Expression) -> bool {
        match expr {
            expressions::Expression::Boolean(b, _) => *b,
            expressions::Expression::Number { value, .. } => *value != 0.0,
            expressions::Expression::String { value, .. } => !value.is_empty(),
            _ => false,
        }
    }

    /// Bind mixin arguments to parameters in a given scope
    fn bind_mixin_arguments_to_scope(
        &mut self,
        mixin: &MixinDefinition,
        call: &MixinCall,
        scope: &mut Scope,
    ) -> Result<()> {
        // Check argument count
        let required_params = mixin
            .parameters
            .iter()
            .filter(|p| p.default_value.is_none())
            .count();
        let provided_args = call.arguments.len();

        if provided_args < required_params {
            return Err(Error::semantic_error(
                &format!(
                    "Mixin '{}' expects at least {} arguments, got {}",
                    mixin.name, required_params, provided_args
                ),
                call.position.line,
                call.position.column,
            ));
        }

        if provided_args > mixin.parameters.len() {
            return Err(Error::semantic_error(
                &format!(
                    "Mixin '{}' expects at most {} arguments, got {}",
                    mixin.name,
                    mixin.parameters.len(),
                    provided_args
                ),
                call.position.line,
                call.position.column,
            ));
        }

        // Bind provided arguments
        for (i, arg) in call.arguments.iter().enumerate() {
            if let Some(param) = mixin.parameters.get(i) {
                let evaluated_arg = self.evaluate_expression(arg)?;
                scope.define_variable(param.name.clone(), evaluated_arg);
            }
        }

        // Bind default values for remaining parameters
        for param in mixin.parameters.iter().skip(call.arguments.len()) {
            if let Some(default_value) = &param.default_value {
                let evaluated_default = self.evaluate_expression(default_value)?;
                scope.define_variable(param.name.clone(), evaluated_default);
            }
        }

        Ok(())
    }

    /// Bind mixin arguments to parameters
    fn bind_mixin_arguments(
        &mut self,
        mixin: &MixinDefinition,
        call: &MixinCall,
        scope: &mut Scope,
    ) -> Result<()> {
        self.bind_mixin_arguments_to_scope(mixin, call, scope)
    }

    /// Compile an import
    fn compile_import(&mut self, import: &Import) -> Result<()> {
        match import.import_type {
            ImportType::Css => {
                self.add_indent();
                self.output.push_str("@import ");
                self.output.push_str(&format!("\"{}\"", import.path));
                if let Some(media) = &import.media {
                    self.add_space();
                    self.output.push_str(media);
                }
                self.output.push(';');
                self.add_newline();
            }
            _ => {
                // TODO: Handle LESS imports
                return Err(Error::import_error(
                    &import.path,
                    "LESS imports not yet implemented",
                    import.position.line,
                    import.position.column,
                ));
            }
        }
        Ok(())
    }

    /// Compile an at-rule
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

    /// 编译媒体查询，支持嵌套和条件合并
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

    /// 编译嵌套媒体查询并合并条件
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
                        self.compile_variable_declaration(var)?;
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

    /// 处理媒体查询中的嵌套规则
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

    /// Compile a nested media query (simple implementation)
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
                        self.compile_variable_declaration(var)?;
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

    /// Create a rule for current selectors with given declarations
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

    /// Output all pending media queries
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

    /// Compile a comment
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

    /// Evaluate an expression to a value
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

    /// Convert an expression to a string for interpolation
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

    /// Resolve a variable by name
    fn resolve_variable(&mut self, name: &str) -> Result<Expression> {
        if let Some(value) = self.current_scope().lookup_variable(name) {
            Ok(value.clone())
        } else {
            Err(Error::undefined_variable(name, 0, 0))
        }
    }

    /// Evaluate a binary operation
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

    /// Evaluate a unary operation
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

    /// Evaluate a function call
    fn evaluate_function_call(
        &self,
        name: &str,
        arguments: &[Expression],
        position: &Position,
    ) -> Result<Expression> {
        // Use the function registry to call the function
        self.function_registry.call(name, arguments, position)
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_variable() {
        let mut compiler = Compiler::new();
        let result = compiler
            .compile("@color: red; .test { color: @color; }")
            .unwrap();

        assert!(result.contains(".test"));
        assert!(result.contains("color: red"));
    }

    #[test]
    fn test_simple_rule() {
        let mut compiler = Compiler::new();
        let result = compiler.compile(".test { color: red; }").unwrap();

        assert!(result.contains(".test {"));
        assert!(result.contains("color: red;"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_arithmetic() {
        let mut compiler = Compiler::new();
        let result = compiler.compile(".test { width: 10px + 5px; }").unwrap();

        assert!(result.contains("width: 15px"));
    }

    #[test]
    fn test_function_call() {
        let mut compiler = Compiler::new();
        let result = compiler.compile(".test { width: round(10.6px); }").unwrap();

        assert!(result.contains("width: 11px"));
    }

    #[test]
    fn test_undefined_variable() {
        let mut compiler = Compiler::new();
        let result = compiler.compile(".test { color: @undefined; }");

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::UndefinedVariable { .. }
        ));
    }

    #[test]
    fn test_percentage_function() {
        let mut compiler = Compiler::new();
        let result = compiler
            .compile(".test { width: percentage(0.5); }")
            .unwrap();

        assert!(result.contains("width: 50%"));
    }

    #[test]
    fn test_compressed_output() {
        let mut compiler = Compiler::compressed();
        let result = compiler.compile(".test { color: red; }").unwrap();

        // Compressed output should not contain extra whitespace
        assert!(!result.contains("  "));
        assert!(!result.contains("\n"));
    }

    #[test]
    fn test_mixin_default_parameters() {
        let mut compiler = Compiler::new();
        let input = r#".test-mixin(@param: red) {
    color: @param;
}

.usage {
    .test-mixin();
}"#;
        let result = compiler.compile(input).unwrap();

        // Should expand mixin with default parameter value
        assert!(result.contains("color: red"));
        assert!(result.contains(".usage"));
    }

    #[test]
    fn test_mixin_multiple_default_parameters() {
        let mut compiler = Compiler::new();
        let input = r#".box-shadow(@x: 0, @y: 0, @blur: 5px, @color: #000) {
    box-shadow: @x @y @blur @color;
}

.card {
    .box-shadow();
}"#;
        let result = compiler.compile(input);

        match result {
            Ok(css) => {
                println!("Compiled CSS: {}", css);
                assert!(css.contains("box-shadow: 0 0 5px #000"));
            }
            Err(e) => {
                panic!("Failed to compile: {}", e);
            }
        }
    }

    #[test]
    fn test_mixin_single_complex_value() {
        let mut compiler = Compiler::new();
        let input = r#".test-mixin(@value: "hello world") {
    content: @value;
}

.usage {
    .test-mixin();
}"#;
        let result = compiler.compile(input);

        match result {
            Ok(css) => {
                println!("Single value CSS: {}", css);
                assert!(css.contains("content: \"hello world\""));
            }
            Err(e) => {
                panic!("Failed to compile single value: {}", e);
            }
        }
    }
}
