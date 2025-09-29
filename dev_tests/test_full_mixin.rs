use rust_less::compile;

fn main() {
    println!("🧪 测试完整混合器功能");
    println!("========================");

    // 测试1: 基础混合器（无参数）
    test_basic_mixin();

    // 测试2: 参数化混合器
    test_parameterized_mixin();

    // 测试3: 混合器嵌套调用
    test_nested_mixin_calls();

    // 测试4: 混合器默认参数
    test_mixin_with_defaults();
}

fn test_basic_mixin() {
    println!("\n🔧 测试1: 基础混合器（无参数）");

    let less_code = r#"
.border-radius {
    border-radius: 5px;
    -webkit-border-radius: 5px;
    -moz-border-radius: 5px;
}

.button {
    padding: 10px;
    background: #007cba;
    .border-radius;
}

.card {
    margin: 20px;
    .border-radius;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            println!("✅ 基础混合器编译成功");
            println!("CSS输出:");
            println!("{}", css);

            // 验证混合器是否被正确展开
            if css.contains(".button")
                && css.contains("border-radius: 5px")
                && css.contains(".card")
            {
                println!("✅ 混合器展开正确");
            } else {
                println!("❌ 混合器展开失败");
            }
        }
        Err(e) => {
            println!("❌ 基础混合器编译失败: {}", e);
        }
    }
}

fn test_parameterized_mixin() {
    println!("\n🔧 测试2: 参数化混合器");

    let less_code = r#"
.border-radius(@radius) {
    border-radius: @radius;
    -webkit-border-radius: @radius;
    -moz-border-radius: @radius;
}

.button {
    .border-radius(10px);
    background: #007cba;
}

.small-button {
    .border-radius(3px);
    background: #28a745;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            println!("✅ 参数化混合器编译成功");
            println!("CSS输出:");
            println!("{}", css);

            // 验证参数替换
            if css.contains("border-radius: 10px") && css.contains("border-radius: 3px") {
                println!("✅ 参数替换正确");
            } else {
                println!("❌ 参数替换失败");
            }
        }
        Err(e) => {
            println!("❌ 参数化混合器编译失败: {}", e);
        }
    }
}

fn test_nested_mixin_calls() {
    println!("\n🔧 测试3: 混合器嵌套调用");

    let less_code = r#"
.box-shadow(@blur) {
    box-shadow: 0 2px @blur rgba(0,0,0,0.1);
    -webkit-box-shadow: 0 2px @blur rgba(0,0,0,0.1);
}

.border-radius(@radius) {
    border-radius: @radius;
    -webkit-border-radius: @radius;
}

.card-style {
    .border-radius(8px);
    .box-shadow(4px);
    padding: 20px;
}

.product-card {
    .card-style;
    background: white;
    border: 1px solid #ddd;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            println!("✅ 嵌套混合器编译成功");
            println!("CSS输出:");
            println!("{}", css);

            // 验证嵌套展开
            if css.contains("border-radius: 8px") && css.contains("box-shadow: 0 2px 4px") {
                println!("✅ 嵌套混合器展开正确");
            } else {
                println!("❌ 嵌套混合器展开失败");
            }
        }
        Err(e) => {
            println!("❌ 嵌套混合器编译失败: {}", e);
        }
    }
}

fn test_mixin_with_defaults() {
    println!("\n🔧 测试4: 默认参数混合器");

    let less_code = r#"
.box-shadow(@x, @y, @blur) {
    box-shadow: @x @y @blur rgba(0,0,0,0.1);
}

.card {
    .box-shadow(0, 2px, 4px);
    background: white;
}

.elevated-card {
    .box-shadow(2px, 4px, 8px);
    background: white;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            println!("✅ 多参数混合器编译成功");
            println!("CSS输出:");
            println!("{}", css);

            // 验证多参数处理
            if css.contains("box-shadow: 0 2px 4px") && css.contains("box-shadow: 2px 4px 8px") {
                println!("✅ 多参数处理正确");
            } else {
                println!("❌ 多参数处理失败");
            }
        }
        Err(e) => {
            println!("❌ 多参数混合器编译失败: {}", e);
        }
    }
}
