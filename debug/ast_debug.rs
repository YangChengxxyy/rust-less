//! AST 调试工具
//!
//! 这个工具用于详细检查 AST 结构，找出 "0 auto" 被解析成什么

use std::env;
use std::process::Command;

fn main() {
    println!("🔍 AST 结构调试器");
    println!("=================\n");

    // 创建一个包含调试信息的临时 Rust 程序
    let debug_program = r#"
use rust_less::*;

fn main() {
    println!("🔍 调试 AST 解析");

    let test_cases = vec![
        ".test { margin: 0 auto; }",
        ".test { margin: 0px auto; }",
        "@zero: 0; .test { margin: @zero auto; }",
    ];

    for (i, input) in test_cases.iter().enumerate() {
        println!("\n📋 测试案例 {}: {}", i + 1, input);

        // 直接使用编译器 API
        match rust_less::compile(input) {
            Ok(css) => {
                println!("✅ 编译成功");
                println!("输出: {}", css.trim());

                if css.contains("0auto") {
                    println!("❌ 发现格式问题！");
                }
            }
            Err(e) => {
                println!("❌ 编译失败: {}", e);
            }
        }
    }
}
"#;

    // 写入临时文件
    let temp_file = "/tmp/ast_debug.rs";
    if let Err(e) = std::fs::write(temp_file, debug_program) {
        println!("❌ 写入临时文件失败: {}", e);
        return;
    }

    // 编译并运行调试程序
    println!("🔧 编译调试程序...");
    let compile_result = Command::new("rustc")
        .arg("--extern")
        .arg("rust_less=./target/debug/librust_less.rlib")
        .arg(temp_file)
        .arg("-o")
        .arg("/tmp/ast_debug")
        .arg("-L")
        .arg("./target/debug/deps")
        .output();

    match compile_result {
        Ok(output) => {
            if output.status.success() {
                println!("✅ 编译成功，运行调试程序...\n");

                let run_result = Command::new("/tmp/ast_debug").output();
                match run_result {
                    Ok(run_output) => {
                        if run_output.status.success() {
                            println!("{}", String::from_utf8_lossy(&run_output.stdout));
                        } else {
                            println!("❌ 运行失败:");
                            println!("{}", String::from_utf8_lossy(&run_output.stderr));
                        }
                    }
                    Err(e) => {
                        println!("❌ 执行失败: {}", e);
                    }
                }
            } else {
                println!("❌ 编译失败:");
                println!("{}", String::from_utf8_lossy(&output.stderr));

                // 尝试更简单的方法
                println!("\n🔄 尝试替代方法...");
                simple_debug();
            }
        }
        Err(e) => {
            println!("❌ 编译命令失败: {}", e);
            println!("\n🔄 尝试替代方法...");
            simple_debug();
        }
    }

    // 清理
    let _ = std::fs::remove_file(temp_file);
    let _ = std::fs::remove_file("/tmp/ast_debug");
}

fn simple_debug() {
    println!("📋 简单调试 - 使用 CLI 工具");

    let test_cases = vec![
        (".test { margin: 0 auto; }", "基本问题"),
        (".test { margin: 0px auto; }", "带单位正常"),
        (".test { margin: 10px auto; }", "其他数字"),
        (".test { margin: auto 0; }", "顺序相反"),
    ];

    for (input, description) in test_cases {
        println!("\n🧪 {}: {}", description, input);

        // 写入临时 LESS 文件
        if let Ok(()) = std::fs::write("/tmp/test.less", input) {
            // 使用 CLI 编译
            let output = Command::new("./target/debug/rust-less")
                .arg("/tmp/test.less")
                .output();

            match output {
                Ok(result) => {
                    if result.status.success() {
                        let css = String::from_utf8_lossy(&result.stdout);
                        println!("输出: {}", css.trim());

                        // 分析问题
                        if css.contains("0auto") {
                            println!("❌ 格式问题: 找到 '0auto'");

                            // 字节级分析
                            let bytes = css.as_bytes();
                            for (i, window) in bytes.windows(6).enumerate() {
                                if let Ok(s) = std::str::from_utf8(window) {
                                    if s.contains("0auto") {
                                        println!("   字节位置 {}: {:?}", i, s);

                                        // 显示前后字符
                                        if i > 0 && i + 7 < bytes.len() {
                                            if let Ok(context) =
                                                std::str::from_utf8(&bytes[i - 1..i + 7])
                                            {
                                                println!("   上下文: {:?}", context);
                                            }
                                        }
                                    }
                                }
                            }
                        } else if css.contains("0 auto") {
                            println!("✅ 格式正确");
                        } else {
                            println!("ℹ️  其他格式");
                        }
                    } else {
                        let error = String::from_utf8_lossy(&result.stderr);
                        println!("❌ 编译失败: {}", error);
                    }
                }
                Err(e) => {
                    println!("❌ 执行失败: {}", e);
                }
            }
        }
    }

    // 清理
    let _ = std::fs::remove_file("/tmp/test.less");

    println!("\n🎯 分析结论:");
    println!("============");
    println!("基于观察到的行为:");
    println!("1. 问题出现在无单位数字 + 标识符的组合");
    println!("2. 有单位数字 + 标识符工作正常");
    println!("3. 变量替换后工作正常");
    println!();
    println!("可能的原因:");
    println!("- 解析器将 '0 auto' 解析为了错误的结构");
    println!("- 表达式求值过程中连接了值");
    println!("- to_css() 方法中有bug");
    println!();
    println!("下一步调试:");
    println!("- 检查解析器的 parse_declaration_value()");
    println!("- 查看是否创建了 BinaryOp 而不是 List");
    println!("- 验证 List 的 to_css() 实现");
}
