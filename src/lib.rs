//! # Rust LESS 编译器
//!
//! 一个用 Rust 编写的全面的 LESS 到 CSS 编译器库。
//! 此库提供将 LESS 样式表解析、编译和转换为 CSS 的功能。

#![warn(missing_docs)]
#![warn(clippy::all)]

use std::path::Path;

pub mod ast;
pub mod compiler;
pub mod error;
pub mod extend;
pub mod lexer;
pub mod parser;
pub mod plugin;

#[cfg(feature = "functions")]
pub mod functions;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use compiler::Compiler;
pub use error::{Error, Result};

#[cfg(feature = "wasm")]
pub use wasm::*;

/// 将 LESS 源代码编译为 CSS
pub fn compile(input: &str) -> Result<String> {
    let mut compiler = Compiler::new();
    compiler.compile(input)
}

/// 编译 LESS 文件
///
/// # 参数
/// * `path` - LESS 文件的路径
///
/// # 示例
/// ```no_run
/// use rust_less::compile_file;
///
/// fn main() -> Result<(), rust_less::Error> {
///     let css = compile_file("styles/main.less")?;
///     println!("{}", css);
///     Ok(())
/// }
/// ```
pub fn compile_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut compiler = Compiler::new();
    compiler.compile_file(path)
}

/// 使用自定义选项将 LESS 源代码编译为 CSS
pub fn compile_with_options(input: &str, options: CompilerOptions) -> Result<String> {
    let mut compiler = build_compiler_from_options(options)?;
    compiler.compile(input)
}

/// 使用自定义选项编译 LESS 文件
///
/// # 参数
/// * `path` - LESS 文件的路径
/// * `options` - 编译器选项
pub fn compile_file_with_options<P: AsRef<Path>>(
    path: P,
    options: CompilerOptions,
) -> Result<String> {
    let mut compiler = build_compiler_from_options(options)?;
    compiler.compile_file(path)
}

fn build_compiler_from_options(options: CompilerOptions) -> Result<Compiler> {
    options.build()
}

/// 编译器配置选项
///
/// 与 CLI 标志一一对应：`compress`/`source_map`/`source_map_lessjs_compat`/
/// `source_map_root`/`source_map_file`/`include_paths`。
///
/// 插件通过 `with_function_plugin` / `with_parse_hook` / `with_visitor` /
/// `with_import_resolver` 注册（见 [`plugin`](crate::plugin) 模块与
/// `docs/PLUGIN_HOOKS_DESIGN.md`）。
#[derive(Default)]
pub struct CompilerOptions {
    /// 是否压缩输出的 CSS
    pub compress: bool,
    /// 包含源码映射
    pub source_map: bool,
    /// source map 输出为 less.js 兼容模式（保留默认行为为 false）
    pub source_map_lessjs_compat: bool,
    /// source map 的 sourceRoot 字段（对应 CLI `--source-map-root`）
    pub source_map_root: Option<String>,
    /// source map 的 file 字段，通常为生成的 CSS 文件路径（对应 CLI 输出文件名）
    pub source_map_file: Option<String>,
    /// 导入的额外包含路径
    pub include_paths: Vec<String>,
    /// 自定义函数插件，与内置函数同一调用路径（后注册覆盖同名内置函数）
    pub functions: Vec<Box<dyn plugin::LessFunction>>,
    /// 自定义 at-rule 解析钩子
    pub parse_hooks: Vec<Box<dyn plugin::ParseHook>>,
    /// 编译期 visitor（规则改写 + 输出后处理）
    pub visitors: Vec<Box<dyn plugin::CompileVisitor>>,
    /// 导入解析钩子（插件 → include_paths → 默认文件系统）
    pub import_resolvers: Vec<Box<dyn plugin::ImportResolver>>,
}

