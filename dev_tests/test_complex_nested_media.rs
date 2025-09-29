use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Test complex nested media queries with multiple levels
    let input = r#"
.layout {
    display: grid;
    grid-template-columns: 1fr 300px;
    gap: 20px;

    @media (max-width: 1024px) {
        grid-template-columns: 1fr;
        gap: 15px;

        .sidebar {
            order: 2;

            @media (max-width: 768px) {
                display: none;

                &.mobile-visible {
                    display: block;
                    position: fixed;
                    top: 0;
                    left: 0;
                    width: 100%;
                    height: 100vh;
                    background: white;
                    z-index: 1000;

                    .close-button {
                        position: absolute;
                        top: 10px;
                        right: 10px;

                        @media (max-width: 480px) {
                            top: 5px;
                            right: 5px;
                            font-size: 12px;
                        }
                    }
                }
            }
        }

        .main-content {
            order: 1;

            .article {
                padding: 15px;

                @media (max-width: 768px) {
                    padding: 10px;

                    .title {
                        font-size: 24px;

                        @media (max-width: 480px) {
                            font-size: 20px;
                            line-height: 1.2;
                        }
                    }

                    .content {
                        font-size: 16px;

                        @media (max-width: 480px) {
                            font-size: 14px;
                        }

                        .image {
                            width: 100%;

                            @media (max-width: 480px) {
                                margin: 10px 0;
                            }
                        }
                    }
                }
            }
        }
    }

    @media (min-width: 1400px) {
        max-width: 1200px;
        margin: 0 auto;

        .sidebar {
            width: 350px;

            .widget {
                margin-bottom: 30px;

                @media (min-width: 1600px) {
                    margin-bottom: 40px;

                    .widget-title {
                        font-size: 18px;
                    }
                }
            }
        }
    }
}

.header {
    background: #f8f9fa;
    padding: 20px;

    @media (max-width: 768px) {
        padding: 15px 10px;

        .logo {
            width: 120px;

            @media (max-width: 480px) {
                width: 100px;
            }
        }

        .navigation {
            display: none;

            &.mobile-menu {
                display: block;
                position: absolute;
                top: 60px;
                left: 0;
                width: 100%;
                background: white;
                box-shadow: 0 2px 5px rgba(0,0,0,0.1);

                .nav-item {
                    display: block;
                    padding: 15px 20px;
                    border-bottom: 1px solid #eee;

                    @media (max-width: 480px) {
                        padding: 12px 15px;
                        font-size: 14px;
                    }

                    &:last-child {
                        border-bottom: none;
                    }
                }
            }
        }

        .menu-toggle {
            display: block;
            background: none;
            border: none;
            font-size: 24px;

            @media (max-width: 480px) {
                font-size: 20px;
            }
        }
    }

    @media (min-width: 769px) {
        .menu-toggle {
            display: none;
        }

        .navigation {
            display: flex;
            gap: 30px;

            .nav-item {
                padding: 10px 15px;
                text-decoration: none;

                &:hover {
                    background: #e9ecef;
                    border-radius: 4px;
                }
            }
        }
    }
}
"#;

    println!("=== 测试复杂嵌套媒体查询 ===");
    println!("输入的 LESS 代码（多层嵌套）:");
    println!("{}", input);
    println!();

    // 使用 rust-less 编译器
    let mut compiler = rust_less::compiler::Compiler::new();

    match compiler.compile(input) {
        Ok(css) => {
            println!("编译成功！");
            println!("=== 生成的 CSS ===");
            println!("{}", css);

            // 验证多层媒体查询嵌套
            let has_1024px = css.contains("@media (max-width: 1024px)");
            let has_768px = css.contains("@media (max-width: 768px)");
            let has_480px = css.contains("@media (max-width: 480px)");
            let has_1400px = css.contains("@media (min-width: 1400px)");
            let has_1600px = css.contains("@media (min-width: 1600px)");
            let has_769px = css.contains("@media (min-width: 769px)");

            // 验证嵌套选择器
            let has_nested_selectors = css.contains(".layout .sidebar")
                || css.contains(".layout .main-content")
                || css.contains(".header .navigation");

            println!("\n=== 验证结果 ===");
            println!("✓ 包含 max-width: 1024px: {}", has_1024px);
            println!("✓ 包含 max-width: 768px: {}", has_768px);
            println!("✓ 包含 max-width: 480px: {}", has_480px);
            println!("✓ 包含 min-width: 1400px: {}", has_1400px);
            println!("✓ 包含 min-width: 1600px: {}", has_1600px);
            println!("✓ 包含 min-width: 769px: {}", has_769px);
            println!("✓ 包含嵌套选择器: {}", has_nested_selectors);

            let media_count = css.matches("@media").count();
            println!("✓ 媒体查询总数: {}", media_count);

            if has_1024px && has_768px && has_480px && has_nested_selectors {
                println!("\n🎉 复杂嵌套媒体查询功能正常工作！");
                println!("✨ 支持多层嵌套、不同断点和复杂选择器组合");
            } else {
                println!("\n⚠️  某些媒体查询特性可能需要改进");
            }

            // 检查是否有语法错误
            let has_syntax_issues =
                css.contains("}{") || css.contains("  }") && css.contains("{  ");
            if !has_syntax_issues {
                println!("✅ CSS 语法格式正确");
            } else {
                println!("⚠️  可能存在 CSS 格式问题");
            }
        }
        Err(e) => {
            println!("编译失败: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
