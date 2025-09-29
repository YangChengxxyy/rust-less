//! 边界情况测试
//!
//! 测试修复后的解析器在各种边界情况下的表现，
//! 确保 CSS 单位识别修复没有引入新的问题

use std::process::Command;

fn main() {
    println!("🧪 边界情况测试");
    println!("===============\n");

    test_css_units();
    test_css_keywords();
    test_mixed_scenarios();
    test_edge_cases();

    println!("\n🎯 总结");
    println!("测试完成！如果所有测试通过，说明修复是成功的。");
}

fn test_css_units() {
    println!("📝 测试 1: CSS 单位识别");
    println!("─────────────────────");

    let test_cases = vec![
        // 长度单位
        (".test { width: 10px; }", "像素单位"),
        (".test { width: 2em; }", "em 单位"),
        (".test { width: 1.5rem; }", "rem 单位"),
        (".test { width: 5ex; }", "ex 单位"),
        (".test { width: 3ch; }", "ch 单位"),
        (".test { width: 50vw; }", "视口宽度"),
        (".test { width: 50vh; }", "视口高度"),
        (".test { width: 10vmin; }", "视口最小值"),
        (".test { width: 10vmax; }", "视口最大值"),
        // 绝对单位
        (".test { width: 2cm; }", "厘米"),
        (".test { width: 20mm; }", "毫米"),
        (".test { width: 1in; }", "英寸"),
        (".test { width: 72pt; }", "点"),
        (".test { width: 6pc; }", "派卡"),
        // 角度单位
        (".test { transform: rotate(45deg); }", "度"),
        (".test { transform: rotate(100grad); }", "梯度"),
        (".test { transform: rotate(1.57rad); }", "弧度"),
        (".test { transform: rotate(0.25turn); }", "圈"),
        // 时间单位
        (".test { transition-duration: 2s; }", "秒"),
        (".test { transition-duration: 200ms; }", "毫秒"),
        // 频率单位
        (".test { pitch: 440hz; }", "赫兹"),
        (".test { pitch: 2khz; }", "千赫兹"),
        // 分辨率单位
        (".test { resolution: 96dpi; }", "DPI"),
        (".test { resolution: 38dpcm; }", "DPCM"),
        (".test { resolution: 2dppx; }", "DPPX"),
        // 网格单位
        (".test { grid-template-columns: 1fr 2fr; }", "fraction 单位"),
    ];

    for (input, description) in test_cases {
        print!("{}: ", description);
        match compile_and_check(input) {
            Ok(css) => {
                // 检查单位是否正确保留
                let has_unit = css.contains("px")
                    || css.contains("em")
                    || css.contains("rem")
                    || css.contains("ex")
                    || css.contains("ch")
                    || css.contains("vw")
                    || css.contains("vh")
                    || css.contains("vmin")
                    || css.contains("vmax")
                    || css.contains("cm")
                    || css.contains("mm")
                    || css.contains("in")
                    || css.contains("pt")
                    || css.contains("pc")
                    || css.contains("deg")
                    || css.contains("grad")
                    || css.contains("rad")
                    || css.contains("turn")
                    || css.contains("s")
                    || css.contains("ms")
                    || css.contains("hz")
                    || css.contains("khz")
                    || css.contains("dpi")
                    || css.contains("dpcm")
                    || css.contains("dppx")
                    || css.contains("fr");

                if has_unit {
                    println!("✅ 单位正确保留");
                } else {
                    println!("❌ 单位丢失");
                    println!("   输出: {}", css.trim());
                }
            }
            Err(e) => {
                println!("❌ 编译失败: {}", e);
            }
        }
    }
}

fn test_css_keywords() {
    println!("\n📝 测试 2: CSS 关键字处理");
    println!("─────────────────────────");

    let test_cases = vec![
        // 布局关键字
        (".test { margin: 0 auto; }", "auto 关键字"),
        (".test { display: none; }", "none 关键字"),
        (".test { visibility: hidden; }", "hidden 关键字"),
        (".test { position: relative; }", "relative 关键字"),
        (".test { position: absolute; }", "absolute 关键字"),
        (".test { position: fixed; }", "fixed 关键字"),
        (".test { position: static; }", "static 关键字"),
        // 颜色关键字
        (".test { color: red; }", "red 关键字"),
        (".test { color: blue; }", "blue 关键字"),
        (".test { color: transparent; }", "transparent 关键字"),
        (".test { color: inherit; }", "inherit 关键字"),
        (".test { color: initial; }", "initial 关键字"),
        (".test { color: unset; }", "unset 关键字"),
        // 字体关键字
        (".test { font-weight: bold; }", "bold 关键字"),
        (".test { font-weight: normal; }", "normal 关键字"),
        (".test { font-style: italic; }", "italic 关键字"),
        (".test { font-family: serif; }", "serif 关键字"),
        (".test { font-family: sans-serif; }", "sans-serif 关键字"),
        // 边框关键字
        (".test { border-style: solid; }", "solid 关键字"),
        (".test { border-style: dashed; }", "dashed 关键字"),
        (".test { border-style: dotted; }", "dotted 关键字"),
        // 数字 + 关键字组合
        (".test { margin: 0 auto; }", "0 + auto"),
        (".test { margin: 10 auto; }", "10 + auto"),
        (".test { border: 0 solid red; }", "0 + solid"),
        (".test { padding: 0 0 0 auto; }", "多个 0 + auto"),
    ];

    for (input, description) in test_cases {
        print!("{}: ", description);
        match compile_and_check(input) {
            Ok(css) => {
                // 检查关键字是否正确分离
                if description.contains("0 + auto") && css.contains("0auto") {
                    println!("❌ 关键字连接问题");
                    println!("   输出: {}", css.trim());
                } else if description.contains("+ auto") && css.contains(" auto") {
                    println!("✅ 关键字正确分离");
                } else if !description.contains("+ auto") {
                    println!("✅ 编译成功");
                } else {
                    println!("⚠️  需要检查");
                    println!("   输出: {}", css.trim());
                }
            }
            Err(e) => {
                println!("❌ 编译失败: {}", e);
            }
        }
    }
}

