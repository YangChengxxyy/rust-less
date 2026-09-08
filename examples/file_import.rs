//! 文件导入示例
//!
//! 此示例演示如何使用 rust-less 编译包含导入的 LESS 文件。
//!
//! 运行方式：
//! ```bash
//! cargo run --example file_import
//! ```

use rust_less::{compile_file, Compiler, CompilerOptions, Error};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rust LESS 文件导入示例 ===\n");

    // 获取测试 fixtures 目录路径
    let fixtures_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");

    // 示例 1: 编译单个文件
    println!("1. 编译单个 LESS 文件");
    println!("   文件: tests/fixtures/variables.less");
    println!("   ---");

    let variables_path = fixtures_path.join("variables.less");
    match compile_file(&variables_path) {
        Ok(css) => {
            if css.trim().is_empty() {
                println!("   输出: (空 - 变量定义不产生 CSS 输出)");
            } else {
                println!("   输出:\n{}", indent_text(&css, "   "));
            }
        }
        Err(e) => println!("   错误: {}", e),
    }
    println!();

    // 示例 2: 编译包含导入的文件
    println!("2. 编译包含 @import 的文件");
    println!("   文件: tests/fixtures/main.less");
    println!("   ---");

    let main_path = fixtures_path.join("main.less");
    match compile_file(&main_path) {
        Ok(css) => {
            println!("   输出:\n{}", indent_text(&css, "   "));
        }
        Err(e) => println!("   错误: {}", e),
    }
    println!();

    // 示例 3: 使用 include paths 从字符串编译
    println!("3. 使用 include paths 编译");
    println!("   ---");

    let less_code = r#"
@import "variables.less";

.custom-component {
    color: @primary-color;
    font-size: @font-size-base;
}
"#;

    let mut compiler = Compiler::new();
    compiler.add_include_path(&fixtures_path);

    match compiler.compile(less_code) {
        Ok(css) => {
            println!("   输入 LESS:");
            println!("{}", indent_text(less_code.trim(), "   "));
            println!("\n   输出 CSS:");
            println!("{}", indent_text(&css, "   "));
        }
        Err(e) => println!("   错误: {}", e),
    }
    println!();

    // 示例 4: 压缩输出
    println!("4. 压缩输出模式");
    println!("   ---");

    let options = CompilerOptions {
        compress: true,
        source_map: false,
        source_map_lessjs_compat: false,
        include_paths: vec![fixtures_path.to_string_lossy().to_string()],
        ..Default::default()
    };

    let less_code = r#"
@import "variables.less";

.compressed-output {
    color: @primary-color;
    padding: @spacing-unit;
}
"#;

    match rust_less::compile_with_options(less_code, options) {
        Ok(css) => {
            println!("   压缩后的 CSS:");
            println!("   {}", css.trim());
        }
        Err(e) => println!("   错误: {}", e),
    }
    println!();

    // 示例 5: 处理导入错误
    println!("5. 处理导入错误");
    println!("   ---");

    let less_with_missing_import = r#"
@import "nonexistent-file.less";

.test {
    color: red;
}
"#;

    match rust_less::compile(less_with_missing_import) {
        Ok(_) => println!("   意外成功"),
        Err(e) => {
            println!("   预期的错误: {}", e);
            if let Error::ImportError { path, reason, .. } = e {
                println!("   - 文件: {}", path);
                println!("   - 原因: {}", reason);
            }
        }
    }
    println!();

    // 示例 6: CSS 导入透传
    println!("6. CSS 导入透传");
    println!("   ---");

    let less_with_css_import = r#"
@import "reset.css";
@import "theme.css" screen;

.local-style {
    color: blue;
}
"#;

    match rust_less::compile(less_with_css_import) {
        Ok(css) => {
            println!("   输入 LESS:");
            println!("{}", indent_text(less_with_css_import.trim(), "   "));
            println!("\n   输出 CSS:");
            println!("{}", indent_text(&css, "   "));
        }
        Err(e) => println!("   错误: {}", e),
    }
    println!();

    // 示例 7: 嵌套目录导入
    println!("7. 嵌套目录导入");
    println!("   文件: tests/fixtures/nested/deep.less");
    println!("   ---");

    let nested_path = fixtures_path.join("nested").join("deep.less");
    if nested_path.exists() {
        match compile_file(&nested_path) {
            Ok(css) => {
                println!("   输出:\n{}", indent_text(&css, "   "));
            }
            Err(e) => println!("   错误: {}", e),
        }
    } else {
        println!("   文件不存在，跳过此示例");
    }
    println!();

    // 示例 8: 循环导入处理
    println!("8. 循环导入处理");
    println!("   文件: tests/fixtures/circular-a.less (互相导入 circular-b.less)");
    println!("   ---");

    let circular_path = fixtures_path.join("circular-a.less");
    if circular_path.exists() {
        match compile_file(&circular_path) {
            Ok(css) => {
                println!("   循环导入被正确处理！");
                println!("   输出:\n{}", indent_text(&css, "   "));
            }
            Err(e) => println!("   错误: {}", e),
        }
    } else {
        println!("   文件不存在，跳过此示例");
    }

    println!("\n=== 示例完成 ===");
    Ok(())
}

/// 给文本的每一行添加缩进
fn indent_text(text: &str, indent: &str) -> String {
    text.lines()
        .map(|line| format!("{}{}", indent, line))
        .collect::<Vec<_>>()
        .join("\n")
}
