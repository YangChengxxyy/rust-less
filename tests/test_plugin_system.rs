//! 插件系统集成测试（docs/PLUGIN_HOOKS_DESIGN.md）
//!
//! 覆盖四类扩展点：LessFunction、ParseHook、CompileVisitor、ImportResolver，
//! 以及 API 版本校验、错误包装、source map 冲突声明。

use rust_less::ast::{Comment, Declaration, Expression, Position, Rule, Statement};
use rust_less::plugin::{CompileVisitor, ImportResolver, LessFunction, ParseHook};
use rust_less::{compile_with_options, Compiler, CompilerOptions, Error, Result};

// ---------------------------------------------------------------------------
// 测试插件实现
// ---------------------------------------------------------------------------

/// `double(<number>)` — 数值翻倍的自定义函数
struct DoubleFunction;

impl LessFunction for DoubleFunction {
    fn name(&self) -> &str {
        "double"
    }

    fn call(&self, args: &[Expression], position: &Position) -> Result<Expression> {
        match args.first() {
            Some(Expression::Number { value, unit, .. }) => Ok(Expression::Number {
                value: value * 2.0,
                unit: unit.clone(),
                position: position.clone(),
            }),
            _ => Err(Error::function_error(
                "double",
                "expected one number argument",
                position.line,
                position.column,
            )),
        }
    }
}

/// 覆盖内置 `lighten`：始终返回固定颜色
struct FixedLighten;

impl LessFunction for FixedLighten {
    fn name(&self) -> &str {
        "lighten"
    }

    fn call(&self, _args: &[Expression], position: &Position) -> Result<Expression> {
        Ok(Expression::identifier(
            "#c0ffee".to_string(),
            position.clone(),
        ))
    }
}

/// 总是报错的函数插件（验证错误包装）
struct FailingFunction;

impl LessFunction for FailingFunction {
    fn name(&self) -> &str {
        "boom"
    }

    fn call(&self, _args: &[Expression], position: &Position) -> Result<Expression> {
        Err(Error::parse_error("kaboom", position.line, position.column))
    }
}

/// 声明不兼容 API 版本的函数插件
struct OutdatedFunction;

impl LessFunction for OutdatedFunction {
    fn name(&self) -> &str {
        "outdated"
    }

    fn call(&self, _args: &[Expression], position: &Position) -> Result<Expression> {
        Ok(Expression::null(position.clone()))
    }

    fn api_version(&self) -> u32 {
        999
    }
}

/// `@banner "text";` → 块注释；`@banner { ... }` → 原始文本块规则
struct BannerParseHook;

impl ParseHook for BannerParseHook {
    fn name(&self) -> &str {
        "banner-hook"
    }

    fn at_rule_names(&self) -> &[&str] {
        &["banner"]
    }

    fn parse_at_rule(
        &self,
        name: &str,
        prelude: Option<&str>,
        body: &str,
        position: &Position,
    ) -> Result<Vec<Statement>> {
        assert_eq!(name, "banner");
        if body.is_empty() {
            // 语句形态：把 prelude 转成块注释
            let text = prelude.unwrap_or("").trim_matches('"').to_string();
            Ok(vec![Statement::Comment(Comment::block(
                format!(" banner: {} ", text),
                position.clone(),
            ))])
        } else {
            // 块形态：原始文本重新解析为规则
            let mut parser = rust_less::parser::Parser::from_string(body.to_string())?;
            Ok(parser.parse()?.statements)
        }
    }
}

/// 给每条规则注入 `-x-plugin: 1` 声明
struct InjectDeclarationVisitor;

impl CompileVisitor for InjectDeclarationVisitor {
    fn name(&self) -> &str {
        "inject-declaration"
    }

    fn pre_visit_rule(&self, rule: &mut Rule) -> Result<()> {
        rule.declarations.push(Declaration::new(
            "-x-plugin".to_string(),
            Expression::number(1.0, rule.position.clone()),
            rule.position.clone(),
        ));
        Ok(())
    }
}

/// 在输出头部注入横幅注释
struct HeaderPostProcessor;

impl CompileVisitor for HeaderPostProcessor {
    fn name(&self) -> &str {
        "header-post-processor"
    }

    fn post_process(&self, css: &mut String) -> Result<()> {
        css.insert_str(0, "/* generated with plugin */\n");
        Ok(())
    }
}

/// 破坏 source map 布局的 visitor（声明冲突）
struct LayoutBreakingVisitor;

impl CompileVisitor for LayoutBreakingVisitor {
    fn name(&self) -> &str {
        "layout-breaking"
    }

    fn post_process(&self, css: &mut String) -> Result<()> {
        *css = css.replace(' ', "");
        Ok(())
    }

    fn invalidates_source_map(&self) -> bool {
        true
    }
}

/// 虚拟文件系统导入解析器：`virtual:*` 从内存映射取内容，其余移交
struct VirtualFsResolver;

