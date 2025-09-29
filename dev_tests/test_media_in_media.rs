use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Test media queries nested inside other media queries
    let input = r#"
// 测试案例 1: 基本的媒体查询嵌套媒体查询
@media (min-width: 768px) {
    .container {
        width: 100%;
        padding: 20px;

        @media (max-width: 1200px) {
            width: 80%;
            padding: 15px;
        }

        @media (min-width: 1400px) {
            width: 1200px;
            margin: 0 auto;
        }
    }

    .sidebar {
        width: 300px;

        @media (max-width: 1024px) {
            width: 250px;
        }

        @media (orientation: portrait) {
            width: 100%;
            order: 2;
        }
    }
}

// 测试案例 2: 多层嵌套媒体查询
@media screen {

    .layout {
        display: grid;

        @media (min-width: 768px) {
            grid-template-columns: 1fr 300px;

            @media (max-width: 1200px) {
                grid-template-columns: 1fr 250px;

                @media (max-height: 600px) {
                    grid-template-rows: auto 1fr auto;
                }
            }
        }
    }
}

// 测试案例 3: 复杂的媒体查询组合
@media (min-width: 1024px) and (max-width: 1440px) {
    .content {
        max-width: 1200px;

        @media (min-resolution: 2dppx) {
            .image {
                transform: scale(0.5);
            }
        }

        @media (prefers-color-scheme: dark) {
            background: #333;
            color: white;

            @media (prefers-reduced-motion: reduce) {
                transition: none;
            }
        }
    }
}

// 测试案例 4: 嵌套媒体查询中的选择器嵌套
@media (max-width: 768px) {
    .header {
        position: fixed;

        @media (orientation: landscape) {
            height: 50px;

            .logo {
                width: 80px;

                @media (max-width: 480px) {
                    width: 60px;
                }
            }

            .nav {
                display: none;

                &.mobile {
                    display: block;

                    @media (max-height: 400px) {
                        font-size: 14px;
                    }
                }
            }
        }
    }
}
"#;

    println!("=== 测试媒体查询嵌套媒体查询 ===");
    println!("输入的 LESS 代码（媒体查询内嵌套其他媒体查询）:");
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
            let nested_media_patterns = vec![
                "@media (min-width: 768px) and (max-width: 1200px)",
                "@media (min-width: 768px) and (min-width: 1400px)",
                "@media (min-width: 768px) and (max-width: 1024px)",
                "@media (min-width: 768px) and (orientation: portrait)",
                "@media screen and (min-width: 768px)",
                "@media screen and (min-width: 768px) and (max-width: 1200px)",
                "@media screen and (min-width: 768px) and (max-width: 1200px) and (max-height: 600px)",
                "@media (min-width: 1024px) and (max-width: 1440px) and (min-resolution: 2dppx)",
                "@media (min-width: 1024px) and (max-width: 1440px) and (prefers-color-scheme: dark)",
                "@media (max-width: 768px) and (orientation: landscape)",
                "@media (max-width: 768px) and (orientation: landscape) and (max-width: 480px)",
                "@media (max-width: 768px) and (orientation: landscape) and (max-height: 400px)",
            ];

            println!("\n=== 验证嵌套媒体查询结果 ===");

            let mut found_nested = 0;
            for pattern in &nested_media_patterns {
                if css.contains(pattern) {
                    println!("✅ 找到组合媒体查询: {}", pattern);
                    found_nested += 1;
                } else {
                    println!("❌ 未找到组合媒体查询: {}", pattern);
                }
            }

            // 统计媒体查询总数
            let total_media_queries = css.matches("@media").count();
            println!("\n📊 统计信息:");
            println!("总媒体查询数量: {}", total_media_queries);
            println!(
                "找到的嵌套媒体查询: {}/{}",
                found_nested,
                nested_media_patterns.len()
            );

            // 检查是否正确处理了媒体查询合并
            if found_nested > 0 {
                println!("\n🎉 部分媒体查询嵌套功能正常工作！");

                if found_nested >= nested_media_patterns.len() / 2 {
                    println!("✨ 大部分嵌套媒体查询都被正确处理");
                } else {
                    println!("⚠️  部分嵌套媒体查询可能需要改进");
                }

                println!("\n🔍 期望的行为:");
                println!("- 内层媒体查询应该与外层媒体查询合并");
                println!("- 使用 'and' 操作符连接多个媒体条件");
                println!("- 保持选择器的嵌套结构");
            } else {
                println!("\n❌ 媒体查询嵌套功能可能不支持或需要实现");
                println!("\n💡 如果不支持，CSS 输出可能是:");
                println!("- 分离的媒体查询块（而不是合并的）");
                println!("- 或者编译错误");
            }

            // 分析 CSS 结构
            println!("\n=== CSS 结构分析 ===");
            let lines: Vec<&str> = css.lines().filter(|l| !l.trim().is_empty()).collect();
            for (i, line) in lines.iter().enumerate() {
                if line.trim().starts_with("@media") {
                    println!("第 {} 行: {}", i + 1, line.trim());
                }
            }
        }
        Err(e) => {
            println!("编译失败: {}", e);
            println!("\n💭 可能的原因:");
            println!("1. 媒体查询嵌套功能尚未实现");
            println!("2. 语法解析器不支持嵌套媒体查询");
            println!("3. 编译器无法处理复杂的媒体查询合并");
            return Err(e.into());
        }
    }

    Ok(())
}
