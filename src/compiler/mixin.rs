use super::expression::ExpressionCompiler;
use super::Compiler;
use crate::ast::*;
use crate::error::{Error, Result};

/// 混合器编译特性
pub trait MixinCompiler {
    /// 编译混合器定义
    fn compile_mixin_definition(&mut self, mixin: &MixinDefinition) -> Result<()>;

    /// 编译混合器调用
    fn compile_mixin_call(&mut self, call: &MixinCall) -> Result<()>;

    /// 将规则注册为隐式混合器
    fn register_rule_as_mixin(&mut self, rule: &Rule);

    /// 解析混合器路径
    fn resolve_mixin_path(&mut self, path: &str) -> Result<Vec<MixinDefinition>>;

    /// 在语句体中查找混合器
    fn find_mixins_in_body(
        &self,
        body: &[Statement],
        name: &str,
        source_file_hint: Option<&str>,
    ) -> Vec<MixinDefinition>;

    /// 查找匹配的混合器
    fn find_matching_mixin(
        &mut self,
        mixin_defs: &[MixinDefinition],
        call: &MixinCall,
    ) -> Result<Option<MixinDefinition>>;

    /// 将混合器参数绑定到作用域
    fn bind_mixin_arguments_to_scope(
        &mut self,
        mixin: &MixinDefinition,
        call: &MixinCall,
        scope: &mut Scope,
    ) -> Result<()>;

    /// 绑定混合器参数（便捷方法）
    fn bind_mixin_arguments(
        &mut self,
        mixin: &MixinDefinition,
        call: &MixinCall,
        scope: &mut Scope,
    ) -> Result<()>;
}

impl MixinCompiler for Compiler {
    fn compile_mixin_definition(&mut self, mixin: &MixinDefinition) -> Result<()> {
        // Store mixin definition in current scope
        let mut mixin_def = mixin.clone();
        if mixin_def.source_file.is_none() {
            mixin_def.source_file = Some(self.current_file.clone());
        }
        self.current_scope()
            .define_mixin(mixin_def.name.clone(), mixin_def);
        // Mixins don't produce CSS output directly
        Ok(())
    }

    fn compile_mixin_call(&mut self, call: &MixinCall) -> Result<()> {
        // Resolve mixin definitions (handling namespaces)
        let mixins = self.resolve_mixin_path(&call.name)?;

        if mixins.is_empty() {
            return Err(Error::undefined_mixin(
                &call.name,
                call.position.line,
                call.position.column,
            ));
        }

        // Find the first mixin whose guard evaluates to true
        let matching_mixin = self.find_matching_mixin(&mixins, call)?;

        match matching_mixin {
            Some(mixin_def) => {
                // Check recursion depth
                if self.recursion_depth >= self.max_recursion_depth {
                    return Err(Error::infinite_recursion(
                        format!("Mixin '{}'", call.name),
                        call.position.line,
                        call.position.column,
                    ));
                }

                self.recursion_depth += 1;

                // Create a new scope for the mixin expansion
                let parent = self.current_scope().clone();
                let mut mixin_scope = Scope::with_parent(parent);

                // Bind arguments to parameters
                self.bind_mixin_arguments(&mixin_def, call, &mut mixin_scope)?;

                // Push the mixin scope
                self.scope_stack.push(mixin_scope);

                // Pre-scan variables for forward reference support
                self.pre_scan_variables(&mixin_def.body);

                // Set force_important if the mixin call has !important
                let prev_force_important = self.force_important;
                if call.important {
                    self.force_important = true;
                }

                // Use mixin definition file for source-map attribution
                let previous_file = self.current_file.clone();
                if let Some(source_file) = &mixin_def.source_file {
                    self.current_file = source_file.clone();
                }

                let expansion_result = (|| -> Result<()> {
                    // Expand mixin body
                    for statement in &mixin_def.body {
                        self.compile_statement(statement)?;
                    }
                    Ok(())
                })();

                self.force_important = prev_force_important;
                self.current_file = previous_file;

                // Pop the mixin scope
                self.pop_scope();

                self.recursion_depth -= 1;

                expansion_result?;
                Ok(())
            }
            None => Err(Error::semantic_error(
                format!("No matching guard for mixin '{}'", call.name),
                call.position.line,
                call.position.column,
            )),
        }
    }

