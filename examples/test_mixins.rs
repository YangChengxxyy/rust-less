//! 混合器功能测试示例
//!
//! 这个示例演示了 rust-less 编译器的混合器功能，包括：
//! - 基本混合器定义和调用
//! - 带默认参数的混合器
//! - 守卫条件混合器
//! - 错误处理

use rust_less::compile;

fn main() {
    println!("🧪 Rust LESS 编译器 - 混合器功能测试");
    println!("=====================================\n");

    // 测试 1: 基本混合器定义和调用
    test_basic_mixin();

    // 测试 2: 带默认参数的混合器
    test_default_parameters();

    // 测试 3: 守卫条件混合器
    test_guarded_mixins();

    // 测试 4: 混合器中的嵌套规则
    test_nested_rules_in_mixins();

    // 测试 5: 错误处理
    test_error_handling();

    println!("\n🎯 混合器测试完成");
}

fn test_basic_mixin() {
    println!("📝 测试 1: 基本混合器");
    println!("───────────────────");

    let less_code = r#"
.border-radius(@radius: 5px) {
    border-radius: @radius;
    -webkit-border-radius: @radius;
    -moz-border-radius: @radius;
}

.button {
    background: red;
    .border-radius(10px);
    padding: 10px;
}

.card {
    background: white;
    .border-radius();  // 使用默认值
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}
"#;

    print_test_input(less_code);
    execute_test(less_code);
}

fn test_default_parameters() {
    println!("\n📝 测试 2: 默认参数混合器");
    println!("─────────────────────────");

    let less_code = r#"
.box-shadow(@x: 0, @y: 0, @blur: 5px, @color: #000) {
    box-shadow: @x @y @blur @color;
    -webkit-box-shadow: @x @y @blur @color;
}

.card {
    .box-shadow(2px, 2px, 10px, #333);
}

.simple-card {
    .box-shadow();  // 全部使用默认值
}

.custom-card {
    .box-shadow(0, 4px);  // 部分使用默认值
}
"#;

    print_test_input(less_code);
    execute_test(less_code);
}

fn test_guarded_mixins() {
    println!("\n📝 测试 3: 守卫条件混合器");
    println!("─────────────────────────");

    let less_code = r#"
.responsive(@size) when (@size > 768px) {
    width: 80%;
    max-width: 1200px;
    margin: 0 auto;
}

.responsive(@size) when (@size <= 768px) {
    width: 100%;
    max-width: none;
    margin: 0;
}

.container-large {
    .responsive(1024px);
}

.container-mobile {
    .responsive(480px);
}
"#;

    print_test_input(less_code);
    execute_test(less_code);
}

fn test_nested_rules_in_mixins() {
    println!("\n📝 测试 4: 混合器中的嵌套规则");
    println!("───────────────────────────────");

    let less_code = r#"
.button-style(@bg-color: #333, @text-color: white) {
    background: @bg-color;
    color: @text-color;
    padding: 10px 20px;
    border: none;
    border-radius: 4px;

    &:hover {
        background: lighten(@bg-color, 10%);
        cursor: pointer;
    }

    &:active {
        background: darken(@bg-color, 10%);
    }

    &:disabled {
        background: #ccc;
        cursor: not-allowed;
    }
}

.primary-btn {
    .button-style(#007bff, white);
}

.danger-btn {
    .button-style(#dc3545);
}

.success-btn {
    .button-style(#28a745, white);
}
"#;

    print_test_input(less_code);
    execute_test(less_code);
}

fn test_error_handling() {
    println!("\n📝 测试 5: 错误处理");
    println!("─────────────────");

    // 测试 5a: 未定义的混合器
    println!("\n5a. 未定义的混合器:");
    let undefined_mixin = r#"
.test {
    .undefined-mixin(10px);
    background: red;
}
"#;

    print_test_input(undefined_mixin);
    match compile(undefined_mixin) {
        Ok(css) => {
            println!("⚠️  意外成功:");
            println!("{}", css);
        }
        Err(e) => {
            println!("✅ 正确捕获错误: {}", e);
        }
    }

    // 测试 5b: 参数数量不匹配
    println!("\n5b. 参数数量不匹配:");
    let param_mismatch = r#"
.three-params(@a, @b, @c) {
    margin: @a @b @c;
}

.test {
    .three-params(10px);  // 只传递了 1 个参数，期望 3 个
}
"#;

    print_test_input(param_mismatch);
    match compile(param_mismatch) {
        Ok(css) => {
            println!("⚠️  意外成功:");
            println!("{}", css);
        }
        Err(e) => {
            println!("✅ 正确捕获错误: {}", e);
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
