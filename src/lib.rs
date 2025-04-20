//! # Rust-Less
//!
//! `rust_less` 是一个用 Rust 编写的 LESS 到 CSS 解析器和转换器库。
//! 它能够将 LESS 语法解析为抽象语法树(AST)，并将其转换为有效的 CSS。
//!
//! ## 功能特性
//!
//! - LESS 解析
//! - CSS 转换
//! - 支持嵌套选择器
//! - 支持变量
//! - 支持媒体查询

#[macro_use]
extern crate pest_derive;

pub mod parser;
pub mod selects;
pub mod utils;
mod test;

// 重新导出常用的组件，使API更加清晰
pub use parser::LessParser;
pub use parser::Rule;
pub use selects::ToCss;
// 注意：现在 select 和 media_query 模块已经是公共的，我们可以直接导出它们的类型
pub use selects::Select;
pub use selects::MediaQuery;
pub use selects::Variable;
pub use selects::Selects;

/// 解析LESS字符串并将其转换为CSS
///
/// # 参数
///
/// * `source` - 要解析的LESS源代码字符串
///
/// # 返回值
///
/// 返回转换后的CSS字符串，如果解析失败则返回错误
///
/// # 示例
///
/// ```
/// use rust_less::parse_less;
///
/// let less = ".selector { @width: 100px; width: @width; }";
/// match parse_less(less) {
///     Ok(css) => println!("Generated CSS: {}", css),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn parse_less(source: &str) -> Result<String, String> {
    let ast = parser::parse(source).map_err(|e| e.to_string())?;
    Ok(ast.to_css())
}

/// 从文件中解析LESS并转换为CSS
///
/// # 参数
///
/// * `file_path` - LESS文件的路径
///
/// # 返回值
///
/// 返回转换后的CSS字符串，如果解析失败则返回错误
///
/// # 示例
///
/// ```
/// use rust_less::parse_less_file;
///
/// match parse_less_file("path/to/style.less") {
///     Ok(css) => println!("Generated CSS: {}", css),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn parse_less_file(file_path: &str) -> Result<String, String> {
    use std::fs;
    let source = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    parse_less(&source)
}