use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Test nested media queries
    let input = r#"
.container {
    width: 100%;
    padding: 20px;

    @media (max-width: 768px) {
        width: 90%;
        padding: 10px;

        .inner {
            display: none;
            font-size: 14px;
        }

        .sidebar {
            width: 100%;
            margin-top: 20px;
        }
    }

    @media (min-width: 1200px) {
        width: 1200px;
        margin: 0 auto;

        .content {
            display: flex;
            gap: 30px;
        }
    }
}

.navigation {
    background: #333;

    @media (max-width: 768px) {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;

        .menu-item {
            display: block;
            padding: 15px;

            &:hover {
                background: #555;
            }
        }
    }
}
"#;

    println!("=== 测试嵌套媒体查询 ===");
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

            // 验证是否包含预期的媒体查询
            let has_media_768 = css.contains("@media (max-width: 768px)");
            let has_media_1200 = css.contains("@media (min-width: 1200px)");
            let has_nested_selectors =
                css.contains(".container .inner") || css.contains(".container .sidebar");

            println!("\n=== 验证结果 ===");
            println!("✓ 包含 max-width: 768px 媒体查询: {}", has_media_768);
            println!("✓ 包含 min-width: 1200px 媒体查询: {}", has_media_1200);
            println!("✓ 包含嵌套选择器: {}", has_nested_selectors);

            if has_media_768 && has_media_1200 {
                println!("\n🎉 嵌套媒体查询功能正常工作！");
            } else {
                println!("\n⚠️  媒体查询处理可能存在问题");
            }
        }
        Err(e) => {
            println!("编译失败: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
