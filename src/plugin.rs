//! 插件系统：解析 / 编译扩展点
//!
//! 对齐 `docs/PLUGIN_HOOKS_DESIGN.md` 的四类扩展能力（自定义函数、
//! 自定义 at-rule 解析钩子、编译期 visitor、导入解析器），以及 less.js
//! 插件的三类能力（自定义函数、visitor、后处理器）。
//!
//! 进程内 Rust API 落地形态：插件与编译器同权限（不承诺沙箱），
//! 函数插件必须是纯求值。版本化见 [`PLUGIN_API_VERSION`]。

use crate::ast::{Expression, Position, Rule, Statement};
use crate::error::{Error, Result};

/// 插件 API 版本。插件声明的 [`api_version`](LessFunction::api_version)
/// 必须与该常量一致，否则注册时被拒绝。
pub const PLUGIN_API_VERSION: u32 = 1;

/// 自定义函数插件。与内置函数同一调用路径，求值结果沿用调用点
/// `position`，对 source map 透明。
///
/// # 示例
///
/// ```rust
/// use rust_less::ast::{Expression, Position};
/// use rust_less::plugin::LessFunction;
/// use rust_less::{Compiler, Error, Result};
///
/// struct Double;
/// impl LessFunction for Double {
///     fn name(&self) -> &str { "double" }
///     fn call(&self, args: &[Expression], position: &Position) -> Result<Expression> {
///         match args.first() {
///             Some(Expression::Number { value, unit, .. }) => Ok(Expression::Number {
///                 value: value * 2.0,
///                 unit: unit.clone(),
///                 position: position.clone(),
///             }),
///             _ => Err(Error::function_error("double", "expected one number", position.line, position.column)),
///         }
///     }
/// }
///
/// let mut compiler = Compiler::new();
/// compiler.register_function_plugin(Box::new(Double)).unwrap();
/// let css = compiler.compile(".a { width: double(5px); }").unwrap();
/// assert!(css.contains("10px"));
/// ```
pub trait LessFunction: Send + Sync {
    /// 函数名（调用时的标识符）。后注册覆盖同名内置函数。
    fn name(&self) -> &str;
    /// 求值。返回 [`Expression`]；错误带行列信息。
    /// 必须是纯求值（不允许副作用输出）。
    fn call(&self, args: &[Expression], position: &Position) -> Result<Expression>;
    /// 插件声明的 API 版本，默认 [`PLUGIN_API_VERSION`]。
    /// 新增方法一律带默认实现，老插件无需改动。
    fn api_version(&self) -> u32 {
        PLUGIN_API_VERSION
    }
}

/// 自定义 at-rule 解析钩子。只做文本→AST 转换；
/// 变量 / 选择器求值仍在编译期。
///
/// 挂载点：`Parser::parse_statement` 的 `AtKeyword` 分支——精确匹配
/// [`at_rule_names`](ParseHook::at_rule_names) 的规则先走钩子，
/// 未命中走现有通用路径。
pub trait ParseHook: Send + Sync {
    /// 插件名称，用于错误归因。
    fn name(&self) -> &str;
    /// 接管的自定义 at-rule 名称（如 `"banner"` 对应 `@banner`）。
    fn at_rule_names(&self) -> &[&str];
    /// 将自定义 at-rule 的原始内容解析为语句序列。
    ///
    /// * `prelude`：`@name` 与 `{`/`;` 之间的原始文本（无内容时 `None`）
    /// * `body`：花括号块内的原始文本（无块时为空字符串）
    fn parse_at_rule(
        &self,
        name: &str,
        prelude: Option<&str>,
        body: &str,
        position: &Position,
    ) -> Result<Vec<Statement>>;
    /// 插件声明的 API 版本，默认 [`PLUGIN_API_VERSION`]。
    fn api_version(&self) -> u32 {
        PLUGIN_API_VERSION
    }
}

/// 编译期 visitor：规则改写（`pre_visit_rule`）与输出后处理
///（`post_process`），对齐 less.js 的 visitor 与 post-processor。
pub trait CompileVisitor: Send + Sync {
    /// 插件名称，用于错误归因。
    fn name(&self) -> &str;
    /// 在规则发射前访问（可改写选择器 / 声明）。
    fn pre_visit_rule(&self, _rule: &mut Rule) -> Result<()> {
        Ok(())
    }
    /// 在样式表编译完成后访问输出 CSS（后处理器）。
    ///
    /// 默认只允许不改动字节布局的变换；改动布局的变换必须覆写
    /// [`invalidates_source_map`](CompileVisitor::invalidates_source_map)
    /// 返回 `true`，此时编译器拒绝同时输出 source map。
    fn post_process(&self, _css: &mut String) -> Result<()> {
        Ok(())
    }
    /// 该 visitor 的 `post_process` 是否破坏 source map 字节布局。
    fn invalidates_source_map(&self) -> bool {
        false
    }
    /// 插件声明的 API 版本，默认 [`PLUGIN_API_VERSION`]。
    fn api_version(&self) -> u32 {
        PLUGIN_API_VERSION
    }
}

/// 导入解析钩子。链式调用：插件解析器 → `include_paths` → 默认文件系统。
/// 用途：虚拟文件系统、HTTP 导入、包管理器布局。
pub trait ImportResolver: Send + Sync {
    /// 插件名称，用于错误归因。
    fn name(&self) -> &str;
    /// 将 `@import` 路径解析为 `(规范路径, 内容)`；
    /// 返回 `None` 移交下一个解析器。
    ///
    /// * `specifier`：`@import` 中的原始路径（去引号前）
    /// * `from_file`：发起导入的当前源文件路径
    fn resolve(&self, specifier: &str, from_file: &str) -> Result<Option<(String, String)>>;
    /// 插件声明的 API 版本，默认 [`PLUGIN_API_VERSION`]。
    fn api_version(&self) -> u32 {
        PLUGIN_API_VERSION
    }
}

/// 校验插件 API 版本，不兼容时返回 [`Error::PluginError`]。
pub(crate) fn check_api_version(plugin: &str, version: u32) -> std::result::Result<(), Error> {
    if version != PLUGIN_API_VERSION {
        return Err(Error::plugin_error(
            plugin,
            format!(
                "unsupported plugin API version {} (expected {})",
                version, PLUGIN_API_VERSION
            ),
            0,
            0,
        ));
    }
    Ok(())
}

/// 将插件返回的错误包装为 [`Error::PluginError`]（已是该变体则原样保留），
/// 避免静默吞掉插件错误。
pub(crate) fn wrap_plugin_error(plugin: &str, err: Error) -> Error {
    match err {
        Error::PluginError { .. } => err,
        other => {
            let line = other.line().unwrap_or(1);
            let column = other.column().unwrap_or(1);
            Error::plugin_error(plugin, other.message(), line, column)
        }
    }
}
