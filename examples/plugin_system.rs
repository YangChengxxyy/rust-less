//! 插件系统示例：四类扩展点（docs/PLUGIN_HOOKS_DESIGN.md）
//!
//! 运行：`cargo run --example plugin_system`

use rust_less::ast::{Comment, Declaration, Expression, Position, Rule, Statement};
use rust_less::plugin::{CompileVisitor, ImportResolver, LessFunction, ParseHook};
use rust_less::{compile_with_options, CompilerOptions, Error, Result};

/// 函数插件：`env("VAR")` 读取环境变量为字符串
struct EnvFunction;

impl LessFunction for EnvFunction {
    fn name(&self) -> &str {
        "env"
    }

    fn call(&self, args: &[Expression], position: &Position) -> Result<Expression> {
        let name = match args.first() {
            Some(Expression::String { value, .. }) => value.clone(),
            _ => {
                return Err(Error::function_error(
                    "env",
                    "expected a string argument",
                    position.line,
                    position.column,
                ))
            }
        };
        let value = std::env::var(&name).unwrap_or_default();
        Ok(Expression::string(value, position.clone()))
    }
}

/// 解析钩子：`@banner "text";` → 顶部块注释
struct BannerHook;

impl ParseHook for BannerHook {
    fn name(&self) -> &str {
        "banner-hook"
    }

    fn at_rule_names(&self) -> &[&str] {
        &["banner"]
    }

    fn parse_at_rule(
        &self,
        _name: &str,
        prelude: Option<&str>,
        _body: &str,
        position: &Position,
    ) -> Result<Vec<Statement>> {
        let text = prelude.unwrap_or("").trim_matches('"');
        Ok(vec![Statement::Comment(Comment::block(
            format!(" {} ", text),
            position.clone(),
        ))])
    }
}

/// Visitor：给每条规则注入构建标记声明，并在输出头部加横幅
struct BuildStampVisitor;

impl CompileVisitor for BuildStampVisitor {
    fn name(&self) -> &str {
        "build-stamp"
    }

    fn pre_visit_rule(&self, rule: &mut Rule) -> Result<()> {
        rule.declarations.push(Declaration::new(
            "--built-by".to_string(),
            Expression::string("rust-less-plugin".to_string(), rule.position.clone()),
            rule.position.clone(),
        ));
        Ok(())
    }

    fn post_process(&self, css: &mut String) -> Result<()> {
        css.insert_str(0, "/* built with rust-less plugin system */\n");
        Ok(())
    }
}

/// 导入解析器：`theme:*` 从内置主题表解析，其余移交文件系统
struct ThemeResolver;

impl ImportResolver for ThemeResolver {
    fn name(&self) -> &str {
        "theme-resolver"
    }

    fn resolve(&self, specifier: &str, _from_file: &str) -> Result<Option<(String, String)>> {
        match specifier.strip_prefix("theme:") {
            Some("dark") => Ok(Some((
                "theme://dark.less".to_string(),
                "@bg: #111; @fg: #eee;".to_string(),
            ))),
            Some(_) => Ok(None),
            None => Ok(None),
        }
    }
}

fn main() -> Result<()> {
    std::env::set_var("APP_NAME", "rust-less-demo");

    let options = CompilerOptions::default()
        .with_function_plugin(Box::new(EnvFunction))
        .with_parse_hook(Box::new(BannerHook))
        .with_visitor(Box::new(BuildStampVisitor))
        .with_import_resolver(Box::new(ThemeResolver));

    let less = r#"
@banner "Example: plugin system";
@import "theme:dark";

.app {
    color: @fg;
    background: @bg;

    &::after {
        content: env("APP_NAME");
    }
}
"#;

    let css = compile_with_options(less, options)?;
    println!("{}", css);
    Ok(())
}
