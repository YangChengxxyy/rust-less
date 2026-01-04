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

mod expression;
mod rule;
mod mixin;
mod import;
mod at_rule;

pub(crate) use expression::ExpressionCompiler;
pub(crate) use rule::RuleCompiler;
pub(crate) use mixin::MixinCompiler;
pub(crate) use import::ImportCompiler;
pub(crate) use at_rule::AtRuleCompiler;

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
        }
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
        // Collect extends from this stylesheet and merge into registry
        let mut collector = ExtendCollector::new();
        collector.collect(stylesheet);
        self.extend_registry.merge(collector.registry);

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
            },
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
