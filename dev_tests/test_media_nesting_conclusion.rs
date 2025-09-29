use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Rust LESS 编译器媒体查询嵌套功能分析 ===\n");

    // 测试案例 1: 基本嵌套媒体查询
    let test1 = r#"
@media (min-width: 768px) {
    .container {
        width: 100%;

        @media (max-width: 1200px) {
            width: 80%;
        }
    }
}
"#;

    println!("📋 测试案例 1: 基本媒体查询嵌套");
    println!("输入: {}", test1);

    let mut compiler = rust_less::compiler::Compiler::new();
    match compiler.compile(test1) {
        Ok(css) => {
            println!("输出:");
            println!("{}", css);

            // 检查是否合并了媒体查询
            let has_combined = css.contains("@media (min-width: 768px) and (max-width: 1200px)");
            let has_separate = css.contains("@media (min-width: 768px)")
                && css.contains("@media (max-width: 1200px)");

            println!("✅ 结果分析:");
            println!("   - 合并媒体查询: {}", has_combined);
            println!("   - 分离媒体查询: {}", has_separate);
        }
        Err(e) => println!("❌ 编译失败: {}", e),
    }

    println!("\n{}\n", "=".repeat(60));

    // 测试案例 2: 多层嵌套
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

    println!("📋 测试案例 2: 三层媒体查询嵌套");
    println!("输入: {}", test2);

    let mut compiler2 = rust_less::compiler::Compiler::new();
    match compiler2.compile(test2) {
        Ok(css) => {
            println!("输出:");
            println!("{}", css);

            let media_count = css.matches("@media").count();
            println!("✅ 媒体查询总数: {}", media_count);
        }
        Err(e) => println!("❌ 编译失败: {}", e),
    }

    println!("\n{}\n", "=".repeat(60));

    // 测试案例 3: 对比 - CSS规则内的媒体查询 vs 顶级媒体查询
    let test3 = r#"
.container {
    width: 100%;

    @media (max-width: 768px) {
        width: 90%;
    }
}

@media (max-width: 768px) {
    .sidebar {
        display: none;
    }
}
"#;

    println!("📋 测试案例 3: CSS规则内的媒体查询 vs 顶级媒体查询");
    println!("输入: {}", test3);

    let mut compiler3 = rust_less::compiler::Compiler::new();
    match compiler3.compile(test3) {
        Ok(css) => {
            println!("输出:");
            println!("{}", css);

            let media_count = css.matches("@media (max-width: 768px)").count();
            println!("✅ '@media (max-width: 768px)' 出现次数: {}", media_count);
        }
        Err(e) => println!("❌ 编译失败: {}", e),
    }

    println!("\n{}\n", "=".repeat(60));

    // 结论总结
    println!("🔍 分析结论:");
    println!();
    println!("1. 媒体查询嵌套媒体查询的支持情况:");
    println!("   ❌ 不支持媒体查询条件合并 (使用 'and' 操作符)");
    println!("   ✅ 支持将嵌套媒体查询提取为独立的媒体查询块");
    println!("   ✅ 支持在CSS规则内嵌套媒体查询（但不是媒体查询内嵌套媒体查询）");
    println!();
    println!("2. 实际行为:");
    println!("   - 外层媒体查询: 正常处理");
    println!("   - 内层媒体查询: 被提取为独立的媒体查询，忽略外层条件");
    println!("   - 选择器嵌套: 正确处理");
    println!();
    println!("3. 期望的标准LESS行为:");
    println!("   输入: @media (min-width: 768px) {{ .test {{ @media (max-width: 1200px) {{ width: 80%; }} }} }}");
    println!(
        "   期望: @media (min-width: 768px) and (max-width: 1200px) {{ .test {{ width: 80%; }} }}"
    );
    println!("   实际: @media (min-width: 768px) {{ .test {{ }} }} + @media (max-width: 1200px) {{ .test {{ width: 80%; }} }}");
    println!();
    println!("4. 功能状态:");
    println!("   🟡 部分支持: 能处理嵌套结构但不合并媒体查询条件");
    println!("   🔧 需要改进: 媒体查询条件合并功能");
    println!();
    println!("5. 技术原因:");
    println!("   - compile_at_rule 函数中的 is_nested 参数在 compile_statement 中总是为 false");
    println!("   - 缺少媒体查询条件合并逻辑");
    println!("   - 需要在编译器中添加媒体查询上下文追踪");

    Ok(())
}
