//! 专门测试 auto 格式的示例
//!
//! 这个示例专门测试各种可能导致 "0auto" 问题的情况，
//! 确保编译器正确处理空格分隔的值

use rust_less::compile;

fn main() {
    println!("🔍 Auto 格式测试");
    println!("================\n");

    test_basic_auto();
    test_variable_auto();
    test_calculated_auto();
    test_function_auto();
    test_multiple_values_with_auto();
    test_edge_cases();

    println!("\n🎯 测试总结");
    println!("所有 auto 格式测试完成！");
}

fn test_basic_auto() {
    println!("📝 测试 1: 基本 auto 格式");
    println!("─────────────────────────");

    let test_cases = vec![
        ".test { margin: 0 auto; }",
        ".test { margin: auto 0; }",
        ".test { margin: auto; }",
        ".test { text-align: center; margin: 0 auto; }",
    ];

    for less in test_cases {
        println!("\n输入: {}", less);

        match compile(less) {
            Ok(css) => {
                println!("输出: {}", css.trim());

                // 检查格式问题
                if css.contains("0auto") || css.contains("auto0") {
                    println!("❌ 发现格式问题: 值连接在一起！");
                } else if css.contains("0  auto") || css.contains("auto  0") {
                    println!("⚠️  发现多余空格");
                } else if css.contains("0 auto") || css.contains("auto 0") || css.contains("auto") {
                    println!("✅ 格式正确");
                } else {
                    println!("ℹ️  其他格式");
                }
            }
            Err(e) => {
                println!("❌ 编译失败: {}", e);
            }
        }
    }
}

fn test_variable_auto() {
    println!("\n\n📝 测试 2: 变量与 auto 组合");
    println!("───────────────────────────");

    let test_cases = vec![
        "@zero: 0; .test { margin: @zero auto; }",
        "@auto-val: auto; .test { margin: 0 @auto-val; }",
        "@zero: 0; @auto-val: auto; .test { margin: @zero @auto-val; }",
        "@margin: 10px; .test { margin: @margin auto; }",
    ];

    for less in test_cases {
        println!("\n输入: {}", less);

        match compile(less) {
            Ok(css) => {
                println!("输出: {}", css.trim());

                // 检查变量替换后的格式
                if css.contains("0auto") || css.contains("auto0") || css.contains("10pxauto") {
                    println!("❌ 发现格式问题: 变量值与 auto 连接！");
                } else {
                    println!("✅ 变量替换格式正确");
                }
            }
            Err(e) => {
                println!("❌ 编译失败: {}", e);
            }
        }
    }
}

fn test_calculated_auto() {
    println!("\n\n📝 测试 3: 计算结果与 auto");
    println!("─────────────────────────");

    let test_cases = vec![
        ".test { margin: (5 - 5) auto; }",
        ".test { margin: (10 + 0) auto; }",
        ".test { margin: 0 + 0 auto; }",
        ".test { margin: 20px / 2 auto; }",
        ".test { margin: 10px * 0 auto; }",
    ];

    for less in test_cases {
        println!("\n输入: {}", less);

        match compile(less) {
            Ok(css) => {
                println!("输出: {}", css.trim());

                // 检查计算结果的格式
                if css.contains("0auto") || css.contains("10pxauto") {
                    println!("❌ 发现格式问题: 计算结果与 auto 连接！");
                } else {
                    println!("✅ 计算结果格式正确");
                }
            }
            Err(e) => {
                println!("❌ 编译失败: {}", e);
            }
        }
    }
}

fn test_function_auto() {
    println!("\n\n📝 测试 4: 函数结果与 auto");
    println!("─────────────────────────");

    let test_cases = vec![
        ".test { margin: round(0.4) auto; }",
        ".test { margin: ceil(0.1) auto; }",
        ".test { margin: floor(0.9) auto; }",
        ".test { margin: abs(-10) auto; }",
        ".test { margin: percentage(0) auto; }",
    ];

    for less in test_cases {
        println!("\n输入: {}", less);

        match compile(less) {
            Ok(css) => {
                println!("输出: {}", css.trim());

                // 检查函数结果的格式
                if css.contains("0auto")
                    || css.contains("1auto")
                    || css.contains("10auto")
                    || css.contains("0%auto")
                {
                    println!("❌ 发现格式问题: 函数结果与 auto 连接！");
                } else {
                    println!("✅ 函数结果格式正确");
                }
            }
            Err(e) => {
                println!("❌ 编译失败: {}", e);
            }
        }
    }
}

fn test_multiple_values_with_auto() {
    println!("\n\n📝 测试 5: 多值属性中的 auto");
    println!("────────────────────────────");

    let test_cases = vec![
        ".test { margin: 0 auto 10px; }",
        ".test { margin: 0 auto 10px 20px; }",
        ".test { margin: 10px 0 auto; }",
        ".test { padding: 0 auto 0 auto; }",
        ".test { border-width: 1px auto 2px; }",
    ];

    for less in test_cases {
        println!("\n输入: {}", less);

        match compile(less) {
            Ok(css) => {
                println!("输出: {}", css.trim());

                // 检查多值中的格式
                if css.contains("0auto")
                    || css.contains("auto10px")
                    || css.contains("10pxauto")
                    || css.contains("1pxauto")
                {
                    println!("❌ 发现格式问题: 多值中的连接！");
                } else {
                    println!("✅ 多值格式正确");
                }
            }
            Err(e) => {
                println!("❌ 编译失败: {}", e);
            }
        }
    }
}

fn test_edge_cases() {
    println!("\n\n📝 测试 6: 边界情况");
    println!("─────────────────────");

    let test_cases = vec![
        // 混合单位
        ".test { margin: 0px auto; }",
        ".test { margin: 0em auto; }",
        ".test { margin: 0% auto; }",
        // 复杂表达式
        ".test { margin: (10px - 10px) auto (5px + 5px); }",
        // 嵌套计算
        ".test { margin: round(5.5px - 5px) auto; }",
        // 字符串值
        ".test { content: '0auto'; }", // 这个应该保持不变
        // 其他关键字组合
        ".test { position: absolute; top: 0; left: auto; }",
    ];

    for less in test_cases {
        println!("\n输入: {}", less);

        match compile(less) {
            Ok(css) => {
                println!("输出: {}", css.trim());

                // 特殊检查
                if less.contains("content: '0auto'") {
                    if css.contains("'0auto'") {
                        println!("✅ 字符串保持不变（正确）");
                    } else {
                        println!("❌ 字符串被意外修改");
                    }
                } else if css.contains("0pxauto")
                    || css.contains("0emauto")
                    || css.contains("0%auto")
                {
                    println!("❌ 发现单位格式问题");
                } else {
                    println!("✅ 边界情况处理正确");
                }
            }
            Err(e) => {
                println!("❌ 编译失败: {}", e);
            }
        }
    }
}
