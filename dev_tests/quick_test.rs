//! 快速测试当前 Rust LESS 编译器的实际能力
//!
//! 这个文件直接使用库API测试编译器功能

extern crate rust_less;

fn main() {
    println!("🧪 测试 Rust LESS 编译器当前能力\n");

    // 测试 1: 基本变量
    test_basic_variables();

    // 测试 2: 算术运算
    test_arithmetic();

    // 测试 3: 选择器嵌套
    test_nesting();

    // 测试 4: 父选择器引用
    test_parent_selector();

    // 测试 5: 媒体查询
    test_media_queries();

    // 测试 6: 函数调用
    test_functions();

    // 测试 7: 混合器（预期失败）
    test_mixins();

    // 测试 8: 变量插值（预期失败）
    test_interpolation();

    println!("\n✅ 测试完成！");
}

fn test_basic_variables() {
    println!("📝 测试 1: 基本变量声明和使用");

    let less_code = r#"
@primary-color: #333;
@margin: 10px;

.header {
    color: @primary-color;
    margin: @margin;
}
"#;

    match rust_less::compile(less_code) {
        Ok(css) => {
            println!("✅ 变量功能正常");
            println!(
                "   输出预览: {}",
                css.lines()
                    .filter(|l| !l.trim().is_empty())
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
        Err(e) => {
            println!("❌ 变量功能失败: {:?}", e);
        }
    }
}

fn test_arithmetic() {
    println!("\n🧮 测试 2: 算术运算");

    let less_code = r#"
@base: 10px;
.container {
    width: @base * 2;
    height: @base + 5px;
    margin: @base / 2;
}
"#;

    match rust_less::compile(less_code) {
        Ok(css) => {
            println!("✅ 算术运算正常");
            if css.contains("20px") && css.contains("15px") && css.contains("5px") {
                println!("   计算结果正确: 20px, 15px, 5px");
            } else {
                println!("   ⚠️ 计算结果可能不正确");
                println!(
                    "   输出: {}",
                    css.lines()
                        .filter(|l| l.trim().contains("px"))
                        .take(3)
                        .collect::<Vec<_>>()
                        .join(" | ")
                );
            }
        }
        Err(e) => {
            println!("❌ 算术运算失败: {:?}", e);
        }
    }
}

fn test_nesting() {
    println!("\n🪆 测试 3: 选择器嵌套");

    let less_code = r#"
.navbar {
    height: 60px;

    ul {
        margin: 0;

        li {
            list-style: none;
        }
    }
}
"#;

    match rust_less::compile(less_code) {
        Ok(css) => {
            println!("✅ 选择器嵌套正常");
            if css.contains(".navbar ul") && css.contains(".navbar ul li") {
                println!("   嵌套选择器生成正确");
            } else {
                println!("   ⚠️ 嵌套选择器可能不正确");
                println!(
                    "   输出预览: {}",
                    css.lines()
                        .filter(|l| l.contains(".navbar"))
                        .take(2)
                        .collect::<Vec<_>>()
                        .join(" | ")
                );
            }
        }
        Err(e) => {
            println!("❌ 选择器嵌套失败: {:?}", e);
        }
    }
}

fn test_parent_selector() {
    println!("\n👆 测试 4: 父选择器引用 (&)");

    let less_code = r#"
.button {
    padding: 10px;

    &:hover {
        background: #eee;
    }

    &.active {
        background: #333;
    }
}
"#;

    match rust_less::compile(less_code) {
        Ok(css) => {
            println!("✅ 父选择器引用正常");
            if css.contains(".button:hover") && css.contains(".button.active") {
                println!("   父选择器展开正确");
            } else {
                println!("   ⚠️ 父选择器展开可能不正确");
                println!(
                    "   输出预览: {}",
                    css.lines()
                        .filter(|l| l.contains(".button"))
                        .take(2)
                        .collect::<Vec<_>>()
                        .join(" | ")
                );
            }
        }
        Err(e) => {
            println!("❌ 父选择器引用失败: {:?}", e);
        }
    }
}

fn test_media_queries() {
    println!("\n📱 测试 5: 媒体查询嵌套");

    let less_code = r#"
.responsive {
    width: 100%;

    @media (max-width: 768px) {
        width: 50%;
    }
}
"#;

    match rust_less::compile(less_code) {
        Ok(css) => {
            println!("✅ 媒体查询功能正常");
            if css.contains("@media") && css.contains(".responsive") {
                println!("   媒体查询嵌套正确");
            } else {
                println!("   ⚠️ 媒体查询处理可能不正确");
            }
            println!(
                "   输出预览: {}",
                css.lines()
                    .filter(|l| l.contains("@media") || l.contains(".responsive"))
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
        }
        Err(e) => {
            println!("❌ 媒体查询功能失败: {:?}", e);
        }
    }
}

fn test_functions() {
    println!("\n🔢 测试 6: 内置函数");

    let less_code = r#"
.math {
    rounded: round(10.6px);
    ceiling: ceil(10.1px);
    percentage: percentage(0.5);
}
"#;

    match rust_less::compile(less_code) {
        Ok(css) => {
            println!("✅ 函数调用正常");
            if css.contains("11px") && css.contains("50%") {
                println!("   函数计算结果正确");
            } else {
                println!("   ⚠️ 函数计算结果可能不正确");
            }
            println!(
                "   输出预览: {}",
                css.lines()
                    .filter(|l| l.trim().contains(":"))
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
        }
        Err(e) => {
            println!("❌ 函数调用失败: {:?}", e);
        }
    }
}

fn test_mixins() {
    println!("\n🔀 测试 7: 混合器（预期失败）");

    let less_code = r#"
.border-radius(@radius: 5px) {
    border-radius: @radius;
    -webkit-border-radius: @radius;
}

.button {
    .border-radius(10px);
    padding: 10px;
}
"#;

    match rust_less::compile(less_code) {
        Ok(css) => {
            println!("🎉 意外成功: 混合器功能可用！");
            if css.contains("border-radius: 10px") {
                println!("   混合器参数传递正确");
            }
            println!(
                "   输出预览: {}",
                css.lines()
                    .filter(|l| l.trim().contains("border"))
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
        }
        Err(e) => {
            println!("❌ 预期失败: 混合器尚未实现");
            println!("   错误类型: {:?}", e);
        }
    }
}

fn test_interpolation() {
    println!("\n🔗 测试 8: 变量插值（预期失败）");

    let less_code = r#"
@selector: "header";
@property: "color";

.@{selector} {
    @{property}: red;
}
"#;

    match rust_less::compile(less_code) {
        Ok(css) => {
            println!("🎉 意外成功: 变量插值功能可用！");
            if css.contains(".header") && css.contains("color: red") {
                println!("   插值展开正确");
            }
            println!(
                "   输出预览: {}",
                css.lines()
                    .filter(|l| !l.trim().is_empty())
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
        }
        Err(e) => {
            println!("❌ 预期失败: 变量插值尚未实现");
            println!("   错误类型: {:?}", e);
        }
    }
}
