# less.js 兼容性清单（Maps + Source Map）

本文件用于跟踪 `rust-less` 与 `less.js` 在 Maps 与 Source Map 相关语义上的对齐进度。

## 1. 当前已覆盖回归（内部基线）

### Maps
- `map-merge`：浅合并，后参数覆盖前参数同名 key（last wins）。
- `map-deep-merge`：递归深合并；嵌套 map 继续合并；标量与 map 冲突时后者覆盖前者。
- `map-deep-merge` 多参数顺序：从左到右合并，后参数优先。
- `map-remove` / `map-deep-remove`：缺失路径为 no-op，不报错。
- `map-update` / `map-replace`：缺失路径报错；中间路径非 map 报错。
- 同一字面量内重复键 last-wins（括号访问与 `map-*` 扩展读写一律对齐 less.js ruleset 语义）。
- 单位键（`1px:`）与负数键（`-1:`）解析与访问。
- map 值惰性求值（less.js lazy semantics）：声明时存储原始字面量，使用点按当前作用域求值。
- 插值键 `@{key}:` 与复合键 `pre-@{key}` / `@{key}-suffix`。
- map / detached ruleset 直接作属性值报错（对齐 less.js SyntaxError，不再泄漏内部 Debug 文本）。

对应测试：
- `tests/test_maps_compat.rs`
- `tests/test_round2.rs`

### Source Map（跨文件 token lookup）
- imported rule 的声明 token 可定位到 imported 文件。
- imported mixin 展开后的声明 token 可定位到 mixin 定义文件。
- bubbled `@media` / `@supports` 内声明 token 可定位到 imported 文件。
- imported `@keyframes` 内声明 token 可定位到 imported 文件。
- imported `@keyframes` 规则头 token 可定位到 imported 文件。
- `token.name` 在规则头与声明场景保持一致（如 `@keyframes importedFade`、`color`、`opacity`、`-webkit-border-radius`）。

对应测试：
- `tests/test_imports.rs`
- `tests/test_phase3_phase4.rs`

## 2. 待完成项（与 less.js 的外部对照）

- 继续扩展 less.js 原生可比用例覆盖面（复杂嵌套、更多 at-rule/source-map 组合）。
- 逐条沉淀“扩展语义（unsupported）”与“原生语义（pass/fail）”的边界说明。
- 已支持 `mappings` 对齐开关（`--observe-mappings` / `--strict-mappings`）；`lessjs-compat` 脚本默认仍为观测策略，`status-check` 默认启用 strict-mappings 门禁。

### 最近一次实编译结果（2026-09-09）

- 报告文件：`docs/LESSJS_DIFF_REPORT.md`
- 汇总：`pass=85`, `observed=0`, `fail=0`, `unsupported=16`, `blocked=0`（total=101，strict + strict-mappings）
- 较 2026-02-23 新增 5 个 A 类用例：`maps-native-dup-key-last-wins`、`maps-native-unit-negative-keys`、`maps-native-lazy-value`、`maps-native-interpolated-key`、`maps-native-map-as-value-error`。
- 说明：
  - `unsupported=16`：`map-*` 扩展函数场景（less.js 4.5.1 不原生支持）+ 2 个扩展访问语义场景（`map-variable-key-access`、`map-nested-bracket-access`，less.js 对变量 key 与链式括号访问均报错）。

### 历史实编译结果（2026-02-23）

- 汇总：`pass=80`, `observed=0`, `fail=0`, `unsupported=16`, `blocked=0`
- 说明：
  - `fail=0`：less.js 原生可比场景全部通过，新增 A 类用例：`maps-native-each-map`、`maps-native-each-list`、`maps-native-each-in-rule`、`maps-native-map-override`、`maps-native-media-prelude`。
  - source map 新增深度场景（strict-mappings 全过）：三级导入链混合 at-rule（`sourcemap-triple-chain-mixed-atrule`）、跨文件 detached ruleset（`sourcemap-detached-ruleset-import`）、media prelude 变量求值（`sourcemap-media-prelude-var`）及各 sourceRoot 变体。

