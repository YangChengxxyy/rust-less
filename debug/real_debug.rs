//! 使用实际编译器API的调试测试
//!
//! 这个文件直接使用实际的编译器来调试格式问题

use std::collections::HashMap;

// 我们需要手动模拟编译器的部分，因为无法直接导入
// 但我们可以使用系统命令来调用编译好的二进制文件

fn main() {
    println!("🔍 实际编译器调试");
    println!("==================\n");

    // 测试用例
    let test_cases = vec![
        ("基本问题", ".test { margin: 0 auto; }"),
        ("变量正常", "@zero: 0; .test { margin: @zero auto; }"),
        ("带单位正常", ".test { margin: 0px auto; }"),
        ("纯数字问题", ".test { width: 0 100px; }"),
        ("多个数字", ".test { margin: 0 1 2 3; }"),
        ("混合值", ".test { margin: 0 auto 10px; }"),
        ("函数调用", ".test { margin: round(0.4) auto; }"),
        ("计算表达式", ".test { margin: (5-5) auto; }"),
    ];

    for (name, less_code) in test_cases {
        println!("📝 测试: {}", name);
        println!("输入: {}", less_code);

        // 写入临时文件
        let temp_file = "/tmp/test.less";
        if let Err(e) = std::fs::write(temp_file, less_code) {
            println!("❌ 写入文件失败: {}", e);
            continue;
        }

        // 使用编译器编译
        let output = std::process::Command::new("./target/debug/rust-less")
            .arg(temp_file)
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    let css = String::from_utf8_lossy(&result.stdout);
                    println!("输出: {}", css.trim());

                    // 检查格式问题
                    if css.contains("0auto") {
                        println!("❌ 发现问题: 包含 '0auto'");

                        // 进一步分析
                        let bytes = css.as_bytes();
                        for (i, window) in bytes.windows(5).enumerate() {
                            if let Ok(s) = std::str::from_utf8(window) {
                                if s.contains("0auto") {
                                    println!("   位置 {}: {:?}", i, s);

                                    // 显示前后字符
                                    if i > 0 {
                                        if let Ok(before) =
                                            std::str::from_utf8(&bytes[i - 1..i + 6])
                                        {
                                            println!("   上下文: {:?}", before);
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

        println!();
    }

    // 清理临时文件
    let _ = std::fs::remove_file("/tmp/test.less");

    println!("🎯 总结:");
    println!("如果问题确实存在，我们需要深入检查:");
    println!("1. 词法分析器是否正确分离 token");
    println!("2. 解析器是否正确创建列表");
    println!("3. 编译器是否正确输出列表");
    println!("4. 表达式求值是否改变了结构");
}
