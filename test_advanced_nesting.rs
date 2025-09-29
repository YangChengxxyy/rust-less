use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 测试高级嵌套场景");
    println!("====================");

    // 测试1: 复杂媒体查询嵌套合并
    test_complex_media_nesting()?;

    // 测试2: 嵌套中的变量作用域
    test_nested_variable_scope()?;

    // 测试3: 多重父选择器引用
    test_multiple_parent_selectors()?;

    // 测试4: 深层媒体查询嵌套
    test_deep_media_nesting()?;

    // 测试5: 混合选择器类型嵌套
    test_mixed_selector_types()?;

    // 测试6: 边界情况测试
    test_edge_cases()?;

    println!("\n🎯 高级嵌套测试总结完成！");
    Ok(())
}

fn test_complex_media_nesting() -> Result<(), Box<dyn Error>> {
    println!("\n🔸 测试1: 复杂媒体查询嵌套合并");

    let less_code = r#"
.responsive-layout {
    display: grid;
    grid-template-columns: 1fr 300px;

    @media (max-width: 1024px) {
        grid-template-columns: 1fr;

        .sidebar {
            order: 2;

            @media (max-width: 768px) {
                display: none;

                @media (orientation: landscape) {
                    display: block;
                    position: fixed;
                    top: 0;
                    left: 0;
                }
            }
        }

        .main-content {
            order: 1;

            @media (max-width: 768px) {
                padding: 10px;

                @media (max-width: 480px) {
                    padding: 5px;
                    font-size: 14px;
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

    // 验证媒体查询合并
    let expected_patterns = [
        ("@media (max-width: 1024px)", "第一层媒体查询"),
        ("@media (max-width: 768px)", "第二层媒体查询"),
        ("@media (max-width: 480px)", "第三层媒体查询"),
        ("@media (orientation: landscape)", "方向媒体查询"),
        (".responsive-layout .sidebar", "嵌套侧边栏"),
        (".responsive-layout .main-content", "嵌套主内容"),
    ];

    println!("\n验证媒体查询嵌套:");
    for (pattern, desc) in expected_patterns {
        if css.contains(pattern) {
            println!("✅ {}: {}", desc, pattern);
        } else {
            println!("❌ {}: {} 未找到", desc, pattern);
        }
    }

    // 检查是否有合并的媒体查询
    let media_count = css.matches("@media").count();
    println!("📊 媒体查询总数: {}", media_count);

    Ok(())
}

fn test_nested_variable_scope() -> Result<(), Box<dyn Error>> {
    println!("\n🔸 测试2: 嵌套中的变量作用域");

    let less_code = r#"
@primary-color: #333;
@spacing: 20px;

.container {
    @local-color: #666;
    @spacing: 30px; // 局部覆盖全局变量

    background: @primary-color;
    padding: @spacing;

    .header {
        @header-color: lighten(@local-color, 20%);

        color: @header-color;
        margin-bottom: @spacing;

        .title {
            @title-size: 24px;

            font-size: @title-size;
            color: @local-color;
        }
    }

    .content {
        // 使用外层变量
        color: @local-color;
        padding: @spacing / 2;

        .text {
            // 应该能访问所有上层变量
            line-height: @spacing;
        }
    }
}

.external {
    // 不应该能访问.container内的局部变量
    padding: @spacing; // 应该使用全局的20px
}
"#;

    let css = rust_less::compile(less_code)?;

    println!("输入 LESS:");
    println!("{}", less_code);
    println!("输出 CSS:");
    println!("{}", css);

    // 验证变量作用域
    let checks = [
        ("padding: 30px", "局部变量覆盖全局变量"),
        ("padding: 20px", "全局变量在外部使用"),
        ("margin-bottom: 30px", "嵌套中使用父级变量"),
        ("padding: 15px", "变量运算在嵌套中"),
        ("line-height: 30px", "深层嵌套访问变量"),
    ];

    println!("\n验证变量作用域:");
    for (pattern, desc) in checks {
        if css.contains(pattern) {
            println!("✅ {}: {}", desc, pattern);
        } else {
            println!("❌ {}: {} 未找到", desc, pattern);
        }
    }

    Ok(())
}

fn test_multiple_parent_selectors() -> Result<(), Box<dyn Error>> {
    println!("\n🔸 测试3: 多重父选择器引用");

    let less_code = r#"
.btn {
    display: inline-block;
    padding: 10px 20px;
    border: none;
    cursor: pointer;

    // 基本状态
    &:hover {
        opacity: 0.8;
    }

    &:active {
        transform: translateY(1px);
    }

    &:focus {
        outline: 2px solid blue;
    }

    // 修饰符类
    &.primary {
        background: #007bff;
        color: white;

        &:hover {
            background: #0056b3;
        }

        &:disabled {
            background: #6c757d;
            cursor: not-allowed;
        }
    }

    &.secondary {
        background: #6c757d;
        color: white;

        &:hover {
            background: #545b62;
        }
    }

    &.large {
        padding: 15px 30px;
        font-size: 18px;

        &.primary {
            &:hover {
                background: #004085;
            }
        }
    }

    // 组合选择器
    &.btn-group & {
        margin-right: 5px;

        &:last-child {
            margin-right: 0;
        }
    }

    // 兄弟选择器
    & + & {
        margin-left: 10px;
    }
}
"#;

    let css = rust_less::compile(less_code)?;

    println!("输入 LESS:");
    println!("{}", less_code);
    println!("输出 CSS:");
    println!("{}", css);

    // 验证复杂的父选择器引用
    let checks = [
        (".btn:hover", "基本伪类"),
        (".btn.primary", "修饰符类"),
        (".btn.primary:hover", "修饰符伪类"),
        (".btn.large.primary:hover", "多重修饰符伪类"),
        (".btn + .btn", "兄弟选择器"),
        (".btn-group .btn", "后代选择器"),
        (".btn-group .btn:last-child", "复合选择器"),
    ];

    println!("\n验证父选择器引用:");
    for (pattern, desc) in checks {
        if css.contains(pattern) {
            println!("✅ {}: {}", desc, pattern);
        } else {
            println!("❌ {}: {} 未找到", desc, pattern);
        }
    }

    Ok(())
}

fn test_deep_media_nesting() -> Result<(), Box<dyn Error>> {
    println!("\n🔸 测试4: 深层媒体查询嵌套");

    let less_code = r#"
.dashboard {
    display: grid;
    grid-template: "header header" auto
                   "sidebar content" 1fr / 250px 1fr;

    @media (max-width: 1200px) {
        grid-template: "header" auto
                       "content" 1fr
                       "sidebar" auto / 1fr;

        .sidebar {
            order: 3;

            @media (max-width: 768px) {
                position: fixed;
                top: 0;
                left: -250px;
                transition: left 0.3s;

                &.open {
                    left: 0;

                    @media (max-width: 480px) {
                        width: 100%;
                        left: -100%;

                        &.open {
                            left: 0;
                        }
                    }
                }

                .nav-item {
                    padding: 15px;

                    @media (max-width: 480px) {
                        padding: 10px;
                        font-size: 14px;

                        &:hover {
                            background: #f0f0f0;

                            @media (prefers-color-scheme: dark) {
                                background: #333;
                            }
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

    // 验证深层媒体查询嵌套
    println!("\n验证深层媒体查询:");
    let media_queries = [
        "@media (max-width: 1200px)",
        "@media (max-width: 768px)",
        "@media (max-width: 480px)",
        "@media (prefers-color-scheme: dark)",
    ];

    for query in media_queries {
        let count = css.matches(query).count();
        println!("📱 {}: {} 次", query, count);
    }

    // 验证选择器嵌套
    let nested_selectors = [
        ".dashboard .sidebar",
        ".dashboard .sidebar.open",
        ".dashboard .sidebar .nav-item",
        ".dashboard .sidebar .nav-item:hover",
    ];

    println!("\n验证选择器嵌套:");
    for selector in nested_selectors {
        if css.contains(selector) {
            println!("✅ 找到: {}", selector);
        } else {
            println!("❌ 未找到: {}", selector);
        }
    }

    Ok(())
}

fn test_mixed_selector_types() -> Result<(), Box<dyn Error>> {
    println!("\n🔸 测试5: 混合选择器类型嵌套");

    let less_code = r#"
// ID选择器嵌套
#app {
    .header {
        background: #fff;

        // 属性选择器
        input[type="search"] {
            border: 1px solid #ccc;

            &:focus {
                border-color: #007bff;
            }

            &[disabled] {
                background: #f5f5f5;
            }
        }

        // 伪元素
        .logo {
            &::before {
                content: "";
                display: inline-block;
                width: 20px;
                height: 20px;
                background: url("logo.png");
            }

            &::after {
                content: "®";
                font-size: 10px;
                vertical-align: super;
            }
        }
    }

    // 通用选择器
    * {
        box-sizing: border-box;

        &:focus {
            outline: none;
        }
    }

    // 元素选择器
    h1, h2, h3 {
        font-family: "Arial", sans-serif;
        margin: 0 0 1em 0;

        & + p {
            margin-top: 0.5em;
        }

        &:first-child {
            margin-top: 0;
        }
    }

    // 复杂选择器组合
    .content {
        > .section {
            margin-bottom: 2em;

            &:nth-child(odd) {
                background: #f9f9f9;

                > h2 {
                    color: #333;

                    ~ p {
                        color: #666;
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

    // 验证各种选择器类型
    let selector_types = [
        ("#app", "ID选择器"),
        ("#app .header", "ID + 类选择器"),
        ("#app .header input[type=\"search\"]", "属性选择器"),
        ("#app .header input[type=\"search\"]:focus", "属性 + 伪类"),
        ("#app .header .logo::before", "伪元素"),
        ("#app *", "通用选择器"),
        ("#app h1", "元素选择器"),
        ("#app h1 + p", "兄弟选择器"),
        ("#app .content > .section", "子选择器"),
        ("#app .content > .section:nth-child(odd)", "结构伪类"),
        (
            "#app .content > .section:nth-child(odd) > h2 ~ p",
            "复合选择器",
        ),
    ];

    println!("\n验证选择器类型:");
    for (selector, desc) in selector_types {
        if css.contains(selector) {
            println!("✅ {}: {}", desc, selector);
        } else {
            println!("❌ {}: {} 未找到", desc, selector);
        }
    }

    Ok(())
}

fn test_edge_cases() -> Result<(), Box<dyn Error>> {
    println!("\n🔸 测试6: 边界情况测试");

    let less_code = r#"
// 空规则嵌套
.empty-parent {
    .child {
        color: red;
    }
}

// 只有媒体查询的规则
.media-only {
    @media (max-width: 768px) {
        color: blue;
    }
}

// 深度嵌套到极限
.level1 {
    .level2 {
        .level3 {
            .level4 {
                .level5 {
                    .level6 {
                        .level7 {
                            .level8 {
                                .level9 {
                                    .level10 {
                                        color: deep;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// 复杂的父选择器引用
.complex-parent {
    &:not(.disabled) {
        &[data-active="true"] {
            &:hover:focus {
                &.highlighted {
                    background: yellow;
                }
            }
        }
    }
}

// 多个媒体查询在同一级别
.multi-media {
    color: black;

    @media (max-width: 768px) {
        color: red;
    }

    @media (min-width: 769px) and (max-width: 1024px) {
        color: green;
    }

    @media (min-width: 1025px) {
        color: blue;
    }

    @media print {
        color: black !important;
    }
}

// 嵌套中的注释和空白
.with-comments {
    /* 外层注释 */
    color: black;

    .nested {
        /* 嵌套注释 */
        background: white;

        /*
         * 多行注释
         * 测试
         */

        &:hover {
            // 行注释
            opacity: 0.8;
        }
    }
}
"#;

    let css = rust_less::compile(less_code)?;

    println!("输入 LESS:");
    println!("{}", less_code);
    println!("输出 CSS:");
    println!("{}", css);

    // 验证边界情况
    let edge_case_checks = [
        (".empty-parent .child", "空父规则处理"),
        ("@media (max-width: 768px)", "纯媒体查询规则"),
        (
            ".level1 .level2 .level3 .level4 .level5 .level6 .level7 .level8 .level9 .level10",
            "极深嵌套",
        ),
        (
            ".complex-parent:not(.disabled)[data-active=\"true\"]:hover:focus.highlighted",
            "复杂父选择器",
        ),
        (
            "@media (min-width: 769px) and (max-width: 1024px)",
            "复合媒体查询",
        ),
        ("@media print", "打印媒体查询"),
        (".with-comments .nested", "包含注释的嵌套"),
    ];

    println!("\n验证边界情况:");
    let mut passed = 0;
    let total = edge_case_checks.len();

    for (pattern, desc) in edge_case_checks {
        if css.contains(pattern) {
            println!("✅ {}: {}", desc, pattern);
            passed += 1;
        } else {
            println!("❌ {}: {} 未找到", desc, pattern);
        }
    }

    println!(
        "\n📊 边界情况测试结果: {}/{} 通过 ({:.1}%)",
        passed,
        total,
        (passed as f64 / total as f64) * 100.0
    );

    // 检查CSS结构完整性
    let open_braces = css.matches('{').count();
    let close_braces = css.matches('}').count();

    if open_braces == close_braces {
        println!("✅ CSS 大括号配对正确: {} 对", open_braces);
    } else {
        println!(
            "❌ CSS 大括号配对错误: {} 开括号, {} 闭括号",
            open_braces, close_braces
        );
    }

    Ok(())
}
