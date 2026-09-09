# 插件钩子设计（解析/编译扩展点）

状态：已落地（进程内 Rust API）。四类扩展点（自定义函数、解析钩子、编译期
visitor、导入解析器）均已实现并纳入测试（`tests/test_plugin_system.rs`），
示例见 `examples/plugin_system.rs`。v1.0.0 之前冻结扩展点语义；动态加载与
插件打包/发现约定留待后续版本。

## 1. 目标与非目标

### 目标
- 允许第三方注册自定义函数（如 `env()`、`theme()`），与内置函数同一调用路径。
- 允许在解析与编译阶段挂接扩展点，支持自定义 at-rule 与输出后处理。
- 扩展点行为对 source map、错误报告、压缩模式保持透明。
- 语义对齐 less.js 插件的三类能力：自定义函数、Visitor（AST 遍历）、后处理器（CSS 文本）。

### 非目标（v0.4.0）
- 动态加载（dylib/WASM 插件运行时）。
- JavaScript 插件生态兼容（less.js 插件 API 为 JS 专属）。
- 插件市场/包管理。

## 2. 现状架构与可挂载点

编译流水线：`Lexer → Parser → Stylesheet(AST) → Compiler → CSS + SourceMap`。

现有可复用机制：
- `functions::FunctionRegistry::register(name, fn)` — 已支持注册自定义函数，但未暴露到 `Compiler`/`CompilerOptions`。
- `ast::Visitor` / `Visitable` — AST 遍历接口已存在。
- `Compiler` 内部钩子：`compile_statement` 分发、`compile_import` 解析、输出写入（`write_str`/`add_mapping`）。

## 3. 扩展点定义

### 3.1 函数插件（已实现）

```rust
pub trait LessFunction: Send + Sync {
    /// 函数名（调用时的标识符）
    fn name(&self) -> &str;
    /// 求值。返回 Expression；错误带行列信息
    fn call(&self, args: &[Expression], position: &Position) -> Result<Expression>;
}
```

- 注册入口：`CompilerOptions.functions: Vec<Box<dyn LessFunction>>`（经
  `with_function_plugin` 构建器或同名公共字段），编译器构建时并入
  `FunctionRegistry`；亦可在 `Compiler` 上直接 `register_function_plugin`。
- 冲突规则：后注册覆盖同名内置函数（编译期不告警——当前编译器尚无 strict
  模式，告警通道待 strict 模式引入后补齐）。
- 语义约束：函数必须是纯求值（不允许副作用输出）；需要读环境的（如
  `env()`）由插件自行捕获上下文（`Send + Sync`）。
- source map 透明性：函数返回值沿用调用点 `position`，无需额外映射。
- 错误语义：插件返回的任意错误统一包装为 `Error::PluginError`，不静默吞掉。

### 3.2 解析钩子（已实现）

```rust
pub trait ParseHook: Send + Sync {
    /// 自定义 at-rule 名称（如 "plugin-banner"）；返回 None 表示不接管
    fn at_rule_names(&self) -> &[&str];
    /// 将自定义 at-rule 的原始内容解析为语句序列
    fn parse_at_rule(&self, name: &str, prelude: Option<&str>, body: &str, position: &Position)
        -> Result<Vec<Statement>>;
}
```

- 挂载点：`Parser::parse_statement` 的 `AtKeyword` 分支——`@import`、变量声明、
  detached ruleset 判定之后查询钩子表（精确匹配 `at_rule_names`），未命中走
  现有通用 at-rule 路径。钩子表对导入文件的解析同样生效。
- 约束：钩子只做文本→AST 转换；变量/选择器求值仍在编译期。
- 落地形态：钩子产出的语句序列成为该 at-rule 的块（`AtRule.block`），经
  通用 at-rule 发射路径输出（`@name { ... }`）；钩子返回空序列时不设块，
  保持语句形态（`@name;`）。prelude/body 的完整解释权归钩子。

### 3.3 编译期 Visitor（已实现）

