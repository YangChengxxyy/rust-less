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
    let mut compiler = build_compiler_from_options(&options);
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
    let mut compiler = build_compiler_from_options(&options);
    compiler.compile_file(path)
}

fn build_compiler_from_options(options: &CompilerOptions) -> Compiler {
    let mut compiler = if options.compress {
        Compiler::compressed()
    } else {
        Compiler::new()
    };

    if options.source_map {
        compiler = compiler.with_source_map(true);
    }

    for include_path in &options.include_paths {
        compiler.add_include_path(include_path);
    }

    compiler
}

/// 编译器配置选项
#[derive(Debug, Clone, Default)]
pub struct CompilerOptions {
    /// 是否压缩输出的 CSS
    pub compress: bool,
    /// 包含源码映射
    pub source_map: bool,
    /// 导入的额外包含路径
    pub include_paths: Vec<String>,
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
            include_paths: vec![],
        };
        let mut compiler = build_compiler_from_options(&options);
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
            include_paths: vec![],
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