impl std::fmt::Debug for CompilerOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompilerOptions")
            .field("compress", &self.compress)
            .field("source_map", &self.source_map)
            .field("source_map_lessjs_compat", &self.source_map_lessjs_compat)
            .field("source_map_root", &self.source_map_root)
            .field("source_map_file", &self.source_map_file)
            .field("include_paths", &self.include_paths)
            .field(
                "functions",
                &self.functions.iter().map(|p| p.name()).collect::<Vec<_>>(),
            )
            .field(
                "parse_hooks",
                &self
                    .parse_hooks
                    .iter()
                    .map(|p| p.name())
                    .collect::<Vec<_>>(),
            )
            .field(
                "visitors",
                &self.visitors.iter().map(|p| p.name()).collect::<Vec<_>>(),
            )
            .field(
                "import_resolvers",
                &self
                    .import_resolvers
                    .iter()
                    .map(|p| p.name())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl CompilerOptions {
    /// 注册自定义函数插件（见 [`plugin::LessFunction`]）。
    pub fn with_function_plugin(mut self, plugin: Box<dyn plugin::LessFunction>) -> Self {
        self.functions.push(plugin);
        self
    }

    /// 注册自定义 at-rule 解析钩子（见 [`plugin::ParseHook`]）。
    pub fn with_parse_hook(mut self, hook: Box<dyn plugin::ParseHook>) -> Self {
        self.parse_hooks.push(hook);
        self
    }

    /// 注册编译期 visitor（见 [`plugin::CompileVisitor`]）。
    pub fn with_visitor(mut self, visitor: Box<dyn plugin::CompileVisitor>) -> Self {
        self.visitors.push(visitor);
        self
    }

    /// 注册导入解析钩子（见 [`plugin::ImportResolver`]）。
    pub fn with_import_resolver(mut self, resolver: Box<dyn plugin::ImportResolver>) -> Self {
        self.import_resolvers.push(resolver);
        self
    }

    /// 按当前选项构建配置好的 [`Compiler`]（消耗选项，插件移入编译器）。
    ///
    /// 插件声明的 API 版本与 [`plugin::PLUGIN_API_VERSION`] 不一致时返回
    /// [`Error::PluginError`]。
    pub fn build(self) -> Result<Compiler> {
        let mut compiler = if self.compress {
            Compiler::compressed()
        } else {
            Compiler::new()
        };

        if self.source_map {
            compiler = compiler.with_source_map(true);
            compiler.set_source_map_lessjs_compat(self.source_map_lessjs_compat);
            if let Some(source_root) = &self.source_map_root {
                compiler.set_source_map_source_root(Some(source_root.clone()));
            }
            if let Some(file) = &self.source_map_file {
                compiler.set_source_map_file(Some(file.clone()));
            }
        }

        for include_path in &self.include_paths {
            compiler.add_include_path(include_path);
        }

        for function_plugin in self.functions {
            compiler.register_function_plugin(function_plugin)?;
        }
        for parse_hook in self.parse_hooks {
            compiler.register_parse_hook(parse_hook)?;
        }
        for visitor in self.visitors {
            compiler.register_visitor(visitor)?;
        }
        for resolver in self.import_resolvers {
            compiler.register_import_resolver(resolver)?;
        }

        Ok(compiler)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_basic_compilation() {
        // 这个测试最初会失败 - 这就是 TDD 方法
        let less = "@color: red; .test { color: @color; }";
        let result = compile(less);
        assert!(result.is_ok());

        let css = result.unwrap();
        assert!(css.contains(".test"));
        assert!(css.contains("color: red"));
    }

    #[test]
    fn test_build_compiler_from_options_enables_source_map() {
        let options = CompilerOptions {
            compress: false,
            source_map: true,
            source_map_lessjs_compat: false,
            include_paths: vec![],
            ..Default::default()
        };
        let mut compiler = build_compiler_from_options(options).unwrap();
        compiler.compile(".test { color: red; }").unwrap();
        assert!(compiler.generate_source_map().is_some());
    }

    #[test]
    fn test_compile_file_with_options_source_map_path() {
        let tmp_dir = std::env::temp_dir();
        let file_path = tmp_dir.join("rust_less_compile_file_with_options.less");

        let mut file = std::fs::File::create(&file_path).unwrap();
        writeln!(file, ".test {{ color: red; }}").unwrap();

        let options = CompilerOptions {
            compress: false,
            source_map: true,
            source_map_lessjs_compat: false,
            include_paths: vec![],
            ..Default::default()
        };
        let result = compile_file_with_options(&file_path, options);
        assert!(
            result.is_ok(),
            "compile_file_with_options failed: {:?}",
            result
        );

        let css = result.unwrap();
        assert!(css.contains(".test"));
        assert!(css.contains("color: red;"));

        let _ = std::fs::remove_file(file_path);
    }
}