    fn register_rule_as_mixin(&mut self, rule: &Rule) {
        // Convert declarations to statements
        let mut body: Vec<Statement> = rule
            .declarations
            .iter()
            .map(|d| Statement::Declaration(d.clone()))
            .collect();

        // Append nested rules
        body.extend(rule.nested_rules.clone());

        for selector in &rule.selectors {
            // Use to_css() as the mixin name
            // Note: This matches simple selectors like .class or #id
            let name = selector.to_css();

            // Create MixinDefinition
            let mixin = MixinDefinition::new(name.clone(), rule.position.clone())
                .with_body(body.clone())
                .with_source_file(self.current_file.clone());
            // No parameters, no guard

            self.current_scope().define_mixin(name, mixin);
        }
    }

    fn resolve_mixin_path(&mut self, path: &str) -> Result<Vec<MixinDefinition>> {
        // Split path by > or space
        // This is a simplified splitting, might need robust parsing if selectors contain spaces
        let parts: Vec<&str> = path.split(['>', ' ']).filter(|s| !s.is_empty()).collect();

        if parts.is_empty() {
            return Ok(Vec::new());
        }

        // Lookup first part
        let first_name = parts[0];
        let mut candidates = if let Some(defs) = self.current_scope().lookup_mixin(first_name) {
            defs.clone()
        } else {
            return Ok(Vec::new());
        };

        // Iterate through remaining parts
        for part in &parts[1..] {
            let mut next_candidates = Vec::new();

            for candidate in &candidates {
                // Search in candidate's body
                let found = self.find_mixins_in_body(
                    &candidate.body,
                    part,
                    candidate.source_file.as_deref(),
                );
                next_candidates.extend(found);
            }

            if next_candidates.is_empty() {
                return Ok(Vec::new());
            }

            candidates = next_candidates;
        }

        Ok(candidates)
    }

    fn find_mixins_in_body(
        &self,
        body: &[Statement],
        name: &str,
        source_file_hint: Option<&str>,
    ) -> Vec<MixinDefinition> {
        let mut results = Vec::new();

        for stmt in body {
            match stmt {
                Statement::MixinDefinition(def) => {
                    if def.name == name {
                        results.push(def.clone());
                    }
                }
                Statement::Rule(rule) => {
                    // Check if rule selectors match name
                    for selector in &rule.selectors {
                        if selector.to_css() == name {
                            // Convert rule to implicit mixin definition
                            let mut mixin_body: Vec<Statement> = rule
                                .declarations
                                .iter()
                                .map(|d| Statement::Declaration(d.clone()))
                                .collect();
                            mixin_body.extend(rule.nested_rules.clone());

                            let mixin =
                                MixinDefinition::new(name.to_string(), rule.position.clone())
                                    .with_body(mixin_body);
                            let mixin = if let Some(source_file) = source_file_hint {
                                mixin.with_source_file(source_file.to_string())
                            } else {
                                mixin
                            };

                            results.push(mixin);
                            break; // Found match for this rule
                        }
                    }
                }
                _ => {}
            }
        }

        results
    }

