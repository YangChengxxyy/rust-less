use super::Compiler;
use crate::ast::*;
use crate::error::{Error, Result};
use crate::parser::Parser;
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
}

impl ImportCompiler for Compiler {
    fn compile_import(&mut self, import: &Import) -> Result<()> {
        // Optional imports are silently skipped when the target file is missing
        // (CSS pass-through imports never touch the filesystem, so they are exempt).
        if import.optional && import.import_type != ImportType::Css {
            if let Err(Error::ImportError { .. }) = self.resolve_import_path(&import.path) {
                return Ok(());
            }
        }

        match import.import_type {
            ImportType::Css => {
                // CSS imports are passed through as-is
                self.add_mapping(&import.position, Some("@import"));
                self.add_indent();
                self.write_str("@import ");
                self.write_str(&format!("\"{}\"", import.path));
                if let Some(media) = &import.media {
                    self.add_space();
                    self.write_str(media);
                }
                self.write_char(';');
                self.add_newline();
            }
            ImportType::Less => {
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

        // 检查循环依赖（multiple 导入允许同一文件多次引入）
        if !import.multiple {
            if self.imported_files.contains(&resolved_path) {
                // 默认 once 语义：跳过已导入的文件
                return Ok(());
            }

            // 添加到已导入集合
            self.imported_files.insert(resolved_path.clone());
        }

        // 保存当前基础路径和源文件
        let old_base_path = self.base_path.clone();
        let old_file = self.current_file.clone();

        // 更新基础路径和当前文件为导入文件
        self.base_path = resolved_path.parent().map(|p| p.to_path_buf());
        self.current_file = resolved_path.display().to_string();

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

        // 恢复基础路径和源文件
        self.base_path = old_base_path;
        self.current_file = old_file;

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

        // 检查循环依赖（multiple 导入允许同一文件多次引入）
        if !import.multiple {
            if self.imported_files.contains(&resolved_path) {
                return Ok(());
            }

            self.imported_files.insert(resolved_path.clone());
        }

        let old_base_path = self.base_path.clone();
        let old_file = self.current_file.clone();
        self.base_path = resolved_path.parent().map(|p| p.to_path_buf());
        self.current_file = resolved_path.display().to_string();

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

        // 使用 suppress_output 标志来处理引用导入
        // 这允许我们复用完整的 compile_stylesheet 逻辑（包括嵌套 mixins, 变量等）
        // 而不会生成任何 CSS 输出
        let old_suppress = self.suppress_output;
        self.suppress_output = true;

        self.compile_stylesheet(&stylesheet)?;

        self.suppress_output = old_suppress;
        self.base_path = old_base_path;
        self.current_file = old_file;

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

        // Inline import maps directly to the imported file content.
        let old_file = self.current_file.clone();
        self.current_file = resolved_path.display().to_string();

        let mut src_line = 1usize;
        for line in content.split_inclusive('\n') {
            let pos = Position::new(src_line, 1);
            self.add_mapping(&pos, None);
            self.write_str(line);
            src_line += 1;
        }

        // Handle files without trailing newline while preserving old behavior.
        if !content.ends_with('\n') {
            self.add_newline();
        }

        self.current_file = old_file;

        Ok(())
    }
}
