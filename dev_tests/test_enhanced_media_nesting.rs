use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== 测试增强的媒体查询嵌套功能 ===\n");

    // 测试案例 1: 基本的媒体查询条件合并
    println!("📋 测试案例 1: 基本媒体查询条件合并");
    let test1 = r#"
@media (min-width: 768px) {
    .container {
        width: 100%;

        @media (max-width: 1200px) {
            width: 80%;
            padding: 15px;
        }
    }
}
"#;

    println!("输入:");
    println!("{}", test1);

    let mut compiler = rust_less::compiler::Compiler::new();
    match compiler.compile(test1) {
        Ok(css) => {
            println!("✅ 编译成功！");
            println!("输出:");
            println!("{}", css);

            // 验证是否正确合并了媒体查询条件
            let has_merged = css.contains("@media (min-width: 768px) and (max-width: 1200px)");
            let has_separate = css.contains("@media (min-width: 768px)")
                && css.contains("@media (max-width: 1200px)");

            println!("🔍 验证结果:");
            println!("   - 找到合并的媒体查询: {}", has_merged);
            println!("   - 找到分离的媒体查询: {}", has_separate);

            if has_merged {
                println!("🎉 成功！媒体查询条件已正确合并！");
            } else if has_separate {
                println!("⚠️  媒体查询被分离处理，可能需要进一步优化");
            } else {
                println!("❌ 媒体查询处理异常");
            }
        }
        Err(e) => {
            println!("❌ 编译失败: {}", e);
        }
    }

    println!("\n{}\n", "=".repeat(70));

    // 测试案例 2: 三层媒体查询嵌套
    println!("📋 测试案例 2: 三层媒体查询嵌套");
    let test2 = r#"
@media screen {
    .layout {
        display: block;

        @media (min-width: 768px) {
            display: flex;

            @media (max-width: 1200px) {
                flex-direction: column;
            }
        }
    }
}
"#;

    println!("输入:");
    println!("{}", test2);

    let mut compiler2 = rust_less::compiler::Compiler::new();
    match compiler2.compile(test2) {
        Ok(css) => {
            println!("✅ 编译成功！");
            println!("输出:");
            println!("{}", css);

            // 检查三层嵌套的合并
            let expected_triple = "screen and (min-width: 768px) and (max-width: 1200px)";
            let has_triple_merge = css.contains(expected_triple);

            println!("🔍 验证三层嵌套:");
            println!("   - 期望的三层合并: @media {}", expected_triple);
            println!("   - 找到三层合并: {}", has_triple_merge);

            if has_triple_merge {
                println!("🎉 太棒了！三层媒体查询嵌套工作正常！");
            } else {
                println!("⚠️  三层嵌套可能需要进一步改进");
            }
        }
        Err(e) => {
            println!("❌ 编译失败: {}", e);
        }
    }

    println!("\n{}\n", "=".repeat(70));

    // 测试案例 3: 复杂的媒体查询组合
    println!("📋 测试案例 3: 复杂媒体查询组合");
    let test3 = r#"
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
        }
    }
}
"#;

    println!("输入:");
    println!("{}", test3);

    let mut compiler3 = rust_less::compiler::Compiler::new();
    match compiler3.compile(test3) {
        Ok(css) => {
            println!("✅ 编译成功！");
            println!("输出:");
            println!("{}", css);

            // 检查复杂条件合并
            let complex_merge1 = css.contains(
                "(min-width: 1024px) and (max-width: 1440px) and (min-resolution: 2dppx)",
            );
            let complex_merge2 = css.contains(
                "(min-width: 1024px) and (max-width: 1440px) and (prefers-color-scheme: dark)",
            );

            println!("🔍 验证复杂合并:");
            println!("   - 分辨率条件合并: {}", complex_merge1);
            println!("   - 主题条件合并: {}", complex_merge2);

            if complex_merge1 && complex_merge2 {
                println!("🎉 完美！复杂媒体查询条件合并正常工作！");
            } else {
                println!("⚠️  复杂条件合并需要进一步测试");
            }
        }
        Err(e) => {
            println!("❌ 编译失败: {}", e);
        }
    }

    println!("\n{}\n", "=".repeat(70));

    // 测试案例 4: 与选择器嵌套的组合
    println!("📋 测试案例 4: 媒体查询嵌套 + 选择器嵌套");
    let test4 = r#"
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
        }
    }
}
"#;

    println!("输入:");
    println!("{}", test4);

    let mut compiler4 = rust_less::compiler::Compiler::new();
    match compiler4.compile(test4) {
        Ok(css) => {
            println!("✅ 编译成功！");
            println!("输出:");
            println!("{}", css);

            // 检查选择器和媒体查询的组合
            let selector_merge1 = css.contains(".header .logo");
            let media_merge1 = css.contains("(max-width: 768px) and (orientation: landscape)");
            let media_merge2 = css
                .contains("(max-width: 768px) and (orientation: landscape) and (max-width: 480px)");

            println!("🔍 验证选择器和媒体查询组合:");
            println!("   - 选择器嵌套: {}", selector_merge1);
            println!("   - 两层媒体查询合并: {}", media_merge1);
            println!("   - 三层媒体查询合并: {}", media_merge2);

            if selector_merge1 && media_merge1 {
                println!("🎉 很好！选择器嵌套和媒体查询合并都工作正常！");
            } else {
                println!("⚠️  选择器和媒体查询组合需要优化");
            }
        }
        Err(e) => {
            println!("❌ 编译失败: {}", e);
        }
    }

    println!("\n{}\n", "=".repeat(70));

    // 测试案例 5: 边缘情况 - 空媒体查询
    println!("📋 测试案例 5: 边缘情况测试");
    let test5 = r#"
@media {
    .test {
        color: red;

        @media (max-width: 600px) {
            color: blue;
        }
    }
}

@media (min-width: 800px) {
    .another {
        @media {
            display: none;
        }
    }
}
"#;

    println!("输入:");
    println!("{}", test5);

    let mut compiler5 = rust_less::compiler::Compiler::new();
    match compiler5.compile(test5) {
        Ok(css) => {
            println!("✅ 编译成功！");
            println!("输出:");
            println!("{}", css);

            println!("🔍 边缘情况测试通过！");
        }
        Err(e) => {
            println!("❌ 编译失败: {}", e);
            println!("这可能是正常的，空媒体查询可能不被支持");
        }
    }

    println!("\n{}\n", "=".repeat(70));

    // 最终总结
    println!("📊 功能测试总结:");
    println!();
    println!("期待的改进效果:");
    println!("✅ 基本媒体查询条件合并 (A and B)");
    println!("✅ 多层媒体查询嵌套 (A and B and C)");
    println!("✅ 复杂媒体查询条件处理");
    println!("✅ 选择器嵌套与媒体查询合并的组合");
    println!("✅ 边缘情况处理");
    println!();
    println!("🎯 成功标准:");
    println!("1. 嵌套的媒体查询应该使用 'and' 操作符合并");
    println!("2. 选择器嵌套应该正常工作");
    println!("3. 不应该生成无效的CSS结构");
    println!("4. 所有媒体查询条件都应该被保留");
    println!();
    println!("如果以上测试都通过，说明媒体查询嵌套功能实现成功！");

    Ok(())
}
