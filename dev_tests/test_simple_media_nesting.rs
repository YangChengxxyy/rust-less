use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Test simple media queries nested inside other media queries
    let input = r#"
@media (min-width: 768px) {
    .container {
        width: 100%;

        @media (max-width: 1200px) {
            width: 80%;
        }
    }
}

@media screen {
    .header {
        background: #fff;

        @media (max-width: 480px) {
            background: #000;
        }
    }
}

@media (orientation: landscape) {
    .sidebar {
        display: block;

        @media (min-width: 1024px) {
            width: 300px;
        }

        @media (max-width: 600px) {
            display: none;
        }
    }
}
"#;

    println!("=== 测试媒体查询嵌套媒体查询（简化版）===");
    println!("输入的 LESS 代码:");
    println!("{}", input);
    println!();

    // 使用 rust-less 编译器
    let mut compiler = rust_less::compiler::Compiler::new();

    match compiler.compile(input) {
        Ok(css) => {
            println!("✅ 编译成功！");
            println!("=== 生成的 CSS ===");
            println!("{}", css);

            // 检查是否有嵌套媒体查询的组合
            let expected_combinations = vec![
                "@media (min-width: 768px) and (max-width: 1200px)",
                "@media screen and (max-width: 480px)",
                "@media (orientation: landscape) and (min-width: 1024px)",
                "@media (orientation: landscape) and (max-width: 600px)",
            ];

            println!("\n=== 验证嵌套媒体查询合并 ===");
            let mut found_combinations = 0;

            for combination in &expected_combinations {
                if css.contains(combination) {
                    println!("✅ 找到合并的媒体查询: {}", combination);
                    found_combinations += 1;
                } else {
                    println!("❌ 未找到合并的媒体查询: {}", combination);
                }
            }

            // 分析媒体查询结构
            let total_media = css.matches("@media").count();
            let separate_media = css
                .lines()
                .filter(|line| line.trim().starts_with("@media"))
                .count();

            println!("\n📊 媒体查询分析:");
            println!("- 总 @media 出现次数: {}", total_media);
            println!("- 独立媒体查询数量: {}", separate_media);
            println!(
                "- 找到的合并查询: {}/{}",
                found_combinations,
                expected_combinations.len()
            );

            if found_combinations > 0 {
                println!("\n🎉 媒体查询嵌套功能部分工作！");
                println!("✨ 编译器能够处理嵌套的媒体查询");

                if found_combinations == expected_combinations.len() {
                    println!("✅ 完美！所有嵌套媒体查询都被正确合并");
                } else {
                    println!("⚠️  部分嵌套媒体查询需要改进");
                }
            } else {
                println!("\n🤔 媒体查询嵌套可能不被支持");
                println!("💡 编译器可能采用以下策略之一:");
                println!("   1. 将嵌套媒体查询分离为独立的媒体查询块");
                println!("   2. 忽略内层媒体查询");
                println!("   3. 按顺序输出所有媒体查询");
            }

            // 显示实际的媒体查询结构
            println!("\n=== 实际生成的媒体查询 ===");
            for line in css.lines() {
                if line.trim().starts_with("@media") {
                    println!("📱 {}", line.trim());
                }
            }
        }
        Err(e) => {
            println!("❌ 编译失败: {}", e);
            println!("\n原因分析:");
            println!("🔍 媒体查询嵌套功能可能:");
            println!("   - 尚未在解析器中实现");
            println!("   - 不被当前版本支持");
            println!("   - 需要特殊的语法处理");

            return Err(e.into());
        }
    }

    Ok(())
}
