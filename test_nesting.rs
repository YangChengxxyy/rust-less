use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🧪 测试嵌套文件场景");
    println!("==================");

    // 测试1: 基本选择器嵌套
    test_basic_nesting()?;

    // 测试2: 媒体查询嵌套
    test_media_query_nesting()?;

    // 测试3: 多层嵌套
    test_deep_nesting()?;

    // 测试4: 父选择器引用
    test_parent_selector()?;

    // 测试5: 混合嵌套场景
    test_mixed_nesting()?;

    println!("\n✅ 所有嵌套测试完成！");
    Ok(())
}

fn test_basic_nesting() -> Result<(), Box<dyn Error>> {
    println!("\n🔸 测试1: 基本选择器嵌套");

    let less_code = r#"
.navbar {
    background: #333;
    height: 60px;

    .logo {
        float: left;
        width: 120px;
    }

    .menu {
        float: right;

        .item {
            display: inline-block;
            padding: 10px;

            .link {
                color: white;
                text-decoration: none;
            }
        }
    }
}
"#;

    let css = rust_less::compile(less_code)?;

    println!("输入 LESS:");
    println!("{}", less_code);
    println!("输出 CSS:");
    println!("{}", css);

    // 验证嵌套是否正确展开
    let checks = [
        (".navbar", "基本选择器"),
        (".navbar .logo", "一级嵌套"),
        (".navbar .menu", "一级嵌套"),
        (".navbar .menu .item", "二级嵌套"),
        (".navbar .menu .item .link", "三级嵌套"),
    ];

    for (selector, desc) in checks {
        if css.contains(selector) {
            println!("✅ {}: {}", desc, selector);
        } else {
            println!("❌ {}: {} 未找到", desc, selector);
        }
    }

    Ok(())
}

fn test_media_query_nesting() -> Result<(), Box<dyn Error>> {
    println!("\n🔸 测试2: 媒体查询嵌套");

    let less_code = r#"
.container {
    width: 1200px;
    margin: 0 auto;

    @media (max-width: 768px) {
        width: 100%;
        padding: 20px;

        .content {
            font-size: 14px;
        }
    }

    .sidebar {
        width: 300px;

        @media (max-width: 768px) {
            width: 100%;
            margin-top: 20px;
        }
    }
}
"#;

    let css = rust_less::compile(less_code)?;

    println!("输入 LESS:");
    println!("{}", less_code);
    println!("输出 CSS:");
    println!("{}", css);

    // 验证媒体查询和嵌套
    let checks = [
        ("@media (max-width: 768px)", "媒体查询"),
        (".container", "基本容器"),
        (".container .content", "嵌套内容"),
        (".container .sidebar", "嵌套边栏"),
    ];

    for (pattern, desc) in checks {
        if css.contains(pattern) {
            println!("✅ {}: 找到 {}", desc, pattern);
        } else {
            println!("❌ {}: {} 未找到", desc, pattern);
        }
    }

    Ok(())
}

fn test_deep_nesting() -> Result<(), Box<dyn Error>> {
    println!("\n🔸 测试3: 多层深度嵌套");

    let less_code = r#"
.layout {
    .header {
        .navigation {
            .menu {
                .item {
                    .link {
                        color: blue;

                        .icon {
                            margin-right: 5px;
                        }
                    }
                }
            }
        }
    }
}
"#;

    let css = rust_less::compile(less_code)?;

    println!("输入 LESS:");
    println!("{}", less_code);
    println!("输出 CSS:");
    println!("{}", css);

    // 验证深层嵌套
    let deep_selector = ".layout .header .navigation .menu .item .link";
    let deepest_selector = ".layout .header .navigation .menu .item .link .icon";

    if css.contains(deep_selector) {
        println!("✅ 深层嵌套: {}", deep_selector);
    } else {
        println!("❌ 深层嵌套失败: {}", deep_selector);
    }

    if css.contains(deepest_selector) {
        println!("✅ 最深嵌套: {}", deepest_selector);
    } else {
        println!("❌ 最深嵌套失败: {}", deepest_selector);
    }

    Ok(())
}

fn test_parent_selector() -> Result<(), Box<dyn Error>> {
    println!("\n🔸 测试4: 父选择器引用 (&)");

    let less_code = r#"
.button {
    background: blue;
    color: white;

    &:hover {
        background: darkblue;
    }

    &.active {
        background: green;
    }

    &.large {
        font-size: 18px;

        &:hover {
            background: darkgreen;
        }
    }
}
"#;

    let css = rust_less::compile(less_code)?;

    println!("输入 LESS:");
    println!("{}", less_code);
    println!("输出 CSS:");
    println!("{}", css);

    // 验证父选择器引用
    let checks = [
        (".button", "基本选择器"),
        (".button:hover", "伪类选择器"),
        (".button.active", "类修饰符"),
        (".button.large", "类修饰符"),
        (".button.large:hover", "嵌套父选择器"),
    ];

    for (selector, desc) in checks {
        if css.contains(selector) {
            println!("✅ {}: {}", desc, selector);
        } else {
            println!("❌ {}: {} 未找到", desc, selector);
        }
    }

    Ok(())
}

fn test_mixed_nesting() -> Result<(), Box<dyn Error>> {
    println!("\n🔸 测试5: 混合嵌套场景");

    let less_code = r#"
.card {
    border: 1px solid #ddd;
    border-radius: 4px;

    .header {
        padding: 15px;
        background: #f5f5f5;

        &:hover {
            background: #eeeeee;
        }

        .title {
            font-size: 18px;
            margin: 0;
        }
    }

    .content {
        padding: 15px;

        p {
            margin-bottom: 10px;

            &:last-child {
                margin-bottom: 0;
            }
        }

        @media (max-width: 600px) {
            padding: 10px;

            p {
                font-size: 14px;
            }
        }
    }

    &.highlighted {
        border-color: #007bff;

        .header {
            background: #e3f2fd;
        }
    }
}
"#;

    let css = rust_less::compile(less_code)?;

    println!("输入 LESS:");
    println!("{}", less_code);
    println!("输出 CSS:");
    println!("{}", css);

    // 验证混合场景
    let checks = [
        (".card", "基本卡片"),
        (".card .header", "卡片头部"),
        (".card .header:hover", "头部悬停"),
        (".card .header .title", "标题嵌套"),
        (".card .content", "内容区域"),
        (".card .content p", "段落嵌套"),
        (".card .content p:last-child", "伪类嵌套"),
        (".card.highlighted", "修饰符类"),
        (".card.highlighted .header", "修饰符下的嵌套"),
        ("@media (max-width: 600px)", "媒体查询"),
    ];

    println!("验证结果:");
    for (pattern, desc) in checks {
        if css.contains(pattern) {
            println!("✅ {}: {}", desc, pattern);
        } else {
            println!("❌ {}: {} 未找到", desc, pattern);
        }
    }

    // 统计结果
    let found_count = checks
        .iter()
        .filter(|(pattern, _)| css.contains(pattern))
        .count();
    let total_count = checks.len();

    println!(
        "\n📊 总体结果: {}/{} 通过 ({:.1}%)",
        found_count,
        total_count,
        (found_count as f64 / total_count as f64) * 100.0
    );

    Ok(())
}
