//! 插件扩展包示例：一个 bundle 同时提供自定义函数与输出后处理器。

use rust_less::ast::{Expression, Position};
use rust_less::plugin::{CompileVisitor, LessFunction, PluginBundle};
use rust_less::{Compiler, Result};

struct DoubleFn;

impl LessFunction for DoubleFn {
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
            _ => Err(rust_less::Error::function_error(
                "double",
                "expected one number",
                position.line,
                position.column,
            )),
        }
    }
}

struct HeaderVisitor;

impl CompileVisitor for HeaderVisitor {
    fn name(&self) -> &str {
        "header"
    }
    fn post_process(&self, css: &mut String) -> Result<()> {
        *css = format!("/* generated via rust-less-plugin-demo */\n{css}");
        Ok(())
    }
}

struct DemoBundle;

impl PluginBundle for DemoBundle {
    fn name(&self) -> &str {
        "rust-less-plugin-demo"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn register(self: Box<Self>, compiler: &mut Compiler) -> Result<()> {
        compiler.register_function_plugin(Box::new(DoubleFn))?;
        compiler.register_visitor(Box::new(HeaderVisitor))?;
        Ok(())
    }
}

fn main() {
    let mut compiler = Compiler::new();
    compiler
        .register_plugin_bundle(Box::new(DemoBundle))
        .expect("register bundle");
    let css = compiler
        .compile(".card { width: double(120px); padding: double(8px); }")
        .expect("compile");
    println!("{css}");
}
