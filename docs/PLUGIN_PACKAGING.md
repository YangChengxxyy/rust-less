# 插件包打包与发现约定（Plugin Packaging）

本文约定 v1.0.0 起插件以 **插件扩展包（`PluginBundle`）** 为发布与注册单位，冻结 `PLUGIN_API_VERSION = 1` 的语义。

## 1. 核心概念

`PluginBundle`（`rust_less::plugin::PluginBundle`）将四类扩展点——自定义函数（`LessFunction`）、at-rule 解析钩子（`ParseHook`）、编译期 visitor（`CompileVisitor`）、导入解析器（`ImportResolver`）——打包为一个具名、带版本的整体：

```rust
pub trait PluginBundle: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn api_version(&self) -> u32 { PLUGIN_API_VERSION }
    fn register(self: Box<Self>, compiler: &mut Compiler) -> Result<()>;
}
```

注册入口为 `Compiler::register_plugin_bundle`：先校验 `api_version` 与编译器 `PLUGIN_API_VERSION` 一致，再由插件包自行注册各扩展点；任何错误统一包装为 `Error::PluginError` 并带上插件包名称，便于归因。

## 2. crate 命名与依赖约定

- 插件 crate 命名：`rust-less-plugin-*`（如 `rust-less-plugin-demo`）。
- 依赖 `rust-less` 时与编译器保持**同主版本**（`rust-less = "1"`）。次版本差异在 API 版本兼容表（§5）范围内允许。

## 3. 入口约定

每个插件 crate 提供统一入口函数：

```rust
pub fn bundle() -> Box<dyn rust_less::plugin::PluginBundle>;
```

## 4. 发现方式：显式组合（v1.0.0）

v1.0.0 的插件发现方式为**显式组合**：用户在 `Cargo.toml` 中添加依赖，然后显式注册：

```rust
use rust_less::Compiler;

let mut compiler = Compiler::new();
compiler.register_plugin_bundle(rust_less_plugin_demo::bundle())?;
```

**动态加载（dylib / 运行时发现 / 注册表）明确不在 v1.0.0 范围内**，理由与非目标见 `docs/PLUGIN_HOOKS_DESIGN.md` 的非目标节。进程内插件与编译器同权限运行，**不提供沙箱**（安全声明见 §7）。

## 5. `PLUGIN_API_VERSION = 1` 语义冻结

自 v1.0.0 起，插件 API 版本 `1` 的语义冻结：

- **兼容变更**（不递增 API 版本）：扩展点 trait（`LessFunction` / `ParseHook` / `CompileVisitor` / `ImportResolver` / `PluginBundle`）新增方法**必须带默认实现**；新增 trait / 注册入口；错误信息措辞调整。
- **破坏性变更**（递增主 API 版本）：删除或改签名既有 trait 方法、改变注册语义、改变错误归因约定。递增主 API 版本时，旧 API 版本**至少保留 2 个次版本**作为迁移期，期间注册旧版本插件返回带明确迁移提示的 `Error::PluginError`。

### 语义版本兼容表

| 编译器版本 | 插件 API 版本 | 插件兼容范围 |
| --- | --- | --- |
| 1.0.x | 1 | `rust-less-plugin-*` 依赖 `rust-less ^1.0` 的全部版本 |
| 1.x.y（次版本） | 1 | 同上；次版本只做兼容变更 |
| 2.0.0 起 | 2（旧 API 1 在迁移期内报错并提示迁移） | 依赖 `rust-less ^2.0` 的插件 |

## 6. 最低示例 crate

```text
rust-less-plugin-demo/
├── Cargo.toml
└── src/
    └── lib.rs
```

`Cargo.toml`：

```toml
[package]
name = "rust-less-plugin-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rust-less = "1"
```

`src/lib.rs`：

```rust
use rust_less::plugin::{CompileVisitor, LessFunction, PluginBundle};
use rust_less::{Compiler, Result};

struct DoubleFn;
impl LessFunction for DoubleFn { /* ... */ }

struct HeaderVisitor;
impl CompileVisitor for HeaderVisitor { /* ... */ }

struct DemoBundle;

impl PluginBundle for DemoBundle {
    fn name(&self) -> &str { "rust-less-plugin-demo" }
    fn version(&self) -> &str { env!("CARGO_PKG_VERSION") }
    fn register(self: Box<Self>, compiler: &mut Compiler) -> Result<()> {
        compiler.register_function_plugin(Box::new(DoubleFn))?;
        compiler.register_visitor(Box::new(HeaderVisitor))?;
        Ok(())
    }
}

pub fn bundle() -> Box<dyn PluginBundle> {
    Box::new(DemoBundle)
}
```

完整可运行示例见仓库 `examples/plugin_bundle.rs`（`cargo run --example plugin_bundle`）。

## 7. 安全声明

插件在**编译器进程内以同权限运行**（可执行任意 Rust 代码、访问文件系统与网络），rust-less **不承诺任何沙箱或权限隔离**。用户注册第三方插件包前应自行审查其代码与依赖，与引入任意 Rust 依赖的风险等级相同。

## 8. 与 less.js 插件能力对照

| 能力 | less.js 插件 | rust-less 插件包 |
| --- | --- | --- |
| 自定义函数 | ✅ | ✅ `LessFunction` |
| visitor（规则改写） | ✅ | ✅ `CompileVisitor::pre_visit_rule` |
| 后处理器 | ✅ | ✅ `CompileVisitor::post_process` |
| 自定义 at-rule 解析 | 部分（visitor 模拟） | ✅ `ParseHook`（解析期原生挂载） |
| 导入解析器 | ❌（需改写 file manager） | ✅ `ImportResolver` |
| 加载方式 | JS `require` / 选项传入 | 显式组合（Rust 依赖 + `register_plugin_bundle`） |
| 沙箱 | ❌ | ❌（同权限，无沙箱） |
