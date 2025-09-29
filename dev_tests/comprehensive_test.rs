use rust_less::{compile, parser::Parser};

fn main() {
    println!("🔍 Rust LESS 编译器功能全面检查");
    println!("=====================================");

    let mut passed = 0;
    let mut failed = 0;

    // 核心功能测试
    test_variables(&mut passed, &mut failed);
    test_arithmetic(&mut passed, &mut failed);
    test_nesting(&mut passed, &mut failed);
    test_parent_selector(&mut passed, &mut failed);
    test_media_queries(&mut passed, &mut failed);
    test_built_in_functions(&mut passed, &mut failed);

    // 混合器功能测试
    test_basic_mixins(&mut passed, &mut failed);
    test_mixin_parameters(&mut passed, &mut failed);
    test_mixin_errors(&mut passed, &mut failed);

    // 其他功能测试
    test_comments(&mut passed, &mut failed);
    test_imports(&mut passed, &mut failed);
    test_css_passthrough(&mut passed, &mut failed);

    // 总结
    println!("\n📊 测试结果总结");
    println!("=================");
    println!("✅ 通过: {}", passed);
    println!("❌ 失败: {}", failed);
    println!(
        "📈 成功率: {:.1}%",
        (passed as f64 / (passed + failed) as f64) * 100.0
    );

    if failed == 0 {
        println!("\n🎉 所有功能正常工作！");
    } else {
        println!("\n⚠️  发现 {} 个问题需要修复", failed);
    }
}

fn test_variables(passed: &mut i32, failed: &mut i32) {
    println!("\n🧪 测试 1: 变量系统");

    let less_code = r#"
@primary-color: #333;
@margin: 10px;
@font-size: 14px;

.header {
    color: @primary-color;
    margin: @margin;
    font-size: @font-size;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            if css.contains("#333") && css.contains("10px") && css.contains("14px") {
                println!("✅ 变量系统正常");
                *passed += 1;
            } else {
                println!("❌ 变量替换失败");
                println!("   CSS: {}", css);
                *failed += 1;
            }
        }
        Err(e) => {
            println!("❌ 变量编译失败: {}", e);
            *failed += 1;
        }
    }
}

fn test_arithmetic(passed: &mut i32, failed: &mut i32) {
    println!("\n🧪 测试 2: 算术运算");

    let less_code = r#"
@base: 10px;

.container {
    width: @base * 2;
    height: @base + 5px;
    margin: @base / 2;
    padding: @base - 2px;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            if css.contains("width: 20px")
                && css.contains("height: 15px")
                && css.contains("margin: 5px")
                && css.contains("padding: 8px")
            {
                println!("✅ 算术运算正常");
                *passed += 1;
            } else {
                println!("❌ 算术运算失败");
                println!("   CSS: {}", css);
                *failed += 1;
            }
        }
        Err(e) => {
            println!("❌ 算术编译失败: {}", e);
            *failed += 1;
        }
    }
}

fn test_nesting(passed: &mut i32, failed: &mut i32) {
    println!("\n🧪 测试 3: 选择器嵌套");

    let less_code = r#"
.navbar {
    height: 60px;
    ul {
        margin: 0;
        li {
            list-style: none;
            a {
                text-decoration: none;
            }
        }
    }
}
"#;

    match compile(less_code) {
        Ok(css) => {
            if css.contains(".navbar ul")
                && css.contains(".navbar ul li")
                && css.contains(".navbar ul li a")
            {
                println!("✅ 选择器嵌套正常");
                *passed += 1;
            } else {
                println!("❌ 选择器嵌套失败");
                println!("   CSS: {}", css);
                *failed += 1;
            }
        }
        Err(e) => {
            println!("❌ 嵌套编译失败: {}", e);
            *failed += 1;
        }
    }
}

fn test_parent_selector(passed: &mut i32, failed: &mut i32) {
    println!("\n🧪 测试 4: 父选择器引用");

    let less_code = r#"
.button {
    padding: 10px;
    &:hover {
        background: #eee;
    }
    &.active {
        background: #333;
    }
    &-large {
        padding: 20px;
    }
}
"#;

    match compile(less_code) {
        Ok(css) => {
            if css.contains(".button:hover")
                && css.contains(".button.active")
                && css.contains(".button-large")
            {
                println!("✅ 父选择器引用正常");
                *passed += 1;
            } else {
                println!("❌ 父选择器引用失败");
                println!("   CSS: {}", css);
                *failed += 1;
            }
        }
        Err(e) => {
            println!("❌ 父选择器编译失败: {}", e);
            *failed += 1;
        }
    }
}

fn test_media_queries(passed: &mut i32, failed: &mut i32) {
    println!("\n🧪 测试 5: 媒体查询嵌套");

    let less_code = r#"
.responsive {
    width: 100%;
    @media (max-width: 768px) {
        width: 50%;
        font-size: 14px;
    }
}
"#;

    match compile(less_code) {
        Ok(css) => {
            if css.contains("@media (max-width: 768px)") && css.contains(".responsive") {
                println!("✅ 媒体查询嵌套正常");
                *passed += 1;
            } else {
                println!("❌ 媒体查询嵌套失败");
                println!("   CSS: {}", css);
                *failed += 1;
            }
        }
        Err(e) => {
            println!("❌ 媒体查询编译失败: {}", e);
            *failed += 1;
        }
    }
}

