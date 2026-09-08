# 插件钩子设计草案（解析/编译扩展点）

状态：草案（v0.4.0 交付物）。目标是在 v1.0.0 之前冻结扩展点语义，先以进程内 Rust API 落地，再演进到版本化插件系统。

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

### 3.1 函数插件（P0，机制已存在，待暴露）

```rust
pub trait LessFunction: Send + Sync {
    /// 函数名（调用时的标识符）
    fn name(&self) -> &str;
    /// 求值。返回 Expression；错误带行列信息
    fn call(&self, args: &[Expression], position: &Position) -> Result<Expression>;
}
```

- 注册入口：`CompilerOptions.functions: Vec<Box<dyn LessFunction>>`，编译器构建时并入 `FunctionRegistry`。
- 冲突规则：后注册覆盖同名内置函数，并在 `strict` 模式下给出 warning。
- 语义约束：函数必须是纯求值（不允许副作用输出）；需要读环境的（如 `env()`）由插件自行捕获上下文。
- source map 透明性：函数返回值沿用调用点 `position`，无需额外映射。

### 3.2 解析钩子（P1）

```rust
pub trait ParseHook: Send + Sync {
    /// 自定义 at-rule 名称（如 "plugin-banner"）；返回 None 表示不接管
    fn at_rule_names(&self) -> &[&str];
    /// 将自定义 at-rule 的原始内容解析为语句序列
    fn parse_at_rule(&self, name: &str, prelude: Option<&str>, body: &str, position: &Position)
        -> Result<Vec<Statement>>;
}
```

- 挂载点：`Parser::parse_statement` 的 `AtKeyword` 分支——未知 at-rule 先查询钩子表，未命中走现有通用路径。
- 约束：钩子只做文本→AST 转换；变量/选择器求值仍在编译期。

### 3.3 编译期 Visitor（P1）

```rust
pub trait CompileVisitor: Send + Sync {
    /// 在规则发射前访问（可改写选择器/声明）
    fn pre_visit_rule(&self, _rule: &mut Rule) -> Result<()> { Ok(()) }
    /// 在样式表编译完成后访问输出 CSS（后处理器）
    fn post_process(&self, _css: &mut String) -> Result<()> { Ok(()) }
}
```

- 对齐 less.js 的 visitor 与 post-processor 两类插件。
- `post_process` 必须保持 source map 有效：默认仅允许不改动字节布局的变换（如注释注入头部）；改动布局的变换需声明 `invalidates_source_map: bool`，此时编译器拒绝同时输出 source map。

### 3.4 导入解析钩子（P2）

```rust
pub trait ImportResolver: Send + Sync {
    /// 将 @import 路径解析为 (规范路径, 内容)；返回 None 移交下一个解析器
    fn resolve(&self, specifier: &str, from_file: &str) -> Result<Option<(String, String)>>;
}
```

- 链式调用：插件解析器 → include_paths → 默认文件系统。
- 用途：虚拟文件系统、HTTP 导入、包管理器布局。

## 4. 生命周期与顺序

1. 构建期：`CompilerOptions` 收集插件 → 校验（重名、source map 冲突声明）。
2. 解析期：`ParseHook` 表只读。
3. 编译期：函数注册表合入；`CompileVisitor::pre_visit_rule` 在每条规则发射前调用；导入解析链在 `compile_import`。
4. 收尾：`post_process` 按注册顺序依次调用；随后生成 source map。

错误语义：插件错误包装为 `Error::PluginError { plugin, message, position }`，不静默吞掉。

## 5. 版本化与稳定性（通往 v1.0.0）

- 扩展点 trait 集标记 `#[non_exhaustive]`，新增默认方法不破坏实现。
- `Plugin API 版本` 常量 `PLUGIN_API_VERSION: u32`；插件声明 `api_version`，编译器拒绝不兼容主版本。
- 稳定边界：仅 `ast`、`error`、`Position`、`Expression` 的子集进入稳定 API；`Compiler` 内部字段永不暴露。
- 每语义版本发布变更记录；废弃周期 ≥ 2 个次版本。

## 6. 安全与沙箱

- 进程内插件即原生代码：不承诺沙箱；文档明确"插件与编译器同权限"。
- WASM 运行时（v1.0.0 候选）再做能力约束（无文件/网络，除非显式授权）。

## 7. 最小示例（目标形态）

```rust
use rust_less::{CompilerOptions, Expression, Position, Result};

struct EnvFunction;
impl LessFunction for EnvFunction {
    fn name(&self) -> &str { "env" }
    fn call(&self, args: &[Expression], position: &Position) -> Result<Expression> {
        // 读取环境变量并返回字符串值
        # unimplemented!()
    }
}

let options = CompilerOptions::default()
    .with_function(Box::new(EnvFunction));
let css = rust_less::compile_with_options(input, options)?;
```

## 8. 里程碑建议

| 版本 | 内容 |
|---|---|
| v0.4.0 | 本草案冻结；`FunctionRegistry` 公开注册路径打通 |
| v0.5.0 | `LessFunction` + `ImportResolver` 落地，示例插件 |
| v0.6.0 | `ParseHook` + `CompileVisitor` 落地 |
| v1.0.0 | `PLUGIN_API_VERSION` 冻结，插件打包/发现约定 |
