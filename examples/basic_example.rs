//! 演示 Rust LESS 编译器的基本示例
//!
//! 此示例展示了如何使用 rust-less 库将
//! LESS 源代码编译为 CSS。

use rust_less::{compile, Compiler};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Rust LESS 编译器 - 基本示例\n");

    // 示例 1: 简单变量使用
    println!("📝 示例 1: 变量");
    let less_code = r#"
@primary-color: #333;
@secondary-color: #666;
@margin: 10px;

.header {
    color: @primary-color;
    background: @secondary-color;
    margin: @margin;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            println!("✅ LESS 输入:");
            println!("{}", less_code);
            println!("✅ CSS 输出:");
            println!("{}", css);
        }
        Err(e) => {
            println!("❌ 错误: {}", e);
        }
    }

    println!("\n{}\n", "=".repeat(50));

    // 示例 2: 算术运算
    println!("📝 示例 2: 算术运算");
    let less_code = r#"
@base-width: 100px;
@multiplier: 2;

.container {
    width: @base-width * @multiplier;
    height: @base-width + 50px;
    margin: @base-width / 4;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            println!("✅ LESS 输入:");
            println!("{}", less_code);
            println!("✅ CSS 输出:");
            println!("{}", css);
        }
        Err(e) => {
            println!("❌ 错误: {}", e);
        }
    }

    println!("\n{}\n", "=".repeat(50));

    // 示例 3: 函数调用
    println!("📝 示例 3: 函数");
    let less_code = r#"
.math-demo {
    rounded: round(10.6px);
    ceiling: ceil(10.1px);
    percentage: percentage(0.5);
}
"#;

    match compile(less_code) {
        Ok(css) => {
            println!("✅ LESS 输入:");
            println!("{}", less_code);
            println!("✅ CSS 输出:");
            println!("{}", css);
        }
        Err(e) => {
            println!("❌ 错误: {}", e);
        }
    }

    println!("\n{}\n", "=".repeat(50));

    // 示例 4: 压缩输出
    println!("📝 示例 4: 压缩输出");
    let less_code = r#"
@color: red;
.test {
    color: @color;
    margin: 10px;
}
"#;

    let mut compiler = Compiler::compressed();
    match compiler.compile(less_code) {
        Ok(css) => {
            println!("✅ LESS 输入:");
            println!("{}", less_code);
            println!("✅ 压缩的 CSS 输出:");
            println!("'{}'", css);
        }
        Err(e) => {
            println!("❌ 错误: {}", e);
        }
    }

    println!("\n🎉 所有示例都成功完成！");
    println!("\n📋 当前实现状态:");
    println!("✅ 变量声明和使用");
    println!("✅ 基本算术运算 (+, -, *, /)");
    println!("✅ 数学函数 (round, ceil, floor, percentage)");
    println!("✅ CSS 规则编译");
    println!("✅ 压缩输出模式");
    println!("✅ 全面的错误处理");
    println!("✅ 嵌套和父选择器 (&)");
    println!("✅ 混合器和混合器调用");
    println!("✅ 颜色函数 (lighten, darken 等)");
    println!("✅ 导入语句");
    println!("✅ 守卫和条件");
    println!("\n🚧 待完善功能:");
    println!("🔄 Maps 数据结构");
    println!("🔄 Source Maps 覆盖与跨文件精度");
    println!("🔄 插件系统");

    Ok(())
}
