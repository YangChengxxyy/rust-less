//! 测试格式问题
//!
//! 这个测试文件用于重现和调试输出格式问题，特别是 "0auto" 应该输出为 "0 auto" 的问题

use rust_less::compile;

fn main() {
    println!("🔍 测试输出格式问题");
    println!("====================\n");

    // 测试可能导致 "0auto" 问题的各种情况
    test_space_concatenation();
    test_list_formatting();
    test_property_values();
    test_function_results();
}

fn test_space_concatenation() {
    println!("📝 测试空格连接问题");
    println!("─────────────────");

    let test_cases = vec![
        // 可能的 0auto 问题来源
        ("@margin: 0 auto;", "简单的 0 auto"),
        ("@padding: 0px auto;", "0px auto"),
        ("@value: 0; .test { margin: @value auto; }", "变量 + auto"),
        (
            "@zero: 0; @auto: auto; .test { margin: @zero @auto; }",
            "两个变量",
        ),
        // 列表和空格分隔符
        (".test { margin: 0 auto 10px; }", "三值边距"),
        (".test { padding: 0 auto 0 auto; }", "四值内边距"),
        // 计算结果
        (".test { margin: (5 - 5) auto; }", "计算结果 + auto"),
        (".test { margin: 0 + 0 auto; }", "加法结果 + auto"),
        // 函数调用结果
        (".test { margin: round(0.4) auto; }", "函数结果 + auto"),
    ];

    for (less_code, description) in test_cases {
        println!("\n🧪 测试: {}", description);
        println!("输入: {}", less_code);

        match compile(less_code) {
            Ok(css) => {
                println!("输出: {}", css.trim());

                // 检查是否有格式问题
                if css.contains("0auto") {
                    println!("❌ 发现问题: 包含 '0auto'，应该是 '0 auto'");
                } else if css.contains("0  auto") {
                    println!("⚠️  可能问题: 包含双空格 '0  auto'");
                } else {
                    println!("✅ 格式正确");
                }
            }
            Err(e) => {
                println!("❌ 编译失败: {}", e);
            }
        }
    }
}

fn test_list_formatting() {
    println!("\n\n📝 测试列表格式化");
    println!("─────────────────");

    let test_cases = vec![
        // 空格分隔的列表
        (".test { margin: 10px 20px 30px 40px; }", "四值列表"),
        (".test { border: 1px solid red; }", "边框简写"),
        (".test { font: 14px Arial, sans-serif; }", "字体简写"),
        // 逗号分隔的列表
        (".test { background: url(a.jpg), url(b.jpg); }", "多背景"),
        (".test { color: rgb(255, 0, 0); }", "RGB 颜色"),
        // 混合分隔符
        (
            ".test { box-shadow: 0 0 10px rgba(0,0,0,0.5), inset 0 1px white; }",
            "复杂阴影",
        ),
    ];

    for (less_code, description) in test_cases {
        println!("\n🧪 测试: {}", description);
        println!("输入: {}", less_code);

        match compile(less_code) {
            Ok(css) => {
                println!("输出: {}", css.trim());

                // 检查各种格式问题
                if css.contains("  ") {
                    println!("⚠️  发现双空格");
                }
                if css.contains(", ") && css.contains(",") && !css.contains(", ") {
                    println!("⚠️  逗号后缺少空格");
                }
            }
            Err(e) => {
                println!("❌ 编译失败: {}", e);
            }
        }
    }
}

fn test_property_values() {
    println!("\n\n📝 测试属性值格式");
    println!("─────────────────");

    let test_cases = vec![
        // 各种可能导致连接问题的属性值
        (".test { margin: 0; }", "单一数值"),
        (".test { margin: auto; }", "单一关键字"),
        (".test { position: absolute; top: 0; left: 0; }", "多个属性"),
        // 变量相关
        ("@base: 0; .test { margin: @base; }", "变量值"),
        ("@unit: px; .test { width: 100@unit; }", "单位变量"),
        // 表达式
        (".test { width: 100 + 200; }", "加法表达式"),
        (".test { width: 100 - 50; }", "减法表达式"),
        (".test { width: 100 * 2; }", "乘法表达式"),
        (".test { width: 100 / 2; }", "除法表达式"),
    ];

    for (less_code, description) in test_cases {
        println!("\n🧪 测试: {}", description);
        println!("输入: {}", less_code);

        match compile(less_code) {
            Ok(css) => {
                println!("输出: {}", css.trim());
            }
            Err(e) => {
                println!("❌ 编译失败: {}", e);
            }
        }
    }
}

fn test_function_results() {
    println!("\n\n📝 测试函数结果格式");
    println!("─────────────────");

    let test_cases = vec![
        // 各种函数调用
        (".test { width: percentage(0.5); }", "百分比函数"),
        (".test { width: round(10.6); }", "round 函数"),
        (".test { width: ceil(10.2); }", "ceil 函数"),
        (".test { width: floor(10.8); }", "floor 函数"),
        (".test { width: abs(-10); }", "abs 函数"),
        (".test { width: min(10, 20); }", "min 函数"),
        (".test { width: max(10, 20); }", "max 函数"),
        // 函数结果与其他值的组合
        (".test { margin: round(0.4) auto; }", "函数 + 关键字"),
        (".test { padding: 0 percentage(0.5); }", "数值 + 函数"),
        (".test { border: round(1.2) solid red; }", "函数 + 多值"),
    ];

    for (less_code, description) in test_cases {
        println!("\n🧪 测试: {}", description);
        println!("输入: {}", less_code);

        match compile(less_code) {
            Ok(css) => {
                println!("输出: {}", css.trim());

                // 检查函数结果格式
                if css.contains("NaN") {
                    println!("❌ 发现 NaN");
                }
                if css.contains("undefined") {
                    println!("❌ 发现 undefined");
                }
                if css.matches(char::is_numeric).count() > 0 && css.contains("auto") {
                    // 如果同时包含数字和 auto，检查是否有空格
                    let parts: Vec<&str> = css.split_whitespace().collect();
                    if parts
                        .iter()
                        .any(|part| part.contains("auto") && part != "auto")
                    {
                        println!("❌ 可能的连接问题: 数字和 auto 连在一起");
                    }
                }
            }
            Err(e) => {
                println!("❌ 编译失败: {}", e);
            }
        }
    }
}
