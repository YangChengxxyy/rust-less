use crate::ast::*;
use crate::error::{Error, Result};
use crate::parser::Parser;
use super::Compiler;
use std::path::{Path, PathBuf};

/// 导入编译特性
pub trait ImportCompiler {
    /// 编译导入
    fn compile_import(&mut self, import: &Import) -> Result<()>;
    
    /// 解析导入路径
    fn resolve_import_path(&self, import_path: &str) -> Result<PathBuf>;
    
    /// 编译 LESS 导入
    fn compile_less_import(&mut self, import: &Import) -> Result<()>;
    
    /// 编译引用导入
    fn compile_less_import_reference(&mut self, import: &Import) -> Result<()>;
    
    /// 编译内联导入
    fn compile_inline_import(&mut self, import: &Import) -> Result<()>;
    
    /// 编译多次导入
    fn compile_less_import_multiple(&mut self, import: &Import) -> Result<()>;
}

impl ImportCompiler for Compiler {
    fn compile_import(&mut self, import: &Import) -> Result<()> {
        match import.import_type {
            ImportType::Css => {
                // CSS imports are passed through as-is
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
            ImportType::Less | ImportType::Once => {
                // LESS imports - read and compile the file
                self.compile_less_import(import)?;
            }
            ImportType::Reference => {
                // Reference imports - only make definitions available, don't output
                self.compile_less_import_reference(import)?;
            }
            ImportType::Inline => {
                // Inline imports - include file content without processing
                self.compile_inline_import(import)?;
            }
            ImportType::Multiple => {
                // Multiple imports - allow importing same file multiple times
                self.compile_less_import_multiple(import)?;
            }
        }
        Ok(())
    }

    fn resolve_import_path(&self, import_path: &str) -> Result<PathBuf> {
        // 移除引号（如果有）
        let clean_path = import_path.trim_matches('"').trim_matches('\'');

        // 添加 .less 扩展名（如果没有）
        let path_with_ext = if !clean_path.ends_with(".less") && !clean_path.ends_with(".css") {
            format!("{}.less", clean_path)
        } else {
            clean_path.to_string()
        };

        let import_path = Path::new(&path_with_ext);

        // 1. 首先检查相对于当前文件的路径
        if let Some(base) = &self.base_path {
            let relative_path = base.join(import_path);
            if relative_path.exists() {
                return relative_path.canonicalize().map_err(|e| {
                    Error::io_error(e.to_string(), Some(relative_path.display().to_string()))
                });
            }
        }

        // 2. 检查导入搜索路径
        for include_path in &self.include_paths {
            let search_path = include_path.join(import_path);
            if search_path.exists() {
                return search_path.canonicalize().map_err(|e| {
                    Error::io_error(e.to_string(), Some(search_path.display().to_string()))
                });
            }
        }

        // 3. 检查当前工作目录
        if import_path.exists() {
            return import_path.canonicalize().map_err(|e| {
                Error::io_error(e.to_string(), Some(import_path.display().to_string()))
            });
        }

        // 未找到文件
        Err(Error::import_error(
            clean_path,
            format!(
                "File not found. Searched in: {:?}, {:?}",
                self.base_path, self.include_paths
            ),
            0,
            0,
        ))
    }

    fn compile_less_import(&mut self, import: &Import) -> Result<()> {
        let resolved_path = self.resolve_import_path(&import.path).map_err(|e| {
            if let Error::ImportError { path, reason, .. } = e {
                Error::import_error(&path, reason, import.position.line, import.position.column)
            } else {
                e
            }
        })?;

        // 检查循环依赖
        if self.imported_files.contains(&resolved_path) {
            // 对于普通导入和 Once 导入，跳过已导入的文件
            return Ok(());
        }

        // 添加到已导入集合
        self.imported_files.insert(resolved_path.clone());

        // 保存当前基础路径
        let old_base_path = self.base_path.clone();

        // 更新基础路径为导入文件的目录
        self.base_path = resolved_path.parent().map(|p| p.to_path_buf());

        // 读取文件内容
        let content = std::fs::read_to_string(&resolved_path).map_err(|e| {
            Error::import_error(
                &import.path,
                e.to_string(),
                import.position.line,
                import.position.column,
            )
        })?;

        // 解析导入的文件
        let mut parser = Parser::from_string(content).map_err(|e| {
            Error::import_error(
                &import.path,
                format!("Parse error: {}", e),
                import.position.line,
                import.position.column,
            )
        })?;

        let stylesheet = parser.parse().map_err(|e| {
            Error::import_error(
                &import.path,
                format!("Parse error: {}", e),
                import.position.line,
                import.position.column,
            )
        })?;

        // 编译导入的样式表
        self.compile_stylesheet(&stylesheet)?;

        // 恢复基础路径
        self.base_path = old_base_path;

        Ok(())
    }

    fn compile_less_import_reference(&mut self, import: &Import) -> Result<()> {
        let resolved_path = self.resolve_import_path(&import.path).map_err(|e| {
            if let Error::ImportError { path, reason, .. } = e {
                Error::import_error(&path, reason, import.position.line, import.position.column)
            } else {
                e
            }
        })?;

        // 检查循环依赖
        if self.imported_files.contains(&resolved_path) {
            return Ok(());
        }

        self.imported_files.insert(resolved_path.clone());

        let old_base_path = self.base_path.clone();
        self.base_path = resolved_path.parent().map(|p| p.to_path_buf());

        let content = std::fs::read_to_string(&resolved_path).map_err(|e| {
            Error::import_error(
                &import.path,
                e.to_string(),
                import.position.line,
                import.position.column,
            )
        })?;

        let mut parser = Parser::from_string(content).map_err(|e| {
            Error::import_error(
                &import.path,
                format!("Parse error: {}", e),
                import.position.line,
                import.position.column,
            )
        })?;

        let stylesheet = parser.parse().map_err(|e| {
            Error::import_error(
                &import.path,
                format!("Parse error: {}", e),
                import.position.line,
                import.position.column,
            )
        })?;

        // 只处理变量和混合器定义，不输出 CSS
        // 这里需要访问 compile_variable_declaration 和 compile_mixin_definition
        // 它们现在分别在 ExpressionCompiler (或 mod.rs?) 和 MixinCompiler 中
        // 假设 Compiler 实现了所有 Trait
        
        // 注意：compile_variable_declaration 如果在 mod.rs，我们不能直接调用 self.compile_variable_declaration() 
        // 除非它是 inherent 方法。如果它是 trait 方法 (ExpressionCompiler?), 那么需要 import ExpressionCompiler.
        // 我们假设 compile_variable_declaration 逻辑很简单，或者我们通过 compile_statement 间接调用？
        // 但是这里我们想过滤语句。
        
        // 我们需要在 ImportCompiler 中能够调用 Compiler 的其他编译方法。
        // 这需要导入相应的 Traits。
        use super::mixin::MixinCompiler;
        // compile_variable_declaration 暂定在 mod.rs，如果是 inherent 方法，则可见。
        // 否则如果它在 StatementCompiler 之类的地方，则需要导入。
        // 我们在 mod.rs 中保留 compile_variable_declaration 为 inherent 方法。

        for statement in &stylesheet.statements {
            match statement {
                Statement::Variable(var) => {
                    // self.compile_variable_declaration(var)?;
                    // Temporarily using evaluate_expression from ExpressionCompiler manually
                    use super::expression::ExpressionCompiler;
                    let value = self.evaluate_expression(&var.value)?;
                    self.current_scope().define_variable(var.name.clone(), value);
                }
                Statement::MixinDefinition(mixin) => {
                    self.compile_mixin_definition(mixin)?;
                }
                _ => {
                    // 忽略其他语句（规则、声明等）
                }
            }
        }

        self.base_path = old_base_path;
        Ok(())
    }

    fn compile_inline_import(&mut self, import: &Import) -> Result<()> {
        let resolved_path = self.resolve_import_path(&import.path).map_err(|e| {
            if let Error::ImportError { path, reason, .. } = e {
                Error::import_error(&path, reason, import.position.line, import.position.column)
            } else {
                e
            }
        })?;

        let content = std::fs::read_to_string(&resolved_path).map_err(|e| {
            Error::import_error(
                &import.path,
                e.to_string(),
                import.position.line,
                import.position.column,
            )
        })?;

        // 直接输出文件内容
        self.output.push_str(&content);
        if !content.ends_with('\n') {
            self.add_newline();
        }

        Ok(())
    }

    fn compile_less_import_multiple(&mut self, import: &Import) -> Result<()> {
        let resolved_path = self.resolve_import_path(&import.path).map_err(|e| {
            if let Error::ImportError { path, reason, .. } = e {
                Error::import_error(&path, reason, import.position.line, import.position.column)
            } else {
                e
            }
        })?;

        // 不检查循环依赖，允许多次导入
        let old_base_path = self.base_path.clone();
        self.base_path = resolved_path.parent().map(|p| p.to_path_buf());

        let content = std::fs::read_to_string(&resolved_path).map_err(|e| {
            Error::import_error(
                &import.path,
                e.to_string(),
                import.position.line,
                import.position.column,
            )
        })?;

        let mut parser = Parser::from_string(content).map_err(|e| {
            Error::import_error(
                &import.path,
                format!("Parse error: {}", e),
                import.position.line,
                import.position.column,
            )
        })?;

        let stylesheet = parser.parse().map_err(|e| {
            Error::import_error(
                &import.path,
                format!("Parse error: {}", e),
                import.position.line,
                import.position.column,
            )
        })?;

        self.compile_stylesheet(&stylesheet)?;
        self.base_path = old_base_path;

        Ok(())
    }
}