### `mappings` 严格模式观测（2026-02-23）

- 命令：`node tools/lessjs-compat/run-lessjs-compat.js --strict-mappings --strict`
- 结果：`pass=80`, `fail=0`, `unsupported=16`, `blocked=0`
- 结论：全部 source-map 原生可比用例（含三级导入链、跨文件 mixin/DR、prelude 变量求值）在 strict mappings 模式下字节级收敛。

### `unsupported` 语义分层（2026-02-21）

- A 类（长期扩展能力，less.js 不原生支持）：
  - `map-set`
  - `map-merge`
  - `map-deep-merge`
  - `map-update`
  - `map-replace`
  - `map-remove`
  - `map-deep-remove`
- B 类（可对齐原生语义，已纳入 pass/fail 对照）：
  - `maps-native-unquoted-key`
  - `maps-native-quoted-key`
- 当前 `unsupported=14` 全部属于 A 类，后续持续维护为“扩展能力清单”，不作为 less.js 原生失败统计。

### 执行脚本（已接入）

```bash
npm install --prefix tools/lessjs-compat
node tools/lessjs-compat/run-lessjs-compat.js
```

`mappings` 对齐相关可选执行：

```bash
# 仅观测 mappings hash 差异（不失败）
node tools/lessjs-compat/run-lessjs-compat.js --case sourcemap --observe-mappings

# 将 mappings hash 差异升级为失败（建议与 --strict 一起用于 CI）
node tools/lessjs-compat/run-lessjs-compat.js --case sourcemap --strict-mappings --strict
```

输出文件：
- `docs/LESSJS_DIFF_REPORT.md`
- `docs/LESSJS_DIFF_REPORT.json`

## 3. 最近更新

- 已完成 Maps 高级语义 LESS 4.x 边界对齐（2026-09-09）：
  - 重复键 last-wins：括号访问与 `map-get`/`map-has-key`/`map-set`/`map-update`/`map-replace`/`map-remove` 族及 `map-merge` 合并目标一律取最后一个生效项。
  - 字面值解析：支持单位键（`1px:`）、负数键（`-1:`）；百分数键作为有意放宽的扩展语法保留。
  - map 值惰性求值：声明时存储原始字面量，变量使用点按当前作用域求值（对齐 less.js lazy semantics）。
  - 插值键 `@{key}:` 与复合键 `pre-@{key}` / `@{key}-suffix` 解析与求值。
  - map / detached ruleset 直接作属性值由泄漏 Debug 文本改为语义报错（对齐 less.js SyntaxError）。
  - 新增 A 类对照用例 5 个（strict + strict-mappings 全过）；扩展语法分层补充进 `docs/MAPS_PRODUCT_BOUNDARY.md`。
  - 附带修复：`test_cli_source_map.rs` 临时目录在并发测试下的偶发碰撞（目录名加入测试标签）。
- 已完成 v0.4.0 对齐修复（2026-02-23）：
  - 解析器：变量声明支持空格分隔多值（`@m: 10px 20px`）与逗号分组（`@f: Arial, sans-serif`），修复值泄漏到语句流；修复逗号列表中标识符被错误加引号。
  - at-rule prelude 变量求值：`@media (min-width: @bp)` / `@sizes[tablet]` 现在按 less.js 语义求值后再发射；压缩模式保留 `@media (` 必需空格。
  - `each()` 支持在规则体内展开声明（此前静默丢失）。
  - 嵌套 `@media`/`@supports` 块内的 mixin / detached ruleset / each 调用现在正确包裹当前选择器并按源顺序展开。
  - detached ruleset 调用兼容 map 字面量（`@map()` 展开为声明，对齐 less.js 的"map 即 ruleset"语义）。
  - 颜色输出对齐 less.js：十六进制字面量保留原始写法（`#f00`/`#aabbcc`），计算色一律输出六位（`#666666`）；`rotate/translateX/translateY/scale` 输出不再带引号。
  - source map：map 条目携带 key 位置，mixin/DR 展开声明归属定义文件（`Scope::define_variable_file` / `Declaration::source_file`）；函数调用值增加中间映射段；prelude 求值后映射锚定源文本偏移。
