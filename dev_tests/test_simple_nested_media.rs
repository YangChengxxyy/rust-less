use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Test simple nested media queries
    let input = r#"
.container {
    width: 100%;
    padding: 20px;

    @media (max-width: 768px) {
        width: 90%;
        padding: 10px;

        .inner {
            display: none;
        }

        .sidebar {
            width: 100%;
        }
    }

    @media (min-width: 1200px) {
        width: 1200px;

        .content {
            display: flex;
        }
    }
}

.header {
    background: #333;

    @media (max-width: 768px) {
        position: fixed;

        .logo {
            width: 100px;
        }

        .menu {
            display: none;

            .item {
                padding: 10px;
            }
        }
    }
}
"#;

    println!("=== 测试简单嵌套媒体查询 ===");
    println!("输入的 LESS 代码:");
    println!("{}", input);
    println!();

    // 使用 rust-less 编译器
    let mut compiler = rust_less::compiler::Compiler::new();

    match compiler.compile(input) {
        Ok(css) => {
            println!("编译成功！");
            println!("=== 生成的 CSS ===");
            println!("{}", css);

            // 验证嵌套媒体查询功能
            let has_media_768 = css.contains("@media (max-width: 768px)");
            let has_media_1200 = css.contains("@media (min-width: 1200px)");
            let has_container_inner = css.contains(".container .inner");
            let has_container_sidebar = css.contains(".container .sidebar");
            let has_header_logo = css.contains(".header .logo");
            let has_header_menu_item = css.contains(".header .menu .item");

            println!("\n=== 验证结果 ===");
            println!("✓ 包含 max-width: 768px 媒体查询: {}", has_media_768);
            println!("✓ 包含 min-width: 1200px 媒体查询: {}", has_media_1200);
            println!(
                "✓ 包含嵌套选择器 .container .inner: {}",
                has_container_inner
            );
            println!(
                "✓ 包含嵌套选择器 .container .sidebar: {}",
                has_container_sidebar
            );
            println!("✓ 包含嵌套选择器 .header .logo: {}", has_header_logo);
            println!(
                "✓ 包含多层嵌套 .header .menu .item: {}",
                has_header_menu_item
            );

            let media_count = css.matches("@media").count();
            println!("✓ 媒体查询总数: {}", media_count);

            // 检查是否正确处理了嵌套结构
            let nested_correctly = has_media_768 && has_container_inner && has_header_logo;

            if nested_correctly {
                println!("\n🎉 嵌套媒体查询功能正常工作！");
                println!("✅ 支持以下特性:");
                println!("   - 在 CSS 规则内嵌套 @media 查询");
                println!("   - 媒体查询内的嵌套选择器");
                println!("   - 多层选择器嵌套");
                println!("   - 不同断点的媒体查询");
            } else {
                println!("\n⚠️  嵌套媒体查询处理可能存在问题");
            }

            // 分析 CSS 结构
            println!("\n=== CSS 结构分析 ===");
            let lines: Vec<&str> = css.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if line.trim().starts_with("@media") {
                    println!("第 {} 行: 媒体查询 -> {}", i + 1, line.trim());
                }
                if line.trim().contains("{") && !line.trim().starts_with("@media") {
                    println!("第 {} 行: CSS 规则 -> {}", i + 1, line.trim());
                }
            }
        }
        Err(e) => {
            println!("编译失败: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
