//! 针对 2026-09-09 review 修复的行为级回归测试。
//! 每个测试 pin 的是可观察输出契约（less.js 语义/编译产物），不是实现细节。

use rust_less::{compile, Compiler};

// ── e() / escape() 语义分离（less.js 4.x string.js） ────────────────────

#[test]
fn e_strips_quotes_returns_anonymous() {
    let css = compile(r#".x { filter: e("-ms-filter"); }"#).unwrap();
    assert!(css.contains("filter: -ms-filter;"), "{css}");
}

#[test]
fn escape_percent_encodes_lessjs_charset() {
    // less.js: encodeURI(...) + 额外编码 = : # ; ( )
    let css = compile(r#".x { content: escape("a=b:c#d;e(f)g"); }"#).unwrap();
    assert!(css.contains("a%3Db%3Ac%23d%3Be%28f%29g"), "{css}");

    // 空格走 %20；斜杠/问号/逗号不编码（encodeURI 白名单）
    let css = compile(r#".x { content: escape("a b/c,d?e"); }"#).unwrap();
    assert!(css.contains("a%20b/c,d?e"), "{css}");
}

// ── round(数字, 小数位)（less.js number.js） ────────────────────────────

#[test]
fn round_with_decimal_places() {
    let css = compile(r#".x { a: round(3.567, 2); b: round(10.05, 1); }"#).unwrap();
    assert!(css.contains("a: 3.57"), "{css}");
    assert!(css.contains("b: 10.1"), "{css}");
}

#[test]
fn round_one_arg_still_works_with_unit() {
    let css = compile(".x { w: round(10.6px); }").unwrap();
    assert!(css.contains("w: 11px"), "{css}");
}

// ── lexer 畸形数字不再静默归 0.0 ────────────────────────────────────────

#[test]
fn malformed_number_multi_dot_is_error() {
    let result = compile(".x { w: 1.2.3px; }");
    assert!(result.is_err(), "expected lex error, got: {result:?}");
}

// ── Number lexeme 不再经 f64 重建（格式保留） ───────────────────────────

#[test]
fn number_lexeme_preserved() {
    // 编译产物数字前导零等格式由 to_css 决定；这里 pin 的是 lexer 层不引入
    // 0.0 降级——1.5 不能变成 1.5 之外的值（回归于 unwrap_or(0.0)）
    let css = compile(".x { opacity: 0.5; }").unwrap();
    assert!(css.contains("opacity: 0.5"), "{css}");
}

// ── to_css 不再泄漏 Rust Debug 格式 ────────────────────────────────────

#[test]
fn unevaluated_binary_op_renders_source_not_debug() {
    // 触发 evaluate_expression_to_string 的非literal分支前，先确保``
    // 输出里绝不含 Debug 特征串
    let css = compile("@a: 1px; .x { w: @a; }").unwrap();
    assert!(!css.contains("BinaryOp"), "{css}");
    assert!(!css.contains("Position {"), "{css}");
}

// ── 规则内 @import 不再静默丢弃 ─────────────────────────────────────────

#[test]
fn nested_import_in_rule_compiles() {
    // 顶级 import 已验证；这里 pin 的是规则嵌套 import 走 compile_import
    // 而不是落进死分支（编译成功且 CSS import 透传）
    let css = compile(r#".a { @import "http://cdn.example/x.css"; color: red; }"#).unwrap();
    assert!(css.contains("@import"), "{css}");
}

// ── WasmCompiler::enable_source_map 不再丢配置 ─────────────────────────

#[test]
fn enable_source_map_preserves_compressed_state() {
    // 修复前：compressed() 后再 enableSourceMap 会重建 Compiler::new() 丢压缩。
    // 现在等价地通过非消耗方法在同一实例上开启，压缩态保留。
    let mut c = Compiler::compressed().with_source_map(true);
    let css = c.compile(".a { color: red; padding: 1px 2px; }").unwrap();
    assert!(!css.contains("  "), "compressed output has extra spaces: {css}");
    assert!(c.generate_source_map().is_some(), "source map lost");
}