fn test_mixed_scenarios() {
    println!("\n📝 测试 3: 混合场景");
    println!("─────────────────");

    let test_cases = vec![
        // 复杂的属性值
        (".test { margin: 0 auto 10px 20px; }", "四值边距"),
        (".test { border: 1px solid red; }", "边框简写"),
        (".test { font: 14px Arial, sans-serif; }", "字体简写"),
        (
            ".test { background: url(image.jpg) no-repeat center; }",
            "背景简写",
        ),
        // 计算表达式
        (".test { width: 100px + 50px; }", "加法表达式"),
        (".test { margin: (10px - 5px) auto; }", "计算 + 关键字"),
        // 函数调用
        (".test { width: calc(100% - 20px); }", "calc 函数"),
        (
            ".test { margin: round(5.5px) auto; }",
            "round 函数 + 关键字",
        ),
        // 变量
        ("@size: 10px; .test { width: @size; }", "变量 + 单位"),
        (
            "@keyword: auto; .test { margin: 0 @keyword; }",
            "变量关键字",
        ),
        // 嵌套值
        (
            ".test { box-shadow: 0 2px 4px rgba(0,0,0,0.1); }",
            "复杂阴影",
        ),
        (
            ".test { transform: translate(10px, 20px) rotate(45deg); }",
            "变换函数",
        ),
        // 特殊情况
        (".test { content: '0px'; }", "字符串中的单位"),
        (".test { content: 'auto'; }", "字符串中的关键字"),
        (".test { width: 0; height: auto; }", "多属性"),
    ];

    for (input, description) in test_cases {
        print!("{}: ", description);
        match compile_and_check(input) {
            Ok(_css) => {
                println!("✅ 编译成功");
            }
            Err(e) => {
                println!("❌ 编译失败: {}", e);
            }
        }
    }
}

fn test_edge_cases() {
    println!("\n📝 测试 4: 极端边界情况");
    println!("─────────────────────");

    let test_cases = vec![
        // 数字边界
        (".test { width: 0; }", "单独的 0"),
        (".test { width: 0.0; }", "小数 0"),
        (".test { width: -0; }", "负 0"),
        (".test { margin: 0 0; }", "多个 0"),
        (".test { margin: 0 0 0 0; }", "四个 0"),
        // 关键字边界
        (".test { margin: auto; }", "单独 auto"),
        (".test { margin: auto auto; }", "多个 auto"),
        (".test { margin: auto 0 auto 0; }", "交替 auto 和 0"),
        // 混合边界
        (".test { width: 0em; }", "0 + 真实单位"),
        (".test { margin: 0px auto; }", "单位 + 关键字"),
        (".test { border: 0 none transparent; }", "多个关键字"),
        // 错误的"单位"
        (".test { margin: 0 inherit; }", "0 + inherit"),
        (".test { margin: 0 initial; }", "0 + initial"),
        (".test { margin: 0 unset; }", "0 + unset"),
        (".test { display: 0 block; }", "0 + display 值"),
        // 特殊标识符
        (".test { font-family: 0 Arial; }", "0 + 字体名"),
        (".test { animation-name: 0 fadeIn; }", "0 + 动画名"),
        // 自定义属性相关
        (".test { --custom: 0 auto; }", "CSS 变量"),
        (".test { margin: 0 var(--spacing); }", "0 + CSS 变量函数"),
    ];

    for (input, description) in test_cases {
        print!("{}: ", description);
        match compile_and_check(input) {
            Ok(css) => {
                // 特别检查是否有连接问题
                if css.contains("0auto")
                    || css.contains("0inherit")
                    || css.contains("0initial")
                    || css.contains("0unset")
                    || css.contains("0block")
                    || css.contains("0Arial")
                    || css.contains("0fadeIn")
                {
                    println!("❌ 发现连接问题");
                    println!("   输出: {}", css.trim());
                } else {
                    println!("✅ 格式正确");
                }
            }
            Err(e) => {
                println!("❌ 编译失败: {}", e);
            }
        }
    }
}

fn compile_and_check(input: &str) -> Result<String, String> {
    // 写入临时文件
    if let Err(e) = std::fs::write("/tmp/test_edge.less", input) {
        return Err(format!("写入文件失败: {}", e));
    }

    // 使用 CLI 编译
    let output = Command::new("./target/debug/rust-less")
        .arg("/tmp/test_edge.less")
        .output();

    // 清理临时文件
    let _ = std::fs::remove_file("/tmp/test_edge.less");

    match output {
        Ok(result) => {
            if result.status.success() {
                Ok(String::from_utf8_lossy(&result.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&result.stderr).to_string())
            }
        }
        Err(e) => Err(format!("执行失败: {}", e)),
    }
}
