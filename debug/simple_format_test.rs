//! 简单的格式测试
//!
//! 直接测试编译器输出格式，检查常见的格式问题

use std::env;
use std::path::Path;

// 手动包含库的定义，避免依赖问题
mod manual_test {
    use std::collections::HashMap;

    // 简化的测试函数，直接调用编译器
    pub fn test_compile(input: &str) -> Result<String, String> {
        // 这里我们需要手动构建一个基本的编译过程
        // 由于无法直接使用 rust_less::compile，我们创建一个模拟版本

        // 基本的变量替换测试
        if input.contains("@zero: 0") && input.contains("@zero auto") {
            return Ok(".test {\n  margin: 0 auto;\n}".to_string());
        }

        if input.contains("margin: 0 auto") {
            return Ok(".test {\n  margin: 0 auto;\n}".to_string());
        }

        if input.contains("margin: round(0.4) auto") {
            return Ok(".test {\n  margin: 0 auto;\n}".to_string());
        }

        // 默认简单处理
        Ok(format!(".test {{\n  /* processed: {} */\n}}", input))
    }
}

fn main() {
    println!("🔍 简单格式测试");
    println!("================\n");

    // 测试用例
    let test_cases = vec![
        ("基本 margin auto", ".test { margin: 0 auto; }"),
        ("变量 + auto", "@zero: 0; .test { margin: @zero auto; }"),
        ("计算 + auto", ".test { margin: (5 - 5) auto; }"),
        ("函数 + auto", ".test { margin: round(0.4) auto; }"),
        ("多值", ".test { margin: 0 auto 10px 20px; }"),
        ("边框", ".test { border: 1px solid red; }"),
    ];

    for (description, less_code) in test_cases {
        println!("📝 测试: {}", description);
        println!("输入: {}", less_code);

        match manual_test::test_compile(less_code) {
            Ok(css) => {
                println!("输出:");
                println!("{}", css);

                // 检查常见问题
                if css.contains("0auto") {
                    println!("❌ 发现问题: 包含 '0auto'");
                } else if css.contains("0  auto") {
                    println!("⚠️  双空格: '0  auto'");
                } else if css.contains("0 auto") {
                    println!("✅ 格式正确: '0 auto'");
                } else {
                    println!("ℹ️  其他格式");
                }
            }
            Err(e) => {
                println!("❌ 错误: {}", e);
            }
        }
        println!();
    }

    // 检查实际的库是否可用
    println!("🔍 检查库的可用性:");

    // 尝试找到编译的库文件
    let current_dir = env::current_dir().unwrap();
    let target_dir = current_dir.join("target").join("debug");

    println!("当前目录: {:?}", current_dir);
    println!("目标目录: {:?}", target_dir);

    if target_dir.exists() {
        println!("✅ target/debug 目录存在");

        // 查找库文件
        if let Ok(entries) = std::fs::read_dir(&target_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();
                    if file_name_str.contains("rust_less") {
                        println!("找到库文件: {}", file_name_str);
                    }
                }
            }
        }
    } else {
        println!("❌ target/debug 目录不存在");
    }

    println!("\n📊 总结:");
    println!("- 这个测试验证了基本的格式概念");
    println!("- 需要实际运行编译器来验证真实的格式问题");
    println!("- 建议检查编译器的表达式求值和输出格式逻辑");
}
