//! LESS 到 CSS 编译器
//!
//! 此模块提供将 LESS AST 转换为 CSS 输出的主要编译逻辑。

use crate::ast::*;
use crate::error::{Error, Result};
use crate::extend::{ExtendCollector, ExtendRegistry};
use crate::functions::FunctionRegistry;
use crate::parser::Parser;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

mod at_rule;
mod expression;
mod import;
mod mixin;
mod rule;
pub mod sourcemap;

pub(crate) use at_rule::AtRuleCompiler;
pub(crate) use expression::ExpressionCompiler;
pub(crate) use import::ImportCompiler;
pub(crate) use mixin::MixinCompiler;
pub(crate) use rule::RuleCompiler;
use sourcemap::SourceMapGenerator;

#[derive(Debug, Clone)]
pub(crate) struct PendingAtRuleMapping {
    pub(crate) source_file: String,
    pub(crate) position: Position,
    pub(crate) name: String,
    /// For `@media`, less.js may emit extra header segments.
    /// Tuple: (generated_column_delta_from_line_start, source_column_delta_from_@media_start).
    pub(crate) media_feature_mappings: Vec<(usize, usize)>,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingMediaQuery {
    pub(crate) header: String,
    pub(crate) statements: Vec<Statement>,
    pub(crate) mapping: Option<PendingAtRuleMapping>,
}

/// LESS 到 CSS 转换的主编译器
pub struct Compiler {
    pub(crate) scope_stack: Vec<Scope>,
    pub(crate) output: String,
    pub(crate) indent_level: usize,
    pub(crate) compressed: bool,
    pub(crate) function_registry: FunctionRegistry,
    pub(crate) extend_registry: ExtendRegistry,
    pub(crate) current_selectors: Vec<String>,
    pub(crate) pending_media_queries: Vec<PendingMediaQuery>,
    /// 媒体查询上下文栈，用于追踪嵌套的媒体查询条件
    pub(crate) media_query_stack: Vec<String>,
    /// 当前编译文件的基础路径
    pub(crate) base_path: Option<PathBuf>,
    /// 导入搜索路径
    pub(crate) include_paths: Vec<PathBuf>,
    /// 已导入的文件集合（用于循环依赖检测）
    pub(crate) imported_files: HashSet<PathBuf>,
    /// 当前递归深度
    pub(crate) recursion_depth: usize,
    /// 最大递归深度限制
    pub(crate) max_recursion_depth: usize,
    /// 源码映射生成器
    pub(crate) source_map_generator: SourceMapGenerator,
    /// source map 是否启用 less.js 兼容输出策略
    pub(crate) source_map_lessjs_compat: bool,
    /// 当前输出行号（0-based）
    pub(crate) current_line: u32,
    /// 当前输出列号（0-based）
    pub(crate) current_col: u32,
    /// 当前正在编译的源文件路径
    pub(crate) current_file: String,
    /// 是否抑制输出（用于 reference 导入）
    pub(crate) suppress_output: bool,
    /// 是否强制 !important（用于 .mixin() !important 传播）
    pub(crate) force_important: bool,
    /// 属性值合并缓冲区: property -> (values, merge_type, important, first_position)
    pub(crate) pending_merges:
        std::collections::HashMap<String, (Vec<String>, MergeType, bool, Position)>,
    /// Original (unresolved) at-rule prelude of the at-rule currently being
    /// compiled; used to keep source-map columns anchored to source text when
    /// the emitted prelude has variables resolved.
    pub(crate) original_at_rule_prelude: Option<String>,
    /// 插件解析钩子表（按注册顺序，见 `plugin::ParseHook`）
    pub(crate) parse_hooks: Vec<Box<dyn crate::plugin::ParseHook>>,
    /// 插件编译期 visitor（按注册顺序，见 `plugin::CompileVisitor`）
    pub(crate) visitors: Vec<Box<dyn crate::plugin::CompileVisitor>>,
    /// 插件导入解析器链（按注册顺序，见 `plugin::ImportResolver`）
    pub(crate) import_resolvers: Vec<Box<dyn crate::plugin::ImportResolver>>,
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
            extend_registry: ExtendRegistry::new(),
            current_selectors: Vec::new(),
            pending_media_queries: Vec::new(),
            media_query_stack: Vec::new(),
            base_path: None,
            include_paths: Vec::new(),
            imported_files: HashSet::new(),
            recursion_depth: 0,
            max_recursion_depth: 100,
            source_map_generator: SourceMapGenerator::default(),
            source_map_lessjs_compat: false,
            current_line: 0,
            current_col: 0,
            current_file: "input.less".to_string(),
            suppress_output: false,
            force_important: false,
            pending_merges: std::collections::HashMap::new(),
            original_at_rule_prelude: None,
            parse_hooks: Vec::new(),
            visitors: Vec::new(),
            import_resolvers: Vec::new(),
        }
    }

    /// 创建压缩模式的编译器
    pub fn compressed() -> Self {
        Self {
            scope_stack: vec![Scope::new()],
            output: String::new(),
            indent_level: 0,
            compressed: true,
            function_registry: FunctionRegistry::new(),
            extend_registry: ExtendRegistry::new(),
            current_selectors: Vec::new(),
            pending_media_queries: Vec::new(),
            media_query_stack: Vec::new(),
            base_path: None,
            include_paths: Vec::new(),
            imported_files: HashSet::new(),
            recursion_depth: 0,
            max_recursion_depth: 100,
            source_map_generator: SourceMapGenerator::default(),
            source_map_lessjs_compat: false,
            current_line: 0,
            current_col: 0,
            current_file: "input.less".to_string(),
            suppress_output: false,
            force_important: false,
            pending_merges: std::collections::HashMap::new(),
            original_at_rule_prelude: None,
            parse_hooks: Vec::new(),
            visitors: Vec::new(),
            import_resolvers: Vec::new(),
        }
    }

    /// 启用源码映射生成
    pub fn with_source_map(mut self, enabled: bool) -> Self {
        self.source_map_generator = SourceMapGenerator::new(enabled);
        self
    }

    /// 注册自定义函数，编译期间可通过 `@name(...)` 调用。
    ///
    /// 后注册的同名函数覆盖内置实现；函数必须是纯求值。
    /// 这是插件钩子设计草案（docs/PLUGIN_HOOKS_DESIGN.md）中函数插件的最小落地形态。
    pub fn register_function<F>(&mut self, name: &str, func: Box<F>)
    where
        F: Fn(&[Expression], &Position) -> Result<Expression> + 'static,
    {
        self.function_registry.register(name, func);
    }

    /// 注册自定义函数插件（[`crate::plugin::LessFunction`]）。
    ///
    /// 与内置函数同一调用路径；后注册覆盖同名内置函数。
    /// 插件声明的 API 版本与 [`crate::plugin::PLUGIN_API_VERSION`] 不一致时
    /// 返回 [`Error::PluginError`]。
    pub fn register_function_plugin(
        &mut self,
        plugin: Box<dyn crate::plugin::LessFunction>,
    ) -> Result<()> {
        crate::plugin::check_api_version(plugin.name(), plugin.api_version())?;
        self.function_registry.register_plugin(plugin);
        Ok(())
    }

    /// 注册自定义 at-rule 解析钩子（[`crate::plugin::ParseHook`]）。
    ///
    /// 精确匹配钩子声明名称的 `@at-rule` 在解析期交由钩子转换；
    /// 未命中走现有通用 at-rule 路径。
    pub fn register_parse_hook(&mut self, hook: Box<dyn crate::plugin::ParseHook>) -> Result<()> {
        crate::plugin::check_api_version(hook.name(), hook.api_version())?;
        self.parse_hooks.push(hook);
        Ok(())
    }

    /// 注册编译期 visitor（[`crate::plugin::CompileVisitor`]）。
    ///
    /// * `pre_visit_rule` 在每条规则发射前调用，可改写选择器/声明；
    /// * `post_process` 在编译完成后按注册顺序依次调用。
    ///
    /// 声明 [`crate::plugin::CompileVisitor::invalidates_source_map`]
    /// 的 visitor 与启用的 source map 冲突时返回 [`Error::PluginError`]。
    pub fn register_visitor(
        &mut self,
        visitor: Box<dyn crate::plugin::CompileVisitor>,
    ) -> Result<()> {
        crate::plugin::check_api_version(visitor.name(), visitor.api_version())?;
        if visitor.invalidates_source_map() && self.source_map_generator.is_enabled() {
            return Err(Error::plugin_error(
                visitor.name(),
                "visitor invalidates source map but source map output is enabled",
                0,
                0,
            ));
        }
        self.visitors.push(visitor);
        Ok(())
    }

    /// 注册导入解析钩子（[`crate::plugin::ImportResolver`]）。
    ///
    /// 解析链顺序：插件解析器 → include_paths → 默认文件系统。
    pub fn register_import_resolver(
        &mut self,
        resolver: Box<dyn crate::plugin::ImportResolver>,
    ) -> Result<()> {
        crate::plugin::check_api_version(resolver.name(), resolver.api_version())?;
        self.import_resolvers.push(resolver);
        Ok(())
    }

    /// 注册插件扩展包（[`crate::plugin::PluginBundle`]）。
    ///
    /// 先校验 API 版本，再由插件包自行注册各扩展点；
    /// 错误统一包装为 [`Error::PluginError`] 并带上插件包名称。
    pub fn register_plugin_bundle(
        &mut self,
        bundle: Box<dyn crate::plugin::PluginBundle>,
    ) -> Result<()> {
        crate::plugin::register_bundle(self, bundle)
    }

    /// 获取生成的源码映射
    pub fn generate_source_map(&self) -> Option<String> {
        self.source_map_generator.generate_json()
    }

    /// 设置 source map 的 file 字段（通常为生成的 CSS 文件路径）。
    pub fn set_source_map_file(&mut self, file: Option<String>) -> &mut Self {
        self.source_map_generator.set_file(file.as_deref());
        self
    }

    /// 设置 source map 的 sourceRoot 字段。
    pub fn set_source_map_source_root(&mut self, source_root: Option<String>) -> &mut Self {
        self.source_map_generator
            .set_source_root(source_root.as_deref());
        self
    }

    /// 设置 source map 输出为 less.js 兼容模式。
    pub fn set_source_map_lessjs_compat(&mut self, enabled: bool) -> &mut Self {
        self.source_map_generator.set_lessjs_compat_mode(enabled);
        self.source_map_lessjs_compat = enabled;
        self
    }

    /// 写入字符串到输出，并更新行号列号
    pub(crate) fn write_str(&mut self, s: &str) {
        if self.suppress_output {
            return;
        }
        self.output.push_str(s);
        for c in s.chars() {
            if c == '\n' {
                self.current_line += 1;
                self.current_col = 0;
            } else {
                self.current_col += 1;
            }
        }
    }

    /// 写入字符到输出
    pub(crate) fn write_char(&mut self, c: char) {
        if self.suppress_output {
            return;
        }
        self.output.push(c);
        if c == '\n' {
            self.current_line += 1;
            self.current_col = 0;
        } else {
            self.current_col += 1;
        }
    }

    /// 添加源码映射
    pub(crate) fn add_mapping(&mut self, position: &Position, name: Option<&str>) {
        if self.suppress_output {
            return;
        }
        let source_file = self.current_file.clone();
        self.source_map_generator.add_mapping(
            &source_file,
            position,
            self.current_line,
            self.current_col,
            name,
        );
    }

    pub(crate) fn add_mapping_at_generated_col(
        &mut self,
        position: &Position,
        name: Option<&str>,
        gen_col: u32,
    ) {
        if self.suppress_output {
            return;
        }
        let source_file = self.current_file.clone();
        self.source_map_generator.add_mapping(
            &source_file,
            position,
            self.current_line,
            gen_col,
            name,
        );
    }

    pub(crate) fn queue_pending_media_query(
        &mut self,
        header: String,
        statements: Vec<Statement>,
        mapping: Option<PendingAtRuleMapping>,
    ) {
        self.pending_media_queries.push(PendingMediaQuery {
            header,
            statements,
            mapping,
        });
    }

    pub(crate) fn emit_pending_media_query(&mut self, query: PendingMediaQuery) -> Result<()> {
        let PendingMediaQuery {
            header,
            statements,
            mapping,
        } = query;

        let previous_file = self.current_file.clone();
        if let Some(mapping_info) = mapping.as_ref() {
            self.current_file = mapping_info.source_file.clone();
            self.add_mapping(&mapping_info.position, Some(&mapping_info.name));

            if !mapping_info.media_feature_mappings.is_empty() {
                let line_start_col = self.current_col;
                for (generated_col_delta, source_col_delta) in &mapping_info.media_feature_mappings
                {
                    let gen_col = line_start_col.saturating_add(*generated_col_delta as u32);
                    let mut source_pos = mapping_info.position.clone();
                    source_pos.column = source_pos.column.saturating_add(*source_col_delta);
                    self.add_mapping_at_generated_col(
                        &source_pos,
                        Some(&mapping_info.name),
                        gen_col,
                    );
                }
            }
        }

        let result = (|| -> Result<()> {
            self.write_str(&header);
            self.add_space();
            self.write_char('{');
            self.add_newline();

            self.indent_level += 1;
            for statement in &statements {
                self.compile_statement(statement)?;
            }
            self.indent_level -= 1;

            self.add_indent();
            self.write_char('}');
            self.add_newline();
            Ok(())
        })();

        self.current_file = previous_file;
        result
    }

    /// 设置最大递归深度
    pub fn with_recursion_limit(mut self, limit: usize) -> Self {
        self.max_recursion_depth = limit;
        self
    }

    /// 添加导入搜索路径
    pub fn add_include_path<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.include_paths.push(path.as_ref().to_path_buf());
        self
    }

    /// 设置多个导入搜索路径
    pub fn with_include_paths<P: AsRef<Path>>(mut self, paths: Vec<P>) -> Self {
        self.include_paths = paths.iter().map(|p| p.as_ref().to_path_buf()).collect();
        self
    }

    /// 编译 LESS 文件
    pub fn compile_file<P: AsRef<Path>>(&mut self, path: P) -> Result<String> {
        let path = path.as_ref();
        self.imported_files.clear();

        // 获取绝对路径
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| Error::io_error(e.to_string(), Some(path.display().to_string())))?
                .join(path)
        };

        // 规范化路径
        let canonical_path = absolute_path.canonicalize().map_err(|e| {
            Error::io_error(
                format!("Cannot resolve path: {}", e),
                Some(path.display().to_string()),
            )
        })?;

        // 设置基础路径
        self.base_path = canonical_path.parent().map(|p| p.to_path_buf());

        // Set current file for source map
        self.current_file = canonical_path.display().to_string();

        // 添加到已导入文件集合
        self.imported_files.insert(canonical_path.clone());

        // 读取文件内容
        let content = std::fs::read_to_string(&canonical_path).map_err(|e| {
            Error::io_error(e.to_string(), Some(canonical_path.display().to_string()))
        })?;

        // 编译内容
        self.compile(&content)
    }

    /// 将 LESS 源代码编译为 CSS
    pub fn compile(&mut self, input: &str) -> Result<String> {
        let source_base_path = {
            let current_file = Path::new(&self.current_file);
            if current_file.is_absolute() {
                current_file.parent().map(|p| p.to_path_buf())
            } else {
                None
            }
        };
        self.source_map_generator
            .set_source_base_path(source_base_path);

        let mut parser = Parser::from_string(input.to_string())?;
        if !self.parse_hooks.is_empty() {
            parser.set_parse_hooks(
                self.parse_hooks
                    .iter()
                    .map(|hook| hook.as_ref() as &dyn crate::plugin::ParseHook)
                    .collect(),
            );
        }
        let stylesheet = parser.parse()?;

        self.output.clear();
        self.indent_level = 0;
        self.pending_media_queries.clear();
        self.pending_merges.clear();
        self.current_line = 0;
        self.current_col = 0;
        self.source_map_generator.reset();

        self.compile_stylesheet(&stylesheet)?;

        // Output any pending media queries
        self.output_pending_media_queries()?;

        // 后处理器：按注册顺序依次调用（对齐 less.js post-processor）
        let mut css = std::mem::take(&mut self.output);
        for visitor in &self.visitors {
            visitor
                .post_process(&mut css)
                .map_err(|e| crate::plugin::wrap_plugin_error(visitor.name(), e))?;
        }
        self.output = css;

        Ok(self.output.clone())
    }

    /// Get the current scope
    fn current_scope(&mut self) -> &mut Scope {
        self.scope_stack
            .last_mut()
            .expect("scope stack should never be empty")
    }

    /// Push a new scope
    fn push_scope(&mut self) {
        let parent = self
            .scope_stack
            .last()
            .expect("scope stack should never be empty")
            .clone();
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
        if !self.compressed && !self.suppress_output {
            for _ in 0..self.indent_level {
                self.output.push_str("  ");
                self.current_col += 2;
            }
        }
    }

    /// Add newline to output
    fn add_newline(&mut self) {
        if !self.compressed && !self.suppress_output {
            self.output.push('\n');
            self.current_line += 1;
            self.current_col = 0;
        }
    }

    /// Add space to output
    fn add_space(&mut self) {
        if !self.compressed && !self.suppress_output {
            self.output.push(' ');
            self.current_col += 1;
        }
    }

    /// Pre-scan variable declarations in a list of statements (lazy evaluation / hoisting).
    /// Variables whose expressions fail to evaluate are silently skipped
    /// and will be picked up during normal compilation.
    pub(crate) fn pre_scan_variables(&mut self, statements: &[Statement]) {
        for stmt in statements {
            if let Statement::Variable(var) = stmt {
                // Map literals are stored raw for lazy use-site evaluation
                // (matching compile_variable_declaration / less.js semantics).
                if matches!(&var.value, Expression::MapLiteral { .. }) {
                    self.current_scope()
                        .define_variable(var.name.clone(), var.value.clone());
                } else if let Ok(value) = self.evaluate_expression(&var.value) {
                    self.current_scope()
                        .define_variable(var.name.clone(), value);
                }
            }
        }
    }

    /// Compile a stylesheet
    fn compile_stylesheet(&mut self, stylesheet: &Stylesheet) -> Result<()> {
        // Collect extends from this stylesheet and merge into registry
        let mut collector = ExtendCollector::new();
        collector.collect(stylesheet);
        self.extend_registry.merge(collector.registry);

        // Pre-scan variables for forward reference support (lazy evaluation)
        self.pre_scan_variables(&stylesheet.statements);

        for statement in &stylesheet.statements {
            self.compile_statement(statement)?;
        }
        Ok(())
    }

    /// Compile a statement
    fn compile_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Variable(var) => self.compile_variable_declaration(var),
            Statement::Rule(rule) => {
                let parent_selectors = self.current_selectors.clone();
                self.compile_rule(rule, &parent_selectors)
            }
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
            Statement::Extend(_) => Ok(()), // Phase 1: Ignore extend statements in compiler
            Statement::EachCall(each_call) => self.compile_each_call(each_call),
            Statement::DetachedRulesetCall(call) => self.compile_detached_ruleset_call(call),
        }
    }

    /// Compile an each() call by expanding the template for each list item
    fn compile_each_call(&mut self, each_call: &EachCall) -> Result<()> {
        // Check recursion depth
        if self.recursion_depth >= self.max_recursion_depth {
            return Err(Error::infinite_recursion(
                "each() call",
                each_call.position.line,
                each_call.position.column,
            ));
        }
        self.recursion_depth += 1;

        let result = (|| -> Result<()> {
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

            for (index, (key_expr, value_expr)) in iterations.into_iter().enumerate() {
                // Create a scope with @value, @key, @index
                self.push_scope();

                self.current_scope()
                    .define_variable("value".to_string(), value_expr);
                self.current_scope()
                    .define_variable("key".to_string(), key_expr);
                self.current_scope().define_variable(
                    "index".to_string(),
                    Expression::number((index + 1) as f64, each_call.position.clone()),
                );

                // Expand the template body
                for statement in &each_call.body {
                    self.compile_statement(statement)?;
                }

                self.pop_scope();
            }

            Ok(())
        })();

        self.recursion_depth -= 1;
        result
    }

    /// Compile a detached ruleset call: resolve the variable, confirm it holds
    /// a `DetachedRuleset`, then compile its body in a new scope.
    fn compile_detached_ruleset_call(&mut self, call: &DetachedRulesetCall) -> Result<()> {
        // Check recursion depth
        if self.recursion_depth >= self.max_recursion_depth {
            return Err(Error::infinite_recursion(
                format!("@{}()", call.name),
                call.position.line,
                call.position.column,
            ));
        }
        self.recursion_depth += 1;

        let result = (|| -> Result<()> {
            // Resolve the variable
            let value = if let Some(v) = self.current_scope().lookup_variable(&call.name) {
                v.clone()
            } else {
                return Err(Error::undefined_variable(
                    &call.name,
                    call.position.line,
                    call.position.column,
                ));
            };

            // Attribute expanded declarations to the definition file
            let definition_file = self
                .current_scope()
                .lookup_variable_file(&call.name)
                .cloned();
            let previous_file = self.current_file.clone();
            if let Some(file) = definition_file {
                self.current_file = file;
            }

            let expand_result = (|| -> Result<()> {
                // Confirm it's a detached ruleset
                if let Expression::DetachedRuleset { body, .. } = value {
                    // Push a new scope and compile the body
                    self.push_scope();
                    for statement in &body {
                        self.compile_statement(statement)?;
                    }
                    self.pop_scope();
                    Ok(())
                } else if let Expression::MapLiteral { entries, .. } = value {
                    // A map literal invoked as `@map()` expands its entries as
                    // declarations, matching less.js where maps are rulesets.
                    for (key, entry_value, key_position) in entries {
                        let declaration = Declaration::new(
                            key.clone(),
                            entry_value.clone(),
                            key_position.clone(),
                        );
                        self.compile_declaration(&declaration)?;
                    }
                    Ok(())
                } else {
                    Err(Error::semantic_error(
                        format!("@{}() is not a detached ruleset", call.name),
                        call.position.line,
                        call.position.column,
                    ))
                }
            })();

            self.current_file = previous_file;
            expand_result
        })();

        self.recursion_depth -= 1;
        result
    }

    /// Flush pending property merges, outputting the combined declarations
    pub(crate) fn flush_pending_merges(&mut self) {
        if self.pending_merges.is_empty() {
            return;
        }
        // Collect entries to avoid borrowing issues
        let entries: Vec<(String, Vec<String>, MergeType, bool, Position)> = self
            .pending_merges
            .drain()
            .map(|(k, (vals, mt, imp, pos))| (k, vals, mt, imp, pos))
            .collect();

        for (property, values, merge_type, important, position) in entries {
            let separator = match merge_type {
                MergeType::Comma => ", ",
                MergeType::Space => " ",
            };
            self.add_mapping(&position, Some(&property));
            self.add_indent();
            self.write_str(&property);
            self.write_char(':');
            self.add_space();
            self.write_str(&values.join(separator));
            if important || self.force_important {
                self.add_space();
                self.write_str("!important");
            }
            self.write_char(';');
            self.add_newline();
        }
    }

    /// Compile a variable declaration
    pub(crate) fn compile_variable_declaration(&mut self, var: &VariableDeclaration) -> Result<()> {
        // Map literals are stored unevaluated so entry values and interpolated
        // keys resolve lazily at each use site (less.js semantics); evaluation
        // happens in Expression::Variable handling.
        let value = if matches!(&var.value, Expression::MapLiteral { .. }) {
            var.value.clone()
        } else {
            self.evaluate_expression(&var.value)?
        };
        let current_file = self.current_file.clone();
        let scope = self.current_scope();
        scope.define_variable(var.name.clone(), value);
        // Remember the definition file for source-map attribution when the
        // variable (e.g. a detached ruleset or map) is expanded elsewhere.
        scope.define_variable_file(var.name.clone(), current_file);
        // Variables don't produce CSS output
        Ok(())
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