impl ImportResolver for VirtualFsResolver {
    fn name(&self) -> &str {
        "virtual-fs"
    }

    fn resolve(&self, specifier: &str, _from_file: &str) -> Result<Option<(String, String)>> {
        if let Some(name) = specifier.strip_prefix("virtual:") {
            let content = match name {
                "theme" => "@brand: #123456; .theme-mixin() { border: 1px solid @brand; }",
                _ => {
                    return Err(Error::import_error(
                        specifier,
                        "virtual file not found",
                        0,
                        0,
                    ))
                }
            };
            Ok(Some((
                format!("virtual://{}.less", name),
                content.to_string(),
            )))
        } else {
            Ok(None)
        }
    }
}

// ---------------------------------------------------------------------------
// LessFunction（§3.1）
// ---------------------------------------------------------------------------

#[test]
fn test_function_plugin_via_compiler_options() {
    let options = CompilerOptions::default().with_function_plugin(Box::new(DoubleFunction));
    let css = compile_with_options(".x { width: double(21px); }", options).unwrap();
    assert!(css.contains("width: 42px"), "Got: {}", css);
}

#[test]
fn test_function_plugin_via_compiler_register() {
    let mut compiler = Compiler::new();
    compiler
        .register_function_plugin(Box::new(DoubleFunction))
        .unwrap();
    let css = compiler.compile(".x { width: double(5px); }").unwrap();
    assert!(css.contains("width: 10px"), "Got: {}", css);
}

#[test]
fn test_function_plugin_overrides_builtin() {
    let mut compiler = Compiler::new();
    compiler
        .register_function_plugin(Box::new(FixedLighten))
        .unwrap();
    let css = compiler
        .compile(".x { color: lighten(#000000, 10%); }")
        .unwrap();
    assert!(css.contains("#c0ffee"), "Got: {}", css);
}

#[test]
fn test_function_plugin_error_wrapped_as_plugin_error() {
    let mut compiler = Compiler::new();
    compiler
        .register_function_plugin(Box::new(FailingFunction))
        .unwrap();
    let err = compiler.compile(".x { width: boom(1); }").unwrap_err();
    match err {
        Error::PluginError {
            plugin, message, ..
        } => {
            assert_eq!(plugin, "boom");
            assert!(message.contains("kaboom"), "Got: {}", message);
        }
        other => panic!("expected PluginError, got: {:?}", other),
    }
}

#[test]
fn test_plugin_api_version_mismatch_rejected() {
    let mut compiler = Compiler::new();
    let err = compiler
        .register_function_plugin(Box::new(OutdatedFunction))
        .unwrap_err();
    match err {
        Error::PluginError {
            plugin, message, ..
        } => {
            assert_eq!(plugin, "outdated");
            assert!(message.contains("999"), "Got: {}", message);
        }
        other => panic!("expected PluginError, got: {:?}", other),
    }

    // CompilerOptions::build 同样拒绝
    let options = CompilerOptions::default().with_function_plugin(Box::new(OutdatedFunction));
    assert!(matches!(options.build(), Err(Error::PluginError { .. })));
}

// ---------------------------------------------------------------------------
// ParseHook（§3.2）
// ---------------------------------------------------------------------------

#[test]
fn test_parse_hook_statement_form() {
    let options = CompilerOptions::default().with_parse_hook(Box::new(BannerParseHook));
    let css = compile_with_options("@banner \"hello world\";", options).unwrap();
    assert!(css.contains("/* banner: hello world */"), "Got: {}", css);
}

#[test]
fn test_parse_hook_block_form() {
    let options = CompilerOptions::default().with_parse_hook(Box::new(BannerParseHook));
    let css = compile_with_options("@banner { .promo { color: red; } }", options).unwrap();
    assert!(css.contains(".promo"), "Got: {}", css);
    assert!(css.contains("color: red"), "Got: {}", css);
}

#[test]
fn test_parse_hook_leaves_other_at_rules_alone() {
    let options = CompilerOptions::default().with_parse_hook(Box::new(BannerParseHook));
    let css = compile_with_options(
        "@charset \"utf-8\"; @media (min-width: 100px) { .a { color: red; } }",
        options,
    )
    .unwrap();
    assert!(css.contains("@charset"), "Got: {}", css);
    assert!(css.contains("@media"), "Got: {}", css);
}

/// 解析钩子对导入文件同样生效（与 ImportResolver 组合）
#[test]
fn test_parse_hook_applies_to_imported_files() {
    struct ImportWithBanner;

    impl ImportResolver for ImportWithBanner {
        fn name(&self) -> &str {
            "import-with-banner"
        }

        fn resolve(&self, specifier: &str, _from_file: &str) -> Result<Option<(String, String)>> {
            if specifier == "virtual:with-banner" {
                Ok(Some((
                    "virtual://with-banner.less".to_string(),
                    "@banner \"from import\";".to_string(),
                )))
            } else {
                Ok(None)
            }
        }
    }

    let options = CompilerOptions::default()
        .with_parse_hook(Box::new(BannerParseHook))
        .with_import_resolver(Box::new(ImportWithBanner));
    let css = compile_with_options("@import \"virtual:with-banner\";", options).unwrap();
    assert!(css.contains("/* banner: from import */"), "Got: {}", css);
}