    fn find_matching_mixin(
        &mut self,
        mixin_defs: &[MixinDefinition],
        call: &MixinCall,
    ) -> Result<Option<MixinDefinition>> {
        // Two-pass matching for default() support:
        // Pass 1: try all mixins except those with default() guard
        // Pass 2: if no match, try default() guarded mixins

        let mut default_candidates = Vec::new();

        // Pass 1: non-default mixins
        for mixin_def in mixin_defs {
            // Check pattern matching first
            if !arguments_match_patterns(mixin_def, call) {
                // Pattern doesn't match, but could be a default candidate
                if let Some(guard) = &mixin_def.guard {
                    if guard_contains_default(guard) {
                        default_candidates.push(mixin_def);
                    }
                }
                continue;
            }

            if let Some(guard) = &mixin_def.guard {
                // Check if this guard contains default()
                if guard_contains_default(guard) {
                    default_candidates.push(mixin_def);
                    continue;
                }

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

        // Pass 2: try default() guarded mixins
        for mixin_def in default_candidates {
            if let Some(guard) = &mixin_def.guard {
                let parent = self.current_scope().clone();
                let mut temp_scope = Scope::with_parent(parent);
                self.bind_mixin_arguments_to_scope(mixin_def, call, &mut temp_scope)?;
                self.scope_stack.push(temp_scope);
                let guard_result = self.evaluate_expression(guard)?;
                self.pop_scope();

                if self.is_truthy(&guard_result) {
                    return Ok(Some(mixin_def.clone()));
                }
            }
        }

        Ok(None)
    }

    fn bind_mixin_arguments_to_scope(
        &mut self,
        mixin: &MixinDefinition,
        call: &MixinCall,
        scope: &mut Scope,
    ) -> Result<()> {
        let has_variadic = mixin.parameters.last().is_some_and(|p| p.variadic);
        let non_variadic_count = if has_variadic {
            mixin.parameters.len() - 1
        } else {
            mixin.parameters.len()
        };

        // Check argument count (pattern params are always required but don't count as named params)
        let required_params = mixin
            .parameters
            .iter()
            .filter(|p| p.default_value.is_none() && !p.variadic && p.pattern_value.is_none())
            .count()
            + mixin
                .parameters
                .iter()
                .filter(|p| p.pattern_value.is_some())
                .count();
        let provided_args = call.arguments.len();

        if provided_args < required_params {
            return Err(Error::semantic_error(
                format!(
                    "Mixin '{}' expects at least {} arguments, got {}",
                    mixin.name, required_params, provided_args
                ),
                call.position.line,
                call.position.column,
            ));
        }

        // Only check max args if there's no variadic parameter
        if !has_variadic && provided_args > mixin.parameters.len() {
            return Err(Error::semantic_error(
                format!(
                    "Mixin '{}' expects at most {} arguments, got {}",
                    mixin.name,
                    mixin.parameters.len(),
                    provided_args
                ),
                call.position.line,
                call.position.column,
            ));
        }

        // Collect all evaluated arguments for @arguments
        let mut all_evaluated_args = Vec::new();

        // Bind non-variadic provided arguments
        let bind_count = provided_args.min(non_variadic_count);
        for (i, arg) in call.arguments.iter().enumerate().take(bind_count) {
            if let Some(param) = mixin.parameters.get(i) {
                let evaluated_arg = self.evaluate_expression(arg)?;
                all_evaluated_args.push(evaluated_arg.clone());
                // Pattern parameters don't bind named variables (unless they have a real name)
                if param.pattern_value.is_none() {
                    scope.define_variable(param.name.clone(), evaluated_arg);
                } else {
                    // Still bind the value if the param has a real variable name like @_
                    if !param.name.starts_with("__pattern_") {
                        scope.define_variable(param.name.clone(), evaluated_arg);
                    }
                }
            }
        }

        // Bind default values for remaining non-variadic parameters
        for param in mixin
            .parameters
            .iter()
            .skip(bind_count)
            .take(non_variadic_count - bind_count)
        {
            if let Some(default_value) = &param.default_value {
                let evaluated_default = self.evaluate_expression(default_value)?;
                all_evaluated_args.push(evaluated_default.clone());
                scope.define_variable(param.name.clone(), evaluated_default);
            }
        }

        // Handle variadic parameter: collect remaining arguments into a list
        if has_variadic {
            let variadic_param = mixin.parameters.last().ok_or_else(|| {
                Error::semantic_error(
                    "Internal error: variadic mixin has no parameters",
                    call.position.line,
                    call.position.column,
                )
            })?;
            let rest_args: Vec<Expression> = call.arguments[bind_count..]
                .iter()
                .map(|arg| self.evaluate_expression(arg))
                .collect::<Result<Vec<_>>>()?;

            all_evaluated_args.extend(rest_args.clone());

            let rest_expr = if rest_args.is_empty() {
                Expression::list(
                    Vec::new(),
                    crate::ast::ListSeparator::Space,
                    call.position.clone(),
                )
            } else if rest_args.len() == 1 {
                rest_args.into_iter().next().unwrap_or_else(|| {
                    Expression::list(
                        Vec::new(),
                        crate::ast::ListSeparator::Space,
                        call.position.clone(),
                    )
                })
            } else {
                Expression::list(
                    rest_args,
                    crate::ast::ListSeparator::Space,
                    call.position.clone(),
                )
            };
            scope.define_variable(variadic_param.name.clone(), rest_expr);
        } else {
            // Evaluate remaining args for @arguments even if no variadic
            for arg in call.arguments.iter().skip(bind_count) {
                let evaluated = self.evaluate_expression(arg)?;
                all_evaluated_args.push(evaluated);
            }
        }

        // Define @arguments special variable containing all arguments
        let arguments_expr = if all_evaluated_args.is_empty() {
            Expression::list(
                Vec::new(),
                crate::ast::ListSeparator::Space,
                call.position.clone(),
            )
        } else if all_evaluated_args.len() == 1 {
            all_evaluated_args.into_iter().next().unwrap_or_else(|| {
                Expression::list(
                    Vec::new(),
                    crate::ast::ListSeparator::Space,
                    call.position.clone(),
                )
            })
        } else {
            Expression::list(
                all_evaluated_args,
                crate::ast::ListSeparator::Space,
                call.position.clone(),
            )
        };
        scope.define_variable("arguments".to_string(), arguments_expr);

        Ok(())
    }

    fn bind_mixin_arguments(
        &mut self,
        mixin: &MixinDefinition,
        call: &MixinCall,
        scope: &mut Scope,
    ) -> Result<()> {
        self.bind_mixin_arguments_to_scope(mixin, call, scope)
    }
}

/// Check if mixin call arguments match the mixin's pattern parameters
fn arguments_match_patterns(mixin: &MixinDefinition, call: &MixinCall) -> bool {
    for (i, param) in mixin.parameters.iter().enumerate() {
        if let Some(pattern) = &param.pattern_value {
            // This parameter requires a specific value
            if let Some(arg) = call.arguments.get(i) {
                if !expressions_match(pattern, arg) {
                    return false;
                }
            } else {
                return false; // Not enough arguments
            }
        }
    }
    true
}

/// Check if two expressions are semantically equal for pattern matching
fn expressions_match(pattern: &Expression, arg: &Expression) -> bool {
    match (pattern, arg) {
        (Expression::String { value: pv, .. }, Expression::String { value: av, .. }) => pv == av,
        (
            Expression::Number {
                value: pv,
                unit: pu,
                ..
            },
            Expression::Number {
                value: av,
                unit: au,
                ..
            },
        ) => (pv - av).abs() < f64::EPSILON && pu == au,
        _ => pattern.to_css() == arg.to_css(),
    }
}

/// Check if a guard expression contains a `default()` function call
fn guard_contains_default(expr: &Expression) -> bool {
    match expr {
        Expression::FunctionCall { name, .. } => name == "default",
        Expression::BinaryOp { left, right, .. } => {
            guard_contains_default(left) || guard_contains_default(right)
        }
        Expression::UnaryOp { operand, .. } => guard_contains_default(operand),
        Expression::Parenthesized(inner, _) => guard_contains_default(inner),
        _ => false,
    }
}