fn test_built_in_functions(passed: &mut i32, failed: &mut i32) {
    println!("\n🧪 测试 6: 内置函数");

    let less_code = r#"
.math {
    rounded: round(10.6px);
    ceiling: ceil(10.1px);
    percentage: percentage(0.5);
}
"#;

    match compile(less_code) {
        Ok(css) => {
            if css.contains("rounded: 11px")
                && css.contains("ceiling: 11px")
                && css.contains("percentage: 50%")
            {
                println!("✅ 内置函数正常");
                *passed += 1;
            } else {
                println!("❌ 内置函数失败");
                println!("   CSS: {}", css);
                *failed += 1;
            }
        }
        Err(e) => {
            println!("❌ 内置函数编译失败: {}", e);
            *failed += 1;
        }
    }
}

fn test_basic_mixins(passed: &mut i32, failed: &mut i32) {
    println!("\n🧪 测试 7: 基础混合器（无参数）");

    let less_code = r#"
.border-radius {
    border-radius: 5px;
    -webkit-border-radius: 5px;
}

.button {
    padding: 10px;
    .border-radius;
    background: #007cba;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            if css.contains(".button")
                && css.contains("border-radius: 5px")
                && css.contains("padding: 10px")
            {
                println!("✅ 基础混合器正常");
                *passed += 1;
            } else {
                println!("❌ 基础混合器失败");
                println!("   CSS: {}", css);
                *failed += 1;
            }
        }
        Err(e) => {
            println!("❌ 基础混合器编译失败: {}", e);
            *failed += 1;
        }
    }
}

fn test_mixin_parameters(passed: &mut i32, failed: &mut i32) {
    println!("\n🧪 测试 8: 参数化混合器");

    let less_code = r#"
.border-radius(@radius) {
    border-radius: @radius;
    -webkit-border-radius: @radius;
}

.button {
    .border-radius(10px);
    background: #007cba;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            if css.contains(".button") && css.contains("border-radius: 10px") {
                println!("✅ 参数化混合器正常");
                *passed += 1;
            } else {
                println!("❌ 参数化混合器失败");
                println!("   CSS: {}", css);
                *failed += 1;
            }
        }
        Err(e) => {
            println!("⚠️  参数化混合器编译失败: {} (预期可能失败)", e);
            *failed += 1;
        }
    }
}

fn test_mixin_errors(passed: &mut i32, failed: &mut i32) {
    println!("\n🧪 测试 9: 混合器错误处理");

    let less_code = r#"
.button {
    .undefined-mixin;
    background: red;
}
"#;

    match compile(less_code) {
        Ok(_) => {
            println!("❌ 应该检测到未定义混合器错误");
            *failed += 1;
        }
        Err(e) => {
            if e.to_string().contains("Undefined mixin") {
                println!("✅ 混合器错误处理正常");
                *passed += 1;
            } else {
                println!("❌ 错误类型不正确: {}", e);
                *failed += 1;
            }
        }
    }
}

fn test_comments(passed: &mut i32, failed: &mut i32) {
    println!("\n🧪 测试 10: 注释处理");

    let less_code = r#"
/* 块注释 */
// 行注释
.test {
    color: red; /* 内联注释 */
    // 另一个行注释
    background: blue;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            if css.contains(".test")
                && css.contains("color: red")
                && css.contains("background: blue")
            {
                println!("✅ 注释处理正常");
                *passed += 1;
            } else {
                println!("❌ 注释处理失败");
                println!("   CSS: {}", css);
                *failed += 1;
            }
        }
        Err(e) => {
            println!("❌ 注释编译失败: {}", e);
            *failed += 1;
        }
    }
}

fn test_imports(passed: &mut i32, failed: &mut i32) {
    println!("\n🧪 测试 11: CSS导入");

    let less_code = r#"
@import "reset.css";

.main {
    color: black;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            if css.contains("@import") && css.contains(".main") {
                println!("✅ CSS导入正常");
                *passed += 1;
            } else {
                println!("❌ CSS导入失败");
                println!("   CSS: {}", css);
                *failed += 1;
            }
        }
        Err(e) => {
            println!("❌ 导入编译失败: {}", e);
            *failed += 1;
        }
    }
}

fn test_css_passthrough(passed: &mut i32, failed: &mut i32) {
    println!("\n🧪 测试 12: 原生CSS透传");

    let less_code = r#"
.standard-css {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
}

#unique-id {
    position: absolute;
    top: 0;
    left: 0;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            if css.contains(".standard-css")
                && css.contains("display: flex")
                && css.contains("#unique-id")
                && css.contains("position: absolute")
            {
                println!("✅ 原生CSS透传正常");
                *passed += 1;
            } else {
                println!("❌ 原生CSS透传失败");
                println!("   CSS: {}", css);
                *failed += 1;
            }
        }
        Err(e) => {
            println!("❌ 原生CSS编译失败: {}", e);
            *failed += 1;
        }
    }
}
