//! 插件扩展包（PluginBundle）端到端测试。

use rust_less::ast::{Expression, Position};
use rust_less::plugin::{CompileVisitor, LessFunction, PluginBundle, PLUGIN_API_VERSION};
use rust_less::{Compiler, Error};

struct DoubleFn;

impl LessFunction for DoubleFn {
    fn name(&self) -> &str {
        "double"
    }
    fn call(&self, args: &[Expression], position: &Position) -> rust_less::Result<Expression> {
        match args.first() {
            Some(Expression::Number { value, unit, .. }) => Ok(Expression::Number {
                value: value * 2.0,
                unit: unit.clone(),
                position: position.clone(),
            }),
            _ => Err(Error::function_error(
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
    fn post_process(&self, css: &mut String) -> rust_less::Result<()> {
        *css = format!("/* processed by demo-bundle */\n{css}");
        Ok(())
    }
}

struct DemoBundle {
    version_override: Option<u32>,
    fail_in_register: bool,
}

impl DemoBundle {
    fn new() -> Box<Self> {
        Box::new(Self {
            version_override: None,
            fail_in_register: false,
        })
    }
}

impl PluginBundle for DemoBundle {
    fn name(&self) -> &str {
        "demo-bundle"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn api_version(&self) -> u32 {
        self.version_override.unwrap_or(PLUGIN_API_VERSION)
    }
    fn register(self: Box<Self>, compiler: &mut Compiler) -> rust_less::Result<()> {
        if self.fail_in_register {
            return Err(Error::function_error("demo-bundle", "boom", 0, 0));
        }
        compiler.register_function_plugin(Box::new(DoubleFn))?;
        compiler.register_visitor(Box::new(HeaderVisitor))?;
        Ok(())
    }
}

#[test]
fn bundle_applies_function_and_post_process() {
    let mut compiler = Compiler::new();
    compiler.register_plugin_bundle(DemoBundle::new()).unwrap();
    let css = compiler.compile(".a { width: double(5px); }").unwrap();
    assert!(css.contains("10px"), "函数插件生效: {css}");
    assert!(css.starts_with("/* processed by demo-bundle */"), "{css}");
}

#[test]
fn incompatible_api_version_is_rejected_with_bundle_name() {
    let mut compiler = Compiler::new();
    let err = compiler
        .register_plugin_bundle(Box::new(DemoBundle {
            version_override: Some(PLUGIN_API_VERSION + 1),
            fail_in_register: false,
        }))
        .unwrap_err();
    match err {
        Error::PluginError { plugin, .. } => assert_eq!(plugin, "demo-bundle"),
        other => panic!("expected PluginError, got {other:?}"),
    }
}

#[test]
fn register_error_is_wrapped_with_bundle_name() {
    let mut compiler = Compiler::new();
    let err = compiler
        .register_plugin_bundle(Box::new(DemoBundle {
            version_override: None,
            fail_in_register: true,
        }))
        .unwrap_err();
    match err {
        Error::PluginError {
            plugin, message, ..
        } => {
            assert_eq!(plugin, "demo-bundle");
            assert!(message.contains("boom"), "{message}");
        }
        other => panic!("expected PluginError, got {other:?}"),
    }
}

struct TripleFn;

impl LessFunction for TripleFn {
    fn name(&self) -> &str {
        "triple"
    }
    fn call(&self, args: &[Expression], position: &Position) -> rust_less::Result<Expression> {
        match args.first() {
            Some(Expression::Number { value, unit, .. }) => Ok(Expression::Number {
                value: value * 3.0,
                unit: unit.clone(),
                position: position.clone(),
            }),
            _ => Err(Error::function_error(
                "triple",
                "expected one number",
                position.line,
                position.column,
            )),
        }
    }
}

struct SecondBundle;

impl PluginBundle for SecondBundle {
    fn name(&self) -> &str {
        "second-bundle"
    }
    fn version(&self) -> &str {
        "0.2.0"
    }
    fn register(self: Box<Self>, compiler: &mut Compiler) -> rust_less::Result<()> {
        compiler.register_function_plugin(Box::new(TripleFn))
    }
}

#[test]
fn two_bundles_both_take_effect() {
    let mut compiler = Compiler::new();
    compiler.register_plugin_bundle(DemoBundle::new()).unwrap();
    compiler
        .register_plugin_bundle(Box::new(SecondBundle))
        .unwrap();
    let css = compiler
        .compile(".a { width: double(5px); height: triple(4px); }")
        .unwrap();
    assert!(css.contains("10px"), "{css}");
    assert!(css.contains("12px"), "{css}");
    assert!(css.starts_with("/* processed by demo-bundle */"), "{css}");
}
