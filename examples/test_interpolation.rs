//! 变量插值功能测试示例
//!
//! 这个示例演示了 rust-less 编译器的变量插值功能，包括：
//! - 选择器中的变量插值
//! - 属性名中的变量插值
//! - 属性值中的变量插值
//! - 复杂的插值组合

use rust_less::compile;

fn main() {
    println!("🧪 Rust LESS 编译器 - 变量插值功能测试");
    println!("=========================================\n");

    // 测试 1: 选择器插值
    test_selector_interpolation();

    // 测试 2: 属性名插值
    test_property_interpolation();

    // 测试 3: 属性值插值
    test_value_interpolation();

    // 测试 4: 复杂插值组合
    test_complex_interpolation();

    // 测试 5: 错误处理
    test_error_handling();

    println!("\n🎯 变量插值测试完成");
}

fn test_selector_interpolation() {
    println!("📝 测试 1: 选择器插值");
    println!("─────────────────────");

    let less_code = r#"
@prefix: nav;
@suffix: item;

.@{prefix} {
    background: #333;

    .@{suffix} {
        color: white;
        padding: 10px;

        &:hover {
            background: #555;
        }
    }
}

.@{prefix}-@{suffix} {
    border: 1px solid #ccc;
    margin: 5px;
}
"#;

    print_test_input(less_code);
    execute_test(less_code);
}

fn test_property_interpolation() {
    println!("\n📝 测试 2: 属性名插值");
    println!("─────────────────────");

    let less_code = r#"
@property: background;
@direction: left;
@size: width;

.dynamic-properties {
    @{property}: red;
    @{property}-color: blue;
    border-@{direction}: 2px solid black;
    min-@{size}: 200px;
    max-@{size}: 800px;
}

.responsive {
    @{size}: 100%;

    @media (max-width: 768px) {
        @{size}: 90%;
    }
}
"#;

    print_test_input(less_code);
    execute_test(less_code);
}

fn test_value_interpolation() {
    println!("\n📝 测试 3: 属性值插值");
    println!("─────────────────────");

    let less_code = r#"
@base-color: #007bff;
@spacing: 10px;
@font-family: Arial, sans-serif;
@border-style: solid;

.interpolated-values {
    color: @{base-color};
    padding: @{spacing};
    margin: calc(@{spacing} * 2);
    font-family: @{font-family};
    border: 1px @{border-style} @{base-color};
}

.computed-values {
    padding: 15px;
    background: linear-gradient(to right, @{base-color}, darken(@{base-color}, 20%));
}
"#;

    print_test_input(less_code);
    execute_test(less_code);
}

fn test_complex_interpolation() {
    println!("\n📝 测试 4: 复杂插值组合");
    println!("───────────────────────");

    let less_code = r#"
@component: button;
@state: hover;
@property-prefix: border;

// 动态生成组件样式
.@{component} {
    padding: 10px;

    &:@{state} {
        @{property-prefix}-color: #333;
        @{property-prefix}-width: 2px;
    }
}

// 主题系统
@theme: dark;
@primary: #ffffff;
@secondary: #333333;

.theme-@{theme} {
    .@{component} {
        background: @{secondary};
        color: @{primary};

        &:@{state} {
            background: lighten(@{secondary}, 10%);
        }
    }
}

// 网格系统
@columns: 12;
@breakpoint: md;

.col-@{breakpoint}-@{columns} {
    width: 100%;

    @media (min-width: 768px) {
        width: percentage(12 / @{columns});
    }
}
"#;

    print_test_input(less_code);
    execute_test(less_code);
}

fn test_error_handling() {
    println!("\n📝 测试 5: 错误处理");
    println!("─────────────────────");

    // 测试 5a: 未定义的插值变量
    println!("\n5a. 未定义的插值变量:");
    let undefined_var = r#"
.test {
    .@{undefined-selector} {
        color: red;
    }
}
"#;

    print_test_input(undefined_var);
    match compile(undefined_var) {
        Ok(css) => {
            println!("⚠️  意外成功:");
            println!("```css");
            println!("{}", css.trim());
            println!("```");
        }
        Err(e) => {
            println!("✅ 正确捕获错误: {}", e);
        }
    }

    // 测试 5b: 属性名插值错误
    println!("\n5b. 属性名插值错误:");
    let invalid_property = r#"
@invalid: ;

.test {
    @{invalid}: red;  // 空属性名
}
"#;

    print_test_input(invalid_property);
    match compile(invalid_property) {
        Ok(css) => {
            println!("⚠️  意外成功:");
            println!("```css");
            println!("{}", css.trim());
            println!("```");
        }
        Err(e) => {
            println!("✅ 正确捕获错误: {}", e);
        }
    }

    // 测试 5c: 嵌套插值
    println!("\n5c. 嵌套插值:");
    let nested_interpolation = r#"
@outer: inner;
@inner: value;

.test {
    content: @{@{outer}};  // 嵌套插值
}
"#;

    print_test_input(nested_interpolation);
    match compile(nested_interpolation) {
        Ok(css) => {
            println!("✅ 编译成功 (嵌套插值支持):");
            println!("```css");
            println!("{}", css.trim());
            println!("```");
        }
        Err(e) => {
            println!("ℹ️  当前不支持嵌套插值: {}", e);
        }
    }
}

fn print_test_input(less_code: &str) {
    println!("输入 LESS 代码:");
    println!("```less");
    println!("{}", less_code.trim());
    println!("```");
}

fn execute_test(less_code: &str) {
    match compile(less_code) {
        Ok(css) => {
            println!("\n✅ 编译成功!");
            println!("输出 CSS:");
            println!("```css");
            println!("{}", css.trim());
            println!("```");
        }
        Err(e) => {
            println!("\n❌ 编译失败: {}", e);
        }
    }
}
