# Maps 产品边界（rust-less vs less.js）

本文定义 `rust-less` 在 Maps 能力上的产品边界，用于统一测试门禁、兼容性报告与发布语义。

## 1. 目标

- 明确哪些能力属于 less.js 原生可对齐语义。
- 明确哪些能力属于 rust-less 扩展语义（less.js 不原生支持）。
- 约束对照报告中 `pass/fail/unsupported` 的判定口径。

## 2. 语义分层

### A 类：less.js 原生可对齐语义

判定标准：
- less.js 4.5.1 可直接编译，无需扩展函数。
- 在 `tools/lessjs-compat/run-lessjs-compat.js` 中按 `lessjs-native` 基线执行 `pass/fail` 对照。

当前覆盖：
- `maps-native-unquoted-key`
- `maps-native-quoted-key`
- `maps-native-each-map`（`each(@map, ...)` 迭代，`@value`/`@key`/`@index`）
- `maps-native-each-list`（`each()` 遍历列表变量）
- `maps-native-each-in-rule`（规则体内 `each()` 展开声明）
- `maps-native-map-override`（变量后声明覆盖）
- `maps-native-media-prelude`（@media prelude 中 map 访问求值）
- `maps-native-dup-key-last-wins`（同一字面量内重复键后出现者覆盖，对齐 less.js ruleset 语义）
- `maps-native-unit-negative-keys`（`1px:`/`2em:`/`-1:` 单位与负数键）
- `maps-native-lazy-value`（map 值惰性求值：声明后方可解析，按使用点作用域求值）
- `maps-native-interpolated-key`（`@{key}:` 插值键与 `pre-@{key}`/`@{key}-suffix` 复合键）
- `maps-native-map-as-value-error`（map/ruleset 直接作属性值时报错，对齐 less.js SyntaxError）

门禁策略：
- A 类场景出现差异，计入 `fail`，阻断 strict 门禁。

### B 类：rust-less 扩展语义

判定标准：
- 依赖 rust-less 扩展函数或扩展规则。
- less.js 4.5.1 不原生支持，无法做一比一原生语义对照。

当前覆盖：
- `maps-map-merge-shallow`
- `maps-map-deep-merge-order`
- `maps-map-key-normalization-extension`
- `maps-map-update-missing-error`
- `maps-map-replace-missing-error`
- `maps-map-intermediate-not-map-error`
- `maps-map-variable-key-access`（`@map[@key]` 变量 key 访问；less.js 报 NameError）
- `maps-map-nested-bracket-access`（`@map[a][b]` 链式访问；less.js 报 SyntaxError）

扩展语法（无对照夹具，less.js 4.5.1 解析期即拒绝，rust-less 有意放宽）：
- 浮点键（`1.5: v`；less.js 报 ParseError）
- 百分数键（`50%: v`；less.js 报 ParseError）
- 带引号键访问（`@map["name"]`；less.js 报 ParseError "at-rule options not recognised"）
- 括号内插值键访问（`@map[@{k}]`；less.js 报 ParseError "Unrecognised input"）
- 嵌套 map 字面量（`{ a: { b: v; }; }`；less.js 报 ParseError "Unrecognised input"）
  与嵌套值的扩展函数深路径操作（`map-set`/`map-update`/`map-deep-remove` 等）

门禁策略：
- B 类场景在对照报告中计为 `unsupported`。
- `unsupported` 不计入 `fail`，不阻断 strict 门禁。

## 3. 报告与门禁口径

- 对照报告：`docs/LESSJS_DIFF_REPORT.md` / `docs/LESSJS_DIFF_REPORT.json`
- 严格门禁脚本：`tools/status-check/run-status-check.sh`
- 当前 strict 门禁判定条件：`fail == 0 && blocked == 0`
- `unsupported` 作为产品边界指标保留追踪，不作为原生兼容失败。

## 4. 维护规则

- 新增 Maps 用例时，必须先标注属于 A 类或 B 类。
- A 类必须进入 strict 对照并保证可回归。
- B 类必须在文档中说明扩展目的与与 less.js 的差异点。
- 若 less.js 后续原生支持某项 B 类能力，应迁移到 A 类并切换为 `pass/fail` 门禁。
