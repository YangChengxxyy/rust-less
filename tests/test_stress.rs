//! 压力测试：验证大型 LESS 文件的编译正确性与内存行为。
//!
//! 生成大量规则/嵌套/变量引用，确认：
//! - 编译不 panic、不栈溢出
//! - 输出规模与输入线性相关（无平方级输出爆炸）

use rust_less::Compiler;
use std::time::Instant;

fn generate_large_input(rules: usize) -> String {
    let mut input = String::with_capacity(rules * 128);
    input.push_str("@base-color: #336699;\n@spacing: 8px;\n\n");
    for i in 0..rules {
        input.push_str(&format!(
            ".component-{i} {{\n  color: @base-color;\n  padding: @spacing * 2;\n\n  .inner {{\n    margin: @spacing / 2;\n\n    &:hover {{\n      color: darken(@base-color, 10%);\n    }}\n  }}\n}}\n\n"
        ));
    }
    input
}

#[test]
fn test_large_file_compiles() {
    let input = generate_large_input(2_000);
    let start = Instant::now();
    let mut compiler = Compiler::new();
    let css = compiler.compile(&input).expect("large file should compile");
    let elapsed = start.elapsed();
    assert_eq!(css.matches(".component-").count(), 6_000);
    assert!(css.contains(".component-0 .inner:hover"));
    // 基本性能护栏：2000 条嵌套规则应在数秒内完成（CI 机器差异留足余量）
    assert!(elapsed.as_secs() < 30, "compilation took too long: {elapsed:?}");
}

#[test]
fn test_deep_nesting_compiles() {
    // 100 层嵌套——超过 max_recursion_depth(100) 的边界附近
    let depth = 90;
    let mut input = String::new();
    for _ in 0..depth {
        input.push_str(".level {\n  width: 10px;\n");
    }
    for _ in 0..depth {
        input.push_str("}\n");
    }
    let mut compiler = Compiler::new();
    let css = compiler.compile(&input).expect("deep nesting should compile");
    assert!(css.matches(".level").count() >= depth);
}

#[test]
fn test_output_scales_linearly() {
    // 输出规模应与规则数线性相关
    let small = {
        let mut c = Compiler::new();
        c.compile(&generate_large_input(500)).unwrap().len()
    };
    let large = {
        let mut c = Compiler::new();
        c.compile(&generate_large_input(1_000)).unwrap().len()
    };
    // 规则数翻倍，输出长度应大致翻倍（允许 ±30% 浮动）
    let ratio = large as f64 / small as f64;
    assert!(
        ratio > 1.4 && ratio < 2.6,
        "output scaling not linear: ratio = {ratio}"
    );
}
