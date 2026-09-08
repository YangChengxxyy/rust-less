# 项目当前状态

**更新日期**: 2026-09-07
**版本**: 0.2.4
**状态**: 🟢 积极开发中

## 测试状态

| 类型 | 通过 | 失败 | 忽略 | 通过率 |
|------|------|------|------|--------|
| 单元测试 | 106 | 0 | 0 | 100% |
| 集成测试 | 246 | 0 | 0 | 100% |
| Doc测试 | 1 | 0 | 0 | 100% |
| **总计** | **353** | **0** | **0** | **100%** |

> 统计口径说明（2026-09-07）：测试数字来自本地执行 `cargo test`（353）；兼容性数字来自 `docs/LESSJS_DIFF_REPORT.json`（`pass=80/fail=0/unsupported=16`）。

## 编译状态

- ✅ `cargo build` - 通过
- ✅ `cargo build --features cli` - 通过
- ✅ `cargo clippy --all-targets --all-features` - 通过（无警告）
- ✅ `cargo test` - 全部通过
- ✅ `cargo test --all-features` - 全部通过
- ✅ `bash tools/status-check/run-status-check.sh` - 门禁脚本已接入（测试 + clippy + less.js strict + strict-mappings 对照）

## 已修复的问题 (2025-01-22)

### WASM 编译错误
- ✅ 修复 `console_error_panic_hook::set_panic_hook` 导入问题
- ✅ 添加 `wasm-bindgen-test` 到 dev-dependencies
- ✅ 将 WASM 测试转换为标准 Rust 测试

### 技术债务与编译修复
- ✅ 修复 `src/compiler/sourcemap.rs` 中的 `sourcemap` 库 API 兼容性问题
- ✅ 移除 `src/extend.rs` 中的临时解析 Hack
- ✅ 修复 `src/compiler/rule.rs` 中的模块访问性问题

## 待处理问题

### 🔴 高优先级

1. ~~源码映射深度对齐~~（✅ v0.4.0 完成：复杂导入链、嵌套 at-rule、跨文件 mixin/DR、prelude 变量求值全部纳入 strict-mappings 门禁，80 pass / 0 fail）

### 🟡 中优先级

2. **性能基线持续维护**
   - 已建立 `docs/PERF_BASELINE.json` 与阈值门禁
   - 持续根据发布节奏更新基线，防止长期漂移

### 🟢 低优先级 (远期功能)

3. **Maps 高级语义补齐** - 嵌套结构/边界行为与 LESS 4.x 对齐
4. **插件系统**
5. **LSP (Language Server Protocol)**

## 功能完成度

| 功能 | 完成度 | 状态 |
|------|--------|------|
| 变量系统 | 100% | ✅ |
| 算术运算 | 100% | ✅ |
| 选择器嵌套 | 100% | ✅ |
| 父选择器引用 | 100% | ✅ |
| 媒体查询嵌套 | 100% | ✅ |
| 混合器系统 | 100% | ✅ |
| 变量插值 | 100% | ✅ |
| @import 解析 | 100% | ✅ |
| 文件系统导入 | 100% | ✅ |
| 颜色函数 | 100% | ✅ |
| 数学函数 | 100% | ✅ |
| :extend() | 100% | ✅ |
| 命名空间 | 100% | ✅ |
| 循环/递归混合器 | 100% | ✅ |
| 字符串函数 | 100% | ✅ |
| Maps | 95% | 🔧 已支持 `map-get`/`map-has-key`/`map-keys`/`map-values`/`map-merge`/`map-deep-merge`/`map-set`/`map-update`/`map-replace`/`map-remove`/`map-deep-remove`，并支持 `each(map, ...)` 迭代 |
| 源码映射 | 95% | 🔧 已接入规则/声明/at-rule，覆盖 imported mixin/keyframes 跨文件归属、`token.name` 一致性与 CLI 外部 `.map` 输出链路，支持 less.js 兼容模式 |

## 版本规划

### v0.2.4 (当前版本)
- [x] 清理技术债务 (extend.rs hack, rule.rs TODO)
- [x] 完善字符串函数
- [x] 实现属性选择器 `i` 标志
- [x] 清理编译警告 (unused imports/variables)
- [x] 性能基准测试（已补跑，存在回归趋势）

### v0.3.0 (1-2 月)
- [x] 最小可用源码映射
- [x] @import 选项完善 (`reference`, `inline`, `optional`, `once`, `multiple`)
- [x] CLI 功能对齐（`--include-path`, `--source-map`）

### v0.4.0 ✅ 已完成
- [x] Maps 基础函数（`map-get`, `map-has-key`, `map-keys`, `map-values`, `map-merge`, `map-deep-merge`, `map-set`, `map-update`, `map-replace`, `map-remove`, `map-deep-remove`）
- [x] Maps 高级能力（`each(map, ...)`、嵌套读写、媒体查询 prelude 求值；less.js 原生语义 strict 门禁全过）
- [x] 完整源码映射（复杂导入链/嵌套 at-rule/跨文件 mixin 与 detached ruleset/prelude 变量求值，strict-mappings 字节级对齐）
- [x] 插件钩子设计（`docs/PLUGIN_HOOKS_DESIGN.md`；`Compiler::register_function` 已落地）

## 下一阶段执行计划 (v0.2.5, 2-4周)

### 1. Source Map 深度对齐
- [ ] 扩展 less.js 原生可比用例（已新增导入链 `keyframes`/`media`/嵌套 at-rule、跨目录 sourceRoot 与同 basename 多目录 source 场景；后续补更复杂多层组合）
- [x] 落地 `mappings` 串严格对齐开关（`--observe-mappings` / `--strict-mappings`）
- [x] 现有 source-map 对照集在 strict mappings 模式收敛（`fail=0`）
- [x] 将 `--strict-mappings --strict` 纳入默认门禁（`tools/status-check/run-status-check.sh`）
- [x] 将 source map 差异分类从 observed 细分为结构差异/编码差异

### 2. Maps 高级语义补齐
- [ ] 继续补齐 less.js 对照中的边界语义（键规范化、冲突优先级、错误文案）
- [x] 增加更复杂嵌套 map 的读写/删除回归（3 层以上路径组合，见 `tests/test_round2.rs` 新增深路径组合用例）
- [x] 梳理“rust-less 扩展能力”与“less.js 原生能力”的产品边界文档（见 `docs/MAPS_PRODUCT_BOUNDARY.md`）

### 3. 质量与兼容性
- [x] 增加 less.js 对照用例（Maps + Source Map）并标注差异清单（已接入 `tools/lessjs-compat/run-lessjs-compat.js` 并完成实编译；当前 `pass=70/observed=0/fail=0/unsupported=14`）
- [x] 新增统一质量门禁脚本：`tools/status-check/run-status-check.sh`
- [x] 建立性能基线与阈值门禁（`docs/PERF_BASELINE.json` + `tools/perf-check/run-perf-check.sh`）
- [x] 阶段验收：`cargo test`、`cargo test --all-features`、`cargo clippy --all-targets --all-features` 全绿

### v1.0.0 (6-12 月)
- [ ] LSP
- [ ] 构建工具插件
- [ ] WASM 标准化发布
- [ ] 插件系统
