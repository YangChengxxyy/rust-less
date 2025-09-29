use rust_less::compile;

fn main() {
    println!("🔍 最简单的混合器测试");
    println!("===================");

    // 测试1: 最基础的混合器调用
    test_simplest_mixin();

    // 测试2: 逐步增加复杂度
    test_step_by_step();

    // 测试3: 调试混合器调用语法
    test_mixin_call_syntax();
}

fn test_simplest_mixin() {
    println!("\n🧪 测试1: 最简单的混合器");

    let less_code = r#"
.test {
    color: red;
}

.use {
    .test;
}
"#;

    println!("LESS代码:");
    println!("{}", less_code);

    match compile(less_code) {
        Ok(css) => {
            println!("✅ 编译成功");
            println!("CSS输出:");
            println!("{}", css);
        }
        Err(e) => {
            println!("❌ 编译失败: {}", e);
        }
    }
}

fn test_step_by_step() {
    println!("\n🧪 测试2: 逐步增加复杂度");

    // 步骤1: 只有混合器定义
    let step1 = r#"
.test {
    color: red;
}
"#;

    println!("\n步骤1 - 只有混合器定义:");
    match compile(step1) {
        Ok(css) => println!("✅ 成功: {}", css.trim()),
        Err(e) => println!("❌ 失败: {}", e),
    }

    // 步骤2: 空的规则
    let step2 = r#"
.use {
}
"#;

    println!("\n步骤2 - 空规则:");
    match compile(step2) {
        Ok(css) => println!("✅ 成功: {}", css.trim()),
        Err(e) => println!("❌ 失败: {}", e),
    }

    // 步骤3: 规则内单个声明
    let step3 = r#"
.use {
    color: blue;
}
"#;

    println!("\n步骤3 - 单个声明:");
    match compile(step3) {
        Ok(css) => println!("✅ 成功: {}", css.trim()),
        Err(e) => println!("❌ 失败: {}", e),
    }

    // 步骤4: 只有混合器调用
    let step4 = r#"
.use {
    .test;
}
"#;

    println!("\n步骤4 - 只有混合器调用:");
    match compile(step4) {
        Ok(css) => println!("✅ 成功: {}", css.trim()),
        Err(e) => println!("❌ 失败: {}", e),
    }
}

fn test_mixin_call_syntax() {
    println!("\n🧪 测试3: 混合器调用语法变体");

    let variants = vec![
        (".test;", "带分号"),
        (".test", "不带分号"),
        (".test();", "空括号+分号"),
        (".test()", "空括号不带分号"),
    ];

    for (syntax, desc) in variants {
        println!("\n测试语法 - {}:", desc);

        let full_code = format!(
            r#"
.use {{
    {}
}}
"#,
            syntax
        );

        match compile(&full_code) {
            Ok(css) => println!("✅ 成功: {}", css.trim()),
            Err(e) => println!("❌ 失败: {}", e),
        }
    }

    // 测试在声明前后的混合器调用
    println!("\n测试混合器调用位置:");

    let position_tests = vec![
        (
            r#"
.use {
    .test;
    color: blue;
}
"#,
            "混合器在前",
        ),
        (
            r#"
.use {
    color: blue;
    .test;
}
"#,
            "混合器在后",
        ),
        (
            r#"
.use {
    color: blue;
    .test;
    background: white;
}
"#,
            "混合器在中间",
        ),
    ];

    for (code, desc) in position_tests {
        println!("\n{}: ", desc);
        match compile(code) {
            Ok(css) => println!("✅ 成功: {}", css.trim()),
            Err(e) => println!("❌ 失败: {}", e),
        }
    }
}