- 插件钩子：新增 `Compiler::register_function` 公开注册路径；设计草案见 `docs/PLUGIN_HOOKS_DESIGN.md`。
- 已新增首批 Maps 兼容性回归集：`tests/test_maps_compat.rs`。
- 已补充 Maps 扩展语义 case（lessjs-compat）：
  - `maps-map-deep-nested-read-write-delete`
  - `maps-map-deep-remove-prune-branch`
  - `maps-map-key-normalization-write-path`
  - `maps-map-update-deep-intermediate-error`
  - `maps-map-set-non-map-argument-error`
  - `maps-map-set-empty-path-error`
  - `maps-map-update-non-map-argument-error`
  - `maps-map-replace-intermediate-not-map-error`
- 已扩展跨文件 Source Map token lookup 场景：
  - imported mixin
  - imported keyframes
  - imported keyframes rule header
  - bubbled at-rule token name 一致性
- 已打通 Source Map 产品化链路：
  - CLI 外部 `.map` sidecar 输出
  - `sourceRoot` 写入
  - CSS `sourceMappingURL` 注释输出策略
- 已接入 less.js 实编译对照脚本：
  - `tools/lessjs-compat/run-lessjs-compat.js`
  - `tests/fixtures/lessjs-compat/*.less`
  - 首版报告已落盘：`docs/LESSJS_DIFF_REPORT.md`
  - 报告已支持 `unsupported` 分类，避免将 less.js 不支持的扩展函数误判为失败
  - 报告已支持 source map 观测差异输出（`names`/`sourceRoot`/可选 `mappings`）
  - 报告脚本已支持 `expectedSourceSuffixes` / `expectedSourceCount` 校验，避免同 basename 多目录场景漏检
- 已修复 `@media` 兼容映射偏移策略：按 less.js 规则追加 header 映射；覆盖 feature 起点、顶层分组分隔（`,` / `and`）和函数调用（如 `calc()`）映射，忽略算术分组括号。
- 已补充 Rust 回归测试：`tests/test_phase3_phase4.rs` 新增 `@media` header 映射规则测试（`screen` / `screen and (...)` / `only screen and (...)` / comma queries / `calc()` 函数 / parenthesized query）。
- 已完成 source map less.js 兼容模式产品化：
  - 新增 CLI 开关：`--source-map-lessjs-compat`
  - 新增编译器开关：`Compiler::set_source_map_lessjs_compat(bool)`
  - 新增库选项：`CompilerOptions.source_map_lessjs_compat`
  - 兼容模式行为：`names=[]`、`sourceRoot` 不输出、`sources` 按 less.js rootpath 规则改写
- 已完成 source map `mappings` 严格对齐开关：
  - `--observe-mappings`：观测 `mappings` hash 差异（非失败）
  - `--strict-mappings`：将 `mappings` hash 差异升级为失败
  - 报告汇总新增 `observedStructural` / `observedEncoding` 细分
