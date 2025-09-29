use rust_less::compile;

fn main() {
    println!("🔍 混合器定义语法测试");
    println!("======================");

    // 测试1: 无括号 - 应该是CSS规则，不是混合器定义
    test_css_rule_vs_mixin_definition();

    // 测试2: 带空括号 - 应该是混合器定义
    test_empty_parentheses_mixin();

    // 测试3: 带参数 - 应该是混合器定义
    test_parameterized_mixin();

    // 测试4: 完整的混合器定义+调用测试
    test_complete_mixin_workflow();
}

fn test_css_rule_vs_mixin_definition() {
    println!("\n🧪 测试1: CSS规则 vs 混合器定义");

    // 这应该被当作CSS规则，不是混合器定义
    let css_rule = r#"
.test {
    color: red;
}
"#;

    println!("无括号语法（CSS规则）:");
    match compile(css_rule) {
        Ok(css) => {
            println!("✅ 编译成功 - 这是CSS规则");
            println!("CSS输出: {}", css.trim());
        }
        Err(e) => {
            println!("❌ 编译失败: {}", e);
        }
    }

    // 尝试调用这个"CSS规则"作为混合器
    let call_css_rule = r#"
.test {
    color: red;
}

.use {
    .test;
}
"#;

    println!("\n尝试调用CSS规则作为混合器:");
    match compile(call_css_rule) {
        Ok(css) => {
            println!("⚠️ 意外成功: {}", css.trim());
        }
        Err(e) => {
            println!("✅ 预期失败（CSS规则不能作为混合器调用）: {}", e);
        }
    }
}

fn test_empty_parentheses_mixin() {
    println!("\n🧪 测试2: 空括号混合器定义");

    let empty_paren_mixin = r#"
.test() {
    color: red;
    background: blue;
}

.use {
    .test();
}
"#;

    println!("空括号语法（应该是混合器定义）:");
    match compile(empty_paren_mixin) {
        Ok(css) => {
            println!("✅ 编译成功");
            println!("CSS输出:");
            println!("{}", css);

            if css.contains(".use") && css.contains("color: red") {
                println!("✅ 混合器正确展开");
            } else {
                println!("❌ 混合器展开失败");
            }
        }
        Err(e) => {
            println!("❌ 编译失败: {}", e);
        }
    }
}

fn test_parameterized_mixin() {
    println!("\n🧪 测试3: 参数化混合器定义");

    let param_mixin = r#"
.border-radius(@radius) {
    border-radius: @radius;
    -webkit-border-radius: @radius;
}

.button {
    .border-radius(10px);
    background: #007cba;
}
"#;

    println!("参数化语法（应该是混合器定义）:");
    match compile(param_mixin) {
        Ok(css) => {
            println!("✅ 编译成功");
            println!("CSS输出:");
            println!("{}", css);

            if css.contains(".button") && css.contains("border-radius: 10px") {
                println!("✅ 参数化混合器正确展开");
            } else {
                println!("❌ 参数化混合器展开失败");
            }
        }
        Err(e) => {
            println!("❌ 编译失败: {}", e);
        }
    }
}

fn test_complete_mixin_workflow() {
    println!("\n🧪 测试4: 完整混合器工作流");

    let complete_workflow = r#"
// 混合器定义1: 无参数
.reset() {
    margin: 0;
    padding: 0;
}

// 混合器定义2: 单参数
.border(@width) {
    border: @width solid #ccc;
}

// 混合器定义3: 多参数
.box-shadow(@x, @y, @blur) {
    box-shadow: @x @y @blur rgba(0,0,0,0.1);
}

// CSS规则（不是混合器）
.base-style {
    font-family: Arial, sans-serif;
}

// 使用混合器的规则
.card {
    .reset();
    .border(1px);
    .box-shadow(0, 2px, 4px);
    background: white;
}

.button {
    .reset();
    .border(2px);
    padding: 10px 20px;
    background: #007cba;
}
"#;

    println!("完整工作流测试:");
    match compile(complete_workflow) {
        Ok(css) => {
            println!("✅ 编译成功");
            println!("CSS输出:");
            println!("{}", css);

            // 验证各种功能
            let checks = vec![
                (css.contains(".base-style"), "CSS规则正常输出"),
                (css.contains(".card"), "card规则存在"),
                (css.contains(".button"), "button规则存在"),
                (css.contains("margin: 0"), "reset混合器展开"),
                (css.contains("border: 1px solid #ccc"), "单参数混合器展开"),
                (css.contains("border: 2px solid #ccc"), "不同参数值展开"),
                (css.contains("box-shadow: 0 2px 4px"), "多参数混合器展开"),
            ];

            for (check, desc) in checks {
                if check {
                    println!("✅ {}", desc);
                } else {
                    println!("❌ {}", desc);
                }
            }
        }
        Err(e) => {
            println!("❌ 编译失败: {}", e);
        }
    }
}