```rust
pub trait CompileVisitor: Send + Sync {
    /// 插件名称，用于错误归因
    fn name(&self) -> &str;
    /// 在规则发射前访问（可改写选择器/声明）
    fn pre_visit_rule(&self, _rule: &mut Rule) -> Result<()> { Ok(()) }
    /// 在样式表编译完成后访问输出 CSS（后处理器）
    fn post_process(&self, _css: &mut String) -> Result<()> { Ok(()) }
    /// 该 visitor 的 post_process 是否破坏 source map 字节布局
    fn invalidates_source_map(&self) -> bool { false }
}
```

- 对齐 less.js 的 visitor 与 post-processor 两类插件。
- `pre_visit_rule` 作用于规则的克隆（不污染混入注册用的原始 AST）。
- `post_process` 必须保持 source map 有效：默认仅允许不改动字节布局的变换
  （如注释注入头部）；改动布局的变换需声明 `invalidates_source_map() -> true`，
  此时注册即被拒绝（source map 已启用时返回 `Error::PluginError`）。

### 3.4 导入解析钩子（已实现）

```rust
pub trait ImportResolver: Send + Sync {
    /// 将 @import 路径解析为 (规范路径, 内容)；返回 None 移交下一个解析器
    fn resolve(&self, specifier: &str, from_file: &str) -> Result<Option<(String, String)>>;
}
```

- 链式调用：插件解析器 → include_paths → 默认文件系统。
- 用途：虚拟文件系统、HTTP 导入、包管理器布局。

## 4. 生命周期与顺序

1. 构建期：`CompilerOptions` 收集插件 → `build()` 时校验（`api_version`、
   source map 冲突声明），失败返回 `Error::PluginError`。
2. 解析期：`ParseHook` 表只读（主输入与所有导入文件共用）。
3. 编译期：函数注册表合入；`CompileVisitor::pre_visit_rule` 在每条规则发射前
   调用；导入解析链在 `compile_import`（插件 → include_paths → 文件系统）。
4. 收尾：`post_process` 按注册顺序依次调用；source map 在编译期间同步生成，
   因此布局破坏型后处理必须在注册期声明并被拒绝。

错误语义：插件错误包装为 `Error::PluginError { plugin, message, line, column }`，
不静默吞掉。

## 5. 版本化与稳定性（通往 v1.0.0）

- 扩展点 trait 的新增方法一律带默认实现，不破坏既有实现。
- `Plugin API 版本` 常量 `PLUGIN_API_VERSION: u32`（当前 `1`）；插件声明
  `api_version`，编译器拒绝不兼容版本。
- 稳定边界：仅 `ast`、`error`、`Position`、`Expression` 的子集进入稳定 API；
  `Compiler` 内部字段永不暴露。
- 每语义版本发布变更记录；废弃周期 ≥ 2 个次版本。

## 6. 安全与沙箱

- 进程内插件即原生代码：不承诺沙箱；文档明确"插件与编译器同权限"。
- WASM 运行时（v1.0.0 候选）再做能力约束（无文件/网络，除非显式授权）。

## 7. 最小示例（当前形态）

```rust
use rust_less::plugin::{LessFunction, PLUGIN_API_VERSION};
use rust_less::{CompilerOptions, Expression, Position, Result};

struct EnvFunction;
impl LessFunction for EnvFunction {
    fn name(&self) -> &str { "env" }
    fn call(&self, args: &[Expression], position: &Position) -> Result<Expression> {
        // 读取环境变量并返回字符串值（完整实现见 examples/plugin_system.rs）
        # Ok(Expression::string(String::new(), position.clone()))
    }
}

let options = CompilerOptions::default()
    .with_function_plugin(Box::new(EnvFunction));
let css = rust_less::compile_with_options(input, options)?;
```

完整可运行示例（四类扩展点）：`cargo run --example plugin_system`。

## 8. 里程碑

| 版本 | 内容 | 状态 |
|---|---|---|
| v0.4.0 | 草案冻结；`FunctionRegistry` 公开注册路径打通（`Compiler::register_function`） | ✅ |
| v0.5.0 | `LessFunction` + `ImportResolver` 落地，示例插件 | ✅ |
| v0.6.0 | `ParseHook` + `CompileVisitor` 落地 | ✅ |
| v1.0.0 | `PLUGIN_API_VERSION` 语义冻结（常量已存在，当前为 1），插件打包/发现约定（`PluginBundle` + `Compiler::register_plugin_bundle`，见 `docs/PLUGIN_PACKAGING.md`） | ✅ 已落地 |
