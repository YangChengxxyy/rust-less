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

/// LESS 到 CSS 转换的主编译器
pub struct Compiler {
    pub(crate) scope_stack: Vec<Scope>,
    pub(crate) output: String,
    pub(crate) indent_level: usize,
    pub(crate) compressed: bool,
    pub(crate) function_registry: FunctionRegistry,
    pub(crate) extend_registry: ExtendRegistry,
    pub(crate) current_selectors: Vec<String>,
    pub(crate) pending_media_queries: Vec<(String, Vec<Statement>)>,
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
            current_line: 0,
            current_col: 0,
            current_file: "input.less".to_string(),
            suppress_output: false,
            force_important: false,
            pending_merges: std::collections::HashMap::new(),
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
            current_line: 0,
            current_col: 0,
            current_file: "input.less".to_string(),
            suppress_output: false,
            force_important: false,
            pending_merges: std::collections::HashMap::new(),
        }
    }

    /// 启用源码映射生成
    pub fn with_source_map(mut self, enabled: bool) -> Self {
        self.source_map_generator = SourceMapGenerator::new(enabled);
        self
    }

    /// 获取生成的源码映射
    pub fn generate_source_map(&self) -> Option<String> {
        self.source_map_generator.generate_json()
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
        let mut parser = Parser::from_string(input.to_string())?;
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
                if let Ok(value) = self.evaluate_expression(&var.value) {
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

        let list = self.evaluate_expression(&each_call.list)?;

        // Extract list items
        let items: Vec<Expression> = match &list {
            Expression::List { values, .. } => values.clone(),
            other => vec![other.clone()],
        };

        for (index, item) in items.iter().enumerate() {
            // Create a scope with @value, @key, @index
            self.push_scope();

            let value_str = self.evaluate_expression_to_string(item)?;
            self.current_scope().define_variable(
                "value".to_string(),
                Expression::identifier(value_str, each_call.position.clone()),
            );
            self.current_scope().define_variable(
                "key".to_string(),
                Expression::number((index + 1) as f64, each_call.position.clone()),
            );
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

        self.recursion_depth -= 1;
        Ok(())
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
        let value = self.evaluate_expression(&var.value)?;
        self.current_scope()
            .define_variable(var.name.clone(), value);
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