// ---------------------------------------------------------------------------
// CompileVisitor（§3.3）
// ---------------------------------------------------------------------------

#[test]
fn test_visitor_pre_visit_rule() {
    let options = CompilerOptions::default().with_visitor(Box::new(InjectDeclarationVisitor));
    let css = compile_with_options(".a { color: red; }", options).unwrap();
    assert!(css.contains("-x-plugin: 1"), "Got: {}", css);
    assert!(css.contains("color: red"), "Got: {}", css);
}

#[test]
fn test_visitor_post_process() {
    let options = CompilerOptions::default().with_visitor(Box::new(HeaderPostProcessor));
    let css = compile_with_options(".a { color: red; }", options).unwrap();
    assert!(
        css.starts_with("/* generated with plugin */"),
        "Got: {}",
        css
    );
}

#[test]
fn test_visitor_invalidating_source_map_conflicts() {
    let options = CompilerOptions {
        source_map: true,
        ..Default::default()
    }
    .with_visitor(Box::new(LayoutBreakingVisitor));
    match options.build() {
        Err(Error::PluginError { plugin, .. }) => assert_eq!(plugin, "layout-breaking"),
        other => panic!("expected PluginError, got: {:?}", other.is_ok()),
    }
}

#[test]
fn test_visitor_without_invalidation_allows_source_map() {
    let options = CompilerOptions {
        source_map: true,
        ..Default::default()
    }
    .with_visitor(Box::new(HeaderPostProcessor));
    let mut compiler = options.build().unwrap();
    let css = compiler.compile(".a { color: red; }").unwrap();
    assert!(css.starts_with("/* generated with plugin */"));
    assert!(compiler.generate_source_map().is_some());
}

// ---------------------------------------------------------------------------
// ImportResolver（§3.4）
// ---------------------------------------------------------------------------

#[test]
fn test_import_resolver_virtual_fs() {
    let options = CompilerOptions::default().with_import_resolver(Box::new(VirtualFsResolver));
    let css = compile_with_options(
        "@import \"virtual:theme\"; .app { color: @brand; .theme-mixin(); }",
        options,
    )
    .unwrap();
    assert!(css.contains("color: #123456"), "Got: {}", css);
    assert!(css.contains("border: 1px solid #123456"), "Got: {}", css);
}

#[test]
fn test_import_resolver_falls_through_to_filesystem() {
    let options = CompilerOptions {
        include_paths: vec![format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR"))],
        ..Default::default()
    }
    .with_import_resolver(Box::new(VirtualFsResolver));
    // `emit.less` 不由插件解析，移交 include_paths / 文件系统
    let css = compile_with_options("@import \"emit\";", options).unwrap();
    assert!(css.contains(".emit"), "Got: {}", css);
    assert!(css.contains("color: blue"), "Got: {}", css);
}

#[test]
fn test_import_resolver_error_wrapped_as_plugin_error() {
    let options = CompilerOptions::default().with_import_resolver(Box::new(VirtualFsResolver));
    let err = compile_with_options("@import \"virtual:missing\";", options).unwrap_err();
    match err {
        Error::PluginError {
            plugin, message, ..
        } => {
            assert_eq!(plugin, "virtual-fs");
            assert!(
                message.contains("virtual file not found"),
                "Got: {}",
                message
            );
        }
        other => panic!("expected PluginError, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 组合：四类扩展点同时工作
// ---------------------------------------------------------------------------

#[test]
fn test_all_extension_points_together() {
    let options = CompilerOptions::default()
        .with_function_plugin(Box::new(DoubleFunction))
        .with_parse_hook(Box::new(BannerParseHook))
        .with_visitor(Box::new(InjectDeclarationVisitor))
        .with_visitor(Box::new(HeaderPostProcessor))
        .with_import_resolver(Box::new(VirtualFsResolver));

    let less = r#"
@import "virtual:theme";
@banner "release";
.app {
    color: @brand;
    width: double(10px);
}
"#;
    let css = compile_with_options(less, options).unwrap();
    assert!(
        css.starts_with("/* generated with plugin */"),
        "Got: {}",
        css
    );
    assert!(css.contains("/* banner: release */"), "Got: {}", css);
    assert!(css.contains("color: #123456"), "Got: {}", css);
    assert!(css.contains("width: 20px"), "Got: {}", css);
    assert!(css.contains("-x-plugin: 1"), "Got: {}", css);
}
