use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== 🧪 Rust LESS 编译器功能状态测试 ===\n");

    let mut passed_tests = 0;
    let mut total_tests = 0;

    // Helper function to test and report
    let mut test_feature = |name: &str, less_code: &str, expected_keywords: &[&str]| {
        total_tests += 1;
        println!("🧪 测试: {}", name);

        let mut compiler = rust_less::compiler::Compiler::new();
        match compiler.compile(less_code) {
            Ok(css) => {
                println!("✅ 编译成功");

                let mut all_found = true;
                for keyword in expected_keywords {
                    if css.contains(keyword) {
                        println!("   ✓ 包含 '{}'", keyword);
                    } else {
                        println!("   ✗ 缺少 '{}'", keyword);
                        all_found = false;
                    }
                }

                if all_found {
                    println!("🎉 测试通过!\n");
                    passed_tests += 1;
                } else {
                    println!("⚠️  测试部分通过\n");
                }
            }
            Err(e) => {
                println!("❌ 编译失败: {}\n", e);
            }
        }
    };

    // 1. 测试变量功能
    test_feature(
        "变量声明和使用",
        r#"
@primary-color: #333;
@margin: 10px;

.header {
    color: @primary-color;
    margin: @margin;
}
"#,
        &[".header", "color: #333", "margin: 10px"],
    );

    // 2. 测试算术运算
    test_feature(
        "算术运算",
        r#"
@base: 10px;
.container {
    width: @base * 2;
    height: @base + 5px;
    margin: @base / 2;
}
"#,
        &[".container", "width: 20px", "height: 15px", "margin: 5px"],
    );

    // 3. 测试数学函数
    test_feature(
        "数学函数",
        r#"
.math {
    rounded: round(10.6px);
    ceiling: ceil(10.1px);
    percentage: percentage(0.5);
}
"#,
        &[".math", "rounded: 11px", "ceiling: 11px", "percentage: 50%"],
    );

    // 4. 测试媒体查询嵌套 (新功能)
    test_feature(
        "媒体查询嵌套",
        r#"
@media (min-width: 768px) {
    .container {
        width: 100%;

        @media (max-width: 1200px) {
            width: 80%;
        }
    }
}
"#,
        &[
            "@media (min-width: 768px) and (max-width: 1200px)",
            ".container",
            "width: 80%",
        ],
    );

    // 5. 测试 CSS 规则内的媒体查询
    test_feature(
        "CSS规则内媒体查询",
        r#"
.responsive {
    width: 100%;

    @media (max-width: 768px) {
        width: 50%;
    }
}
"#,
        &[
            ".responsive",
            "width: 100%",
            "@media (max-width: 768px)",
            "width: 50%",
        ],
    );

    // 6. 测试基本选择器嵌套 (应该失败或部分工作)
    test_feature(
        "基本选择器嵌套 (预期失败)",
        r#"
.navbar {
    height: 60px;

    ul {
        margin: 0;

        li {
            list-style: none;
        }
    }
}
"#,
        &[".navbar ul", ".navbar ul li", "list-style: none"],
    );

    // 7. 测试父选择器引用 (应该失败)
    test_feature(
        "父选择器引用 (&) (预期失败)",
        r#"
.button {
    padding: 10px;

    &:hover {
        background: #eee;
    }
}
"#,
        &[".button:hover", "background: #eee"],
    );

    // 8. 测试混合器 (应该失败)
    test_feature(
        "混合器 (预期失败)",
        r#"
.border-radius(@radius: 5px) {
    border-radius: @radius;
}

.button {
    .border-radius(10px);
}
"#,
        &[".button", "border-radius: 10px"],
    );

    // 9. 测试变量插值 (应该失败)
    test_feature(
        "变量插值 (预期失败)",
        r#"
@selector: "header";
@property: "color";

.@{selector} {
    @{property}: red;
}
"#,
        &[".header", "color: red"],
    );

    // 10. 测试导入 (应该失败或警告)
    test_feature(
        "导入功能 (预期失败)",
        r#"
@import "variables.less";

.main {
    color: red;
}
"#,
        &["@import", ".main", "color: red"],
    );

    println!("=== 📊 测试总结 ===");
    println!("总测试数: {}", total_tests);
    println!("通过测试: {}", passed_tests);
    println!("失败测试: {}", total_tests - passed_tests);
    println!(
        "通过率: {:.1}%",
        (passed_tests as f64 / total_tests as f64) * 100.0
    );

    println!("\n=== 🎯 功能状态概览 ===");
    println!("✅ 已实现功能:");
    println!("   - 变量声明和使用");
    println!("   - 算术运算 (+, -, *, /)");
    println!("   - 数学函数 (round, ceil, floor, percentage, etc.)");
    println!("   - 基本 CSS 规则编译");
    println!("   - 媒体查询嵌套媒体查询 (新增!)");
    println!("   - CSS 规则内媒体查询");
    println!("   - 压缩输出模式");
    println!("   - 错误处理");

    println!("\n❌ 待实现功能 (高优先级):");
    println!("   - 选择器嵌套 (.parent .child)");
    println!("   - 父选择器引用 (&:hover, &.active)");
    println!("   - 混合器定义和调用");
    println!("   - 变量插值 (@{{variable}})");
    println!("   - 导入系统 (@import)");
    println!("   - 扩展功能 (:extend)");
    println!("   - 完整的颜色函数");
    println!("   - 高级选择器功能");

    println!("\n🚀 建议下一步实现:");
    println!("   1. 选择器嵌套和父选择器 - 最重要的 LESS 功能");
    println!("   2. 基础混合器系统 - 代码复用核心功能");
    println!("   3. 变量插值 - 动态生成选择器和属性");
    println!("   4. 导入系统 - 模块化开发支持");

    if passed_tests >= total_tests / 2 {
        println!("\n🎉 当前实现已具备基础的 LESS 编译能力!");
    } else {
        println!("\n💪 还有很多功能待实现，继续努力!");
    }

    Ok(())
}
