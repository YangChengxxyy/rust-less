use crate::ast::*;
use crate::error::{Error, Result};
use super::Compiler;
use super::expression::ExpressionCompiler;

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
    fn find_mixins_in_body(&self, body: &[Statement], name: &str) -> Vec<MixinDefinition>;
    
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
        self.current_scope()
            .define_mixin(mixin.name.clone(), mixin.clone());
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

                // Expand mixin body
                for statement in &mixin_def.body {
                    self.compile_statement(statement)?;
                }

                // Pop the mixin scope
                self.pop_scope();
                
                self.recursion_depth -= 1;

                Ok(())
            }
            None => Err(Error::semantic_error(
                &format!("No matching guard for mixin '{}'", call.name),
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
                .with_body(body.clone());
            // No parameters, no guard

            self.current_scope().define_mixin(name, mixin);
        }
    }

    fn resolve_mixin_path(&mut self, path: &str) -> Result<Vec<MixinDefinition>> {
        // Split path by > or space
        // This is a simplified splitting, might need robust parsing if selectors contain spaces
        let parts: Vec<&str> = path.split(|c| c == '>' || c == ' ')
            .filter(|s| !s.is_empty())
            .collect();

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
                let found = self.find_mixins_in_body(&candidate.body, part);
                next_candidates.extend(found);
            }

            if next_candidates.is_empty() {
                return Ok(Vec::new());
            }

            candidates = next_candidates;
        }

        Ok(candidates)
    }

    fn find_mixins_in_body(&self, body: &[Statement], name: &str) -> Vec<MixinDefinition> {
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

                            let mixin = MixinDefinition::new(name.to_string(), rule.position.clone())
                                .with_body(mixin_body);
                            
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

    fn bind_mixin_arguments(
        &mut self,
        mixin: &MixinDefinition,
        call: &MixinCall,
        scope: &mut Scope,
    ) -> Result<()> {
        self.bind_mixin_arguments_to_scope(mixin, call, scope)
    }
}
