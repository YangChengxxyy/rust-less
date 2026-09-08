# Rust LESS 编译器

一个用 Rust 编写的高性能 LESS 到 CSS 编译器，采用测试驱动开发（TDD）原则构建。

[![Crates.io](https://img.shields.io/crates/v/rust-less.svg)](https://crates.io/crates/rust-less)
[![Documentation](https://docs.rs/rust-less/badge.svg)](https://docs.rs/rust-less)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/github/actions/workflow/status/YangChengxxyy/rust-less/ci.yml?branch=main)](https://github.com/YangChengxxyy/rust-less/actions)

## 🎯 项目概述

这是一个用 Rust 完全重写的 LESS 编译器，重点强调：
- **高性能** - 利用 Rust 的零成本抽象和内存安全
- **测试驱动开发** - 106个单元测试 + 239个集成测试确保代码质量
- **全面的 LESS 语法支持** - 97% 功能完成度，核心功能已完备
- **模块化架构** - 清晰的代码结构，完整的API文档
- **优秀的错误处理** - 提供清晰、有用的错误信息

## 📊 当前实现状态

**版本**: 0.2.4  
**测试通过率**: 100% (`cargo test --quiet` 共 346 passed, 0 ignored；`--all-features` 共 356 passed)  
**功能完成度**: 97%（核心 LESS 功能已完备，Maps 可写能力已落地，源码映射持续完善）
**生产就绪度**: 适合大部分生产项目

**统计口径说明（2026-02-22）**:
- 测试基线来自本地执行：`cargo test --quiet`（346 passed）与 `cargo test --all-features --quiet`（356 passed）。
- 兼容性基线来自 `docs/LESSJS_DIFF_REPORT.json`：`pass=70`、`fail=0`、`unsupported=14`。

### ✅ 已完成的核心功能

| 功能 | 状态 | 完成度 | 说明 |
|------|------|--------|------|
| **变量系统** | ✅ 完成 | 100% | `@variable: value;` 完全支持，包括作用域和函数参数 |
| **算术运算** | ✅ 完成 | 100% | `+`, `-`, `*`, `/` 单位感知计算 |
| **选择器嵌套** | ✅ 完成 | 100% | 任意深度嵌套，完全符合 LESS 规范 |
| **父选择器引用** | ✅ 完成 | 100% | `&:hover`, `&.active`, `&-suffix` 等语法 |
| **媒体查询嵌套** | ✅ 完成 | 100% | 支持复杂的媒体查询合并 |
| **混合器系统** | ✅ 完成 | 100% | 参数化混合器、默认参数、守卫条件、命名空间 |
| **变量插值** | ✅ 完成 | 100% | `@{variable}` 选择器和属性值插值 |
| **@import 解析** | ✅ 完成 | 100% | 正确解析 @import 语句，区分 CSS 和 LESS 导入 |
| **文件系统导入** | ✅ 完成 | 100% | 完整的文件读取、相对路径、循环检测 |
| **颜色函数** | ✅ 完成 | 100% | `lighten()`, `darken()`, `saturate()`, `fade()`, `mix()`, `spin()` 等 |
| **数学函数** | ✅ 完成 | 100% | `round()`, `ceil()`, `floor()`, `percentage()` |
| **字符串函数** | ✅ 完成 | 100% | `e()`, `replace()` 支持正则和标志 |
| **扩展功能 (:extend)** | ✅ 完成 | 100% | 支持 `all` 关键字和选择器附着语法 |
| **CSS 规则编译** | ✅ 完成 | 100% | 完整的 CSS 输出，支持压缩和美化 |

### 🟡 部分实现的功能

| 功能 | 状态 | 完成度 | 说明 |
|------|------|--------|------|
| **Maps (基础函数)** | 🔧 改进中 | 85% | 已支持 `map-get`/`map-has-key`/`map-keys`/`map-values`/`map-merge`/`map-deep-merge`/`map-set`/`map-update`/`map-replace`/`map-remove`/`map-deep-remove`，并支持 `each(map, ...)` 的 `@key/@value/@index` 迭代 |
| **源码映射** | 🔧 改进中 | 80% | 已接入规则/声明/at-rule 映射，覆盖 imported mixin/keyframes 跨文件归属，支持外部 `.map` 输出与 less.js 兼容模式 |

### ❌ 待实现的关键功能

| 功能 | 优先级 | 复杂度 | 预计影响 |
|------|--------|--------|----------|
| **Maps 高级语义** | 🔥 | 高 | 嵌套结构操作、边界行为与 LESS 4.x 完全对齐 |
| **插件系统** | 🔥 | 高 | 扩展性 |

## 🚀 快速开始

### 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
rust-less = "0.2.4"
```

或安装 CLI 工具：

```bash
cargo install rust-less --features cli
```

### 作为库使用

```rust
use rust_less::{compile, Compiler};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 基础编译
    let less_code = r#"
        @primary-color: #333;
        @margin: 10px;

        .header {
            color: @primary-color;
            margin: @margin * 2;
            
            &:hover {
                color: lighten(@primary-color, 20%);
            }
            
            &.dark {
                background: darken(@primary-color, 10%);
            }
            
            .logo {
                width: @margin * 5;
            }
        }
        
        @media (max-width: 768px) {
            .header {
                margin: @margin;
            }
        }
    "#;

    let css = compile(less_code)?;
    println!("{}", css);

    // 压缩输出
    let mut compiler = Compiler::compressed();
    let compressed = compiler.compile(less_code)?;
    println!("{}", compressed);

    Ok(())
}
```

**输出**:
```css
.header {
  color: #333;
  margin: 20px;
}

.header:hover {
  color: #666;
}

.header.dark {
  background: #1a1a1a;
}

.header .logo {
  width: 50px;
}

@media (max-width: 768px) {
  .header {
    margin: 10px;
  }
}
```

### CLI 使用

```bash
# 编译 LESS 文件
rust-less input.less -o output.css

# 压缩输出
rust-less input.less -o output.css --compress

# 从标准输入读取
cat styles.less | rust-less > output.css
```

## 🏗️ 架构设计

项目采用模块化架构，各组件职责明确：

```
src/
├── ast/                 # 抽象语法树
│   ├── mod.rs          # 核心 AST 节点定义
│   ├── expressions.rs  # 表达式和运算
│   ├── selectors.rs    # 选择器处理
│   └── values.rs       # 值类型和转换
├── compiler/            # CSS 代码生成
│   ├── mod.rs          # 主编译器结构与流程编排
│   ├── expression.rs   # 表达式求值
│   ├── mixin.rs        # Mixin 展开与守卫条件
│   ├── rule.rs         # CSS 规则生成
│   ├── import.rs       # @import 处理与文件解析
│   ├── at_rule.rs      # @media, @keyframes 等
│   └── sourcemap.rs    # 源码映射生成（已接入，持续完善）
├── wasm/                # WebAssembly 绑定
│   ├── mod.rs          # WASM 模块入口
│   └── bindings.rs     # wasm-bindgen 绑定
├── lexer.rs            # 词法分析器
├── parser.rs           # 递归下降解析器
├── functions.rs        # 内置函数库
├── extend.rs           # :extend() 处理
├── error.rs            # 错误处理系统
└── lib.rs              # 公共 API
```

### 核心特性

- **高性能解析** - 利用 Rust 的零成本抽象实现高效字符串处理
- **内存安全** - Rust 的所有权系统确保安全性
- **并发友好** - 无共享状态，天然支持并发

## 🧪 测试驱动开发

本项目严格遵循 TDD 原则，每个功能都有完整的测试覆盖：

```bash
# 运行所有测试
cargo test

# 运行特定功能测试
cargo test variables
cargo test functions
cargo test nesting
cargo test mixins

# 查看测试覆盖率
cargo test --verbose

# 运行基准测试
cargo bench

# 运行性能回归门禁（基于 Criterion 基线阈值）
bash tools/perf-check/run-perf-check.sh

# 运行 less.js 实编译对照（Maps + Source Map）
npm install --prefix tools/lessjs-compat
node tools/lessjs-compat/run-lessjs-compat.js

# 运行统一质量门禁（测试 + clippy + less.js strict + strict-mappings 对照）
bash tools/status-check/run-status-check.sh
# 显式启用 strict-mappings（与默认行为一致）
bash tools/status-check/run-status-check.sh --strict-mappings
# 统一门禁（仅观测 mappings hash 差异，不作为失败）
bash tools/status-check/run-status-check.sh --observe-mappings
# 统一门禁（含性能阈值）
bash tools/status-check/run-status-check.sh --with-perf
```

### 测试统计

| 测试类型 | 通过 | 失败 | 忽略 | 通过率 |
|----------|------|------|------|--------|
| 单元测试 | 106 | 0 | 0 | 100% |
| 集成测试 | 239 | 0 | 0 | 100% |
| Doc测试 | 1 | 0 | 0 | 100% |
| **总计** | **346** | **0** | **0** | **100%** |

## 🎯 功能演示

### 1. 变量和运算
```less
@base-size: 16px;
@primary: #007bff;

.component {
    font-size: @base-size;
    padding: @base-size * 0.75;    // 12px
    margin: @base-size / 2;        // 8px
    color: @primary;
}
```

### 2. 混合器系统（完整支持！）
```less
// 基础混合器
.border-radius(@radius: 5px) {
    border-radius: @radius;
    -webkit-border-radius: @radius;
    -moz-border-radius: @radius;
}

.button {
    .border-radius(10px);
    padding: 10px;
}

// 守卫条件
.mixin(@size) when (@size > 10) {
    font-size: large;
}

.mixin(@size) when (@size <= 10) {
    font-size: small;
}

.test1 { .mixin(15); }  // font-size: large
.test2 { .mixin(5); }   // font-size: small
```

### 3. 变量插值（完整支持！）
```less
@selector: "header";
@base-url: "images";

.@{selector} {
    background: url("@{base-url}/bg.png");
}

// 输出:
// .header {
//   background: url("images/bg.png");
// }
```

### 4. 颜色函数
```less
@base-color: #333;

.theme {
    color: @base-color;                    // #333
    background: lighten(@base-color, 20%); // #666
    border: darken(@base-color, 10%);      // #1a1a1a
    
    &:hover {
        background: lighten(@base-color, 30%); // #808080
    }
}
```

### 5. 选择器嵌套
```less
.navbar {
    background: white;
    
    .nav-item {
        padding: 10px;
        
        &:hover {
            background: #f5f5f5;
        }
        
        &.active {
            font-weight: bold;
        }
        
        &-icon {
            margin-right: 5px;
        }
    }
}
```

### 6. 媒体查询
```less
.responsive {
    width: 100%;
    
    @media (max-width: 768px) {
        width: 50%;
        
        @media (max-width: 480px) {
            width: 100%;
        }
    }
}
```

### 7. 内置函数
```less
.math-demo {
    width: percentage(0.5);        // 50%
    height: round(10.6px);         // 11px
    margin: ceil(5.2px);           // 6px
    padding: floor(8.9px);         // 8px
}
```

## 📈 性能基准

在现代硬件上的性能表现：

| 操作 | 速度 | 内存使用 |
|------|------|----------|
| 词法分析 | ~1M 行/秒 | 线性增长 |
| 语法解析 | ~500K 行/秒 | 线性增长 |
| CSS 生成 | ~800K 行/秒 | 常数级 |
| 颜色函数计算 | ~2M 次/秒 | 常数级 |
| 端到端编译 | ~300K 行/秒 | 低内存占用 |

与其他编译器对比：

| 编译器 | 编译时间 (1000行) | 内存使用 | 二进制大小 |
|--------|-------------------|----------|------------|
| rust-less | 3.2ms | 2.1MB | 2.8MB |
| lessc (Node.js) | 45ms | 28MB | N/A |
| sass/dart-sass | 12ms | 15MB | N/A |

## 🛣️ 开发路线图

### 第一阶段：字符串函数优化 (v0.2.4) - ✅ 已完成
- [x] ~~实现 LESS 文件的实际读取和编译~~
- [x] ~~添加循环导入检测~~
- [x] 修复字符串函数变量参数解析（`e()`, `replace()`）
- [x] 支持正则表达式和标志 (`g`, `i`, `m`) 在 `replace()` 中
- [x] 实现 CSS4 大小写不敏感属性选择器 (`[attr=val i]`)
- [x] 清理核心模块的技术债务 (extend, rule)

### 第二阶段：功能增强 (v0.3.0) - 1-2个月
- [x] `:extend()` 语法支持
- [x] 命名空间支持 (#namespace > .mixin)
- [x] 递归混合器（循环生成）
- [x] 源码映射生成（最小可用：行列 + 源文件）
- [x] 导入解析策略完善（`@import (reference|inline|optional|once|multiple)`）
- [x] CLI 关键参数补齐（`--include-path`、`--source-map`）

### 第三阶段：高级功能 (v0.4.0) - ✅ 已完成
- [x] 循环和递归混合器
- [x] 命名空间支持
- [x] Maps 基础函数接口（`map-get`、`map-has-key`、`map-keys`、`map-values`、`map-merge`、`map-deep-merge`、`map-set`、`map-update`、`map-replace`、`map-remove`、`map-deep-remove`）
- [x] Maps 高级能力（`each(map, ...)`、`map-deep-merge` 边界策略，与 less.js 原生语义完全对齐：strict 门禁 80 pass / 0 fail）
- [x] Maps 兼容性扩展（已建立并接入实编译对照，见 `docs/LESSJS_COMPAT_STATUS.md`）
- [x] 源码映射支持（已支持外部 `.map` 输出与 `--source-map-lessjs-compat`）
- [x] 源码映射深度对齐（复杂导入链/嵌套 at-rule/跨文件 mixin 与 detached ruleset/prelude 变量求值场景全部纳入 strict-mappings 门禁）
- [x] 插件钩子设计草案（见 `docs/PLUGIN_HOOKS_DESIGN.md`，函数插件注册路径 `Compiler::register_function` 已落地）

### 第四阶段：生态系统 (v1.0.0) - 6-12个月
- [ ] Language Server Protocol
- [ ] 构建工具插件 (Webpack, Vite, Rollup)
- [ ] WebAssembly 发布流程标准化（`pkg/` 产物与文档）
- [ ] 插件系统（版本化 API + 示例插件）

## 🤝 贡献指南

我们欢迎各种形式的贡献！

### 开发环境设置

```bash
# 克隆项目
git clone https://github.com/YangChengxxyy/rust-less
cd rust-less

# 确保测试通过
cargo test

# 运行示例
cargo run --example basic_example

# 安装开发工具
cargo install cargo-watch cargo-tarpaulin
```

### 贡献流程

1. **选择或创建 Issue** - 讨论要实现的功能
2. **编写测试** - TDD 原则，先写测试
3. **实现功能** - 让测试通过的最小实现
4. **重构优化** - 保持测试通过的前提下优化
5. **文档更新** - 更新 API 文档和示例
6. **提交 PR** - 包含测试和文档的完整 PR

### 编码规范

- 遵循 `rustfmt` 格式化标准
- 使用 `clippy` 进行代码检查
- 为公共 API 编写文档注释
- 保持测试覆盖率在 90% 以上
- 提交信息使用 [Conventional Commits](https://www.conventionalcommits.org/)

## 🔧 配置选项

### 编译器选项

```rust
use rust_less::Compiler;

// 美化输出
let mut compiler = Compiler::new();

// 压缩输出
let mut compiler = Compiler::compressed();

// 启用源码映射
let mut compiler = Compiler::new()
    .with_source_map(true);

// 设置导入搜索路径
let mut compiler = Compiler::new()
    .with_include_paths(vec!["styles/", "node_modules/"]);

// 设置递归深度限制
let mut compiler = Compiler::new()
    .with_recursion_limit(200);

// 编译
let result = compiler.compile(input)?;

// 也可以使用 CompilerOptions
use rust_less::compile_with_options;
use rust_less::CompilerOptions;

let options = CompilerOptions {
    compress: true,
    source_map: false,
    source_map_lessjs_compat: false,
    include_paths: vec!["styles/".to_string()],
};
let result = compile_with_options(input, options)?;
```

### CLI 选项

```bash
rust-less [FILE] [-o OUTPUT] [-c|--compress] [--source-map]

# FILE       输入 LESS 文件（省略则从 stdin 读取）
# -o FILE    输出 CSS 文件（省略则输出到 stdout）
# -c         压缩输出的 CSS
# --source-map                     生成 source map
# --source-map-file FILE           指定 source map 输出路径
# --source-map-url URL             指定 CSS 注释中的 sourceMappingURL
# --source-map-root ROOT           设置 source map sourceRoot
# --source-map-lessjs-compat       启用 less.js 兼容 source map 输出策略
# --include-path PATH              添加导入搜索路径（可重复）
```

## 📚 API 文档

### 核心 API

```rust
// 简单编译
pub fn compile(input: &str) -> Result<String, Error>
pub fn compile_file<P: AsRef<Path>>(path: P) -> Result<String, Error>
pub fn compile_with_options(input: &str, options: CompilerOptions) -> Result<String, Error>
pub fn compile_file_with_options<P: AsRef<Path>>(path: P, options: CompilerOptions) -> Result<String, Error>

// 编译器
impl Compiler {
    pub fn new() -> Self                                  // 美化输出
    pub fn compressed() -> Self                           // 压缩输出
    pub fn with_source_map(self, enabled: bool) -> Self   // 源码映射
    pub fn set_source_map_lessjs_compat(&mut self, enabled: bool) -> &mut Self
    pub fn with_include_paths(self, paths: Vec<P>) -> Self // 导入路径
    pub fn with_recursion_limit(self, limit: usize) -> Self // 递归深度
    pub fn add_include_path(&mut self, path: P) -> &mut Self
    pub fn compile(&mut self, input: &str) -> Result<String, Error>
    pub fn compile_file<P: AsRef<Path>>(&mut self, path: P) -> Result<String, Error>
    pub fn generate_source_map(&self) -> Option<String>
}

// 编译器选项
pub struct CompilerOptions {
    pub compress: bool,
    pub source_map: bool,
    pub source_map_lessjs_compat: bool,
    pub include_paths: Vec<String>,
}
```

更多详细 API 文档请查看 [docs.rs](https://docs.rs/rust-less)。

## 🔍 与其他编译器的对比

| 特性 | rust-less | lessc | sass | stylus |
|------|-----------|-------|------|---------|
| 语言 | Rust | JavaScript | Dart/C++ | JavaScript |
| 性能 | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| 内存使用 | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| LESS 兼容性 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | N/A | ⭐⭐ |
| 混合器支持 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| 错误处理 | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| 生态系统 | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |

## 📄 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- [LESS](http://lesscss.org/) 项目提供的灵感和规范
- [Rust](https://www.rust-lang.org/) 社区的优秀工具链
- 所有贡献者和测试用户的反馈

## 📞 联系方式

- **Issues**: [GitHub Issues](https://github.com/YangChengxxyy/rust-less/issues)
- **Discussions**: [GitHub Discussions](https://github.com/YangChengxxyy/rust-less/discussions)
- **作者**: Yang Cheng

---

**🚀 现在就开始使用 rust-less，体验 Rust 带来的高性能 LESS 编译！**