- 已补充 source map 原生对照用例（新增 64 个）：
  - `sourcemap-local-rule`
  - `sourcemap-import-same-dir`
  - `sourcemap-import-nested`
  - `sourcemap-media-local`
  - `sourcemap-import-chain-nested-atrule`
  - `sourcemap-import-chain-nested-atrule-source-root`
  - `sourcemap-import-chain-keyframes`
  - `sourcemap-import-chain-keyframes-source-root`
  - `sourcemap-import-chain-media-bubble`
  - `sourcemap-import-chain-media-bubble-source-root`
  - `sourcemap-multi-import-cross-dir`
  - `sourcemap-multi-import-cross-dir-source-root`
  - `sourcemap-import-parent-traversal-source-root`
  - `sourcemap-import-parent-traversal`
  - `sourcemap-parent-multi-import-source-root`
  - `sourcemap-parent-multi-import`
  - `sourcemap-parent-media-bubble-source-root`
  - `sourcemap-parent-media-bubble`
  - `sourcemap-parent-deep-media-supports-source-root`
  - `sourcemap-parent-deep-media-supports`
  - `sourcemap-parent-deep-keyframes-source-root`
  - `sourcemap-parent-deep-keyframes`
  - `sourcemap-parent-mixed-multi-source-root`
  - `sourcemap-parent-mixed-multi`
  - `sourcemap-duplicate-basename-multi-dir`
  - `sourcemap-duplicate-basename-multi-dir-source-root`
  - `sourcemap-duplicate-parent-traversal`
  - `sourcemap-duplicate-parent-traversal-source-root`
  - `sourcemap-duplicate-chain`
  - `sourcemap-duplicate-chain-source-root`
  - `sourcemap-duplicate-chain-keyframes`
  - `sourcemap-duplicate-chain-keyframes-source-root`
  - `sourcemap-duplicate-chain-supports`
  - `sourcemap-duplicate-chain-supports-source-root`
  - `sourcemap-duplicate-chain-media`
  - `sourcemap-duplicate-chain-media-source-root`
  - `sourcemap-duplicate-chain-media-and`
  - `sourcemap-duplicate-chain-media-and-source-root`
  - `sourcemap-parent-deep-media-and`
  - `sourcemap-parent-deep-media-and-source-root`
  - `sourcemap-duplicate-chain-media-only-not`
  - `sourcemap-duplicate-chain-media-only-not-source-root`
  - `sourcemap-parent-deep-media-comma-not`
  - `sourcemap-parent-deep-media-comma-not-source-root`
  - `sourcemap-duplicate-chain-media-not-only-supports`
  - `sourcemap-duplicate-chain-media-not-only-supports-source-root`
  - `sourcemap-parent-deep-media-print-not-only`
  - `sourcemap-parent-deep-media-print-not-only-source-root`
  - `sourcemap-duplicate-chain-media-calc`
  - `sourcemap-duplicate-chain-media-calc-source-root`
  - `sourcemap-parent-deep-media-calc`
  - `sourcemap-parent-deep-media-calc-source-root`
  - `sourcemap-duplicate-chain-media-env-var`
  - `sourcemap-duplicate-chain-media-env-var-source-root`
  - `sourcemap-parent-deep-media-func-mix`
  - `sourcemap-parent-deep-media-func-mix-source-root`
  - `sourcemap-duplicate-chain-media-url-var`
  - `sourcemap-duplicate-chain-media-url-var-source-root`
  - `sourcemap-parent-deep-media-url-var`
  - `sourcemap-parent-deep-media-url-var-source-root`
  - `sourcemap-duplicate-chain-media-orientation-resolution`
  - `sourcemap-duplicate-chain-media-orientation-resolution-source-root`
  - `sourcemap-parent-deep-media-custom-func`
  - `sourcemap-parent-deep-media-custom-func-source-root`

## 4. 后续修复计划（按优先级）

1. ~~继续扩展 less.js 原生 source map 对照覆盖~~（已完成：三级导入链混合 at-rule、跨文件 detached ruleset、media prelude 变量求值、strict-mappings 全过）。
2. ~~持续扩展 strict-mappings 用例覆盖~~（已完成：现有 80 个用例全部 strict 通过）。
3. 维护 Maps 产品边界文档：`docs/MAPS_PRODUCT_BOUNDARY.md`（随用例与实现演进更新）。
4. 插件系统落地：按 `docs/PLUGIN_HOOKS_DESIGN.md` 里程碑推进（函数注册路径 `Compiler::register_function` 已打通）。
