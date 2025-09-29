//! # Rust LESS 编译器
//!
//! 一个用 Rust 编写的全面的 LESS 到 CSS 编译器库。
//! 此库提供将 LESS 样式表解析、编译和转换为 CSS 的功能。

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod ast;
pub mod compiler;
pub mod error;
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

/// 使用自定义选项将 LESS 源代码编译为 CSS
pub fn compile_with_options(input: &str, _options: CompilerOptions) -> Result<String> {
    // 待办：实现选项支持
    compile(input)
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
}
