# less.js 实编译对照差异报告

- 生成时间: 2026-09-10T11:30:45.321Z
- Node.js: v24.14.1
- less.js: 4.5.1
- rust-less CLI: /Users/cc/projects/rust-less/target/debug/rust-less
- 选项: strict=on, strictMappings=on, observeMappings=off
- 总用例: 101
- 通过: 85
- 观测差异（非失败）: 0
- 观测差异（结构）: 0
- 观测差异（编码）: 0
- 失败: 0
- 不支持（扩展语义）: 16
- 阻塞: 0

## 用例结果

| case | category | baseline | status | reason |
|---|---|---|---|---|
| maps-map-merge-shallow | maps | extension | unsupported | less.js does not natively support rust-less map extension functions |
| maps-map-deep-merge-order | maps | extension | unsupported | less.js does not natively support rust-less map extension functions |
| maps-map-key-normalization-extension | maps | extension | unsupported | less.js does not natively support rust-less map extension functions |
| maps-native-unquoted-key | maps | lessjs-native | pass | output matched |
| maps-native-quoted-key | maps | lessjs-native | pass | both compilers returned errors |
| maps-map-update-missing-error | maps | extension | unsupported | less.js does not natively support rust-less map extension functions |
| maps-map-replace-missing-error | maps | extension | unsupported | less.js does not natively support rust-less map extension functions |
| maps-map-intermediate-not-map-error | maps | extension | unsupported | less.js does not natively support rust-less map extension functions |
| maps-map-deep-nested-read-write-delete | maps | extension | unsupported | less.js does not natively support rust-less map extension functions |
| maps-map-deep-remove-prune-branch | maps | extension | unsupported | less.js does not natively support rust-less map extension functions |
| maps-map-key-normalization-write-path | maps | extension | unsupported | less.js does not natively support rust-less map extension functions |
| maps-map-update-deep-intermediate-error | maps | extension | unsupported | less.js does not natively support rust-less map extension functions |
| maps-map-set-non-map-argument-error | maps | extension | unsupported | less.js does not natively support rust-less map extension functions |
| maps-map-set-empty-path-error | maps | extension | unsupported | less.js does not natively support rust-less map extension functions |
| maps-map-update-non-map-argument-error | maps | extension | unsupported | less.js does not natively support rust-less map extension functions |
| maps-map-replace-intermediate-not-map-error | maps | extension | unsupported | less.js does not natively support rust-less map extension functions |
| maps-native-each-map | maps | lessjs-native | pass | output matched |
| maps-native-each-list | maps | lessjs-native | pass | output matched |
| maps-native-each-in-rule | maps | lessjs-native | pass | output matched |
| maps-native-map-override | maps | lessjs-native | pass | output matched |
| maps-native-media-prelude | maps | lessjs-native | pass | output matched |
| maps-native-dup-key-last-wins | maps | lessjs-native | pass | output matched |
| maps-native-unit-negative-keys | maps | lessjs-native | pass | output matched |
| maps-native-lazy-value | maps | lessjs-native | pass | output matched |
| maps-native-interpolated-key | maps | lessjs-native | pass | output matched |
| maps-native-map-as-value-error | maps | lessjs-native | pass | both compilers returned errors |
| maps-map-variable-key-access | maps | extension | unsupported | less.js does not natively support rust-less map extension functions |
| maps-map-nested-bracket-access | maps | extension | unsupported | less.js does not natively support rust-less map extension functions |
| sourcemap-imported-keyframes | source-map | lessjs-native | pass | output matched |
| sourcemap-imported-keyframes-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-media-bubble-import | source-map | lessjs-native | pass | output matched |
| sourcemap-supports-bubble-import | source-map | lessjs-native | pass | output matched |
| sourcemap-local-rule | source-map | lessjs-native | pass | output matched |
| sourcemap-import-same-dir | source-map | lessjs-native | pass | output matched |
| sourcemap-import-nested | source-map | lessjs-native | pass | output matched |
| sourcemap-media-local | source-map | lessjs-native | pass | output matched |
| sourcemap-import-chain-nested-atrule | source-map | lessjs-native | pass | output matched |
| sourcemap-import-chain-nested-atrule-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-import-chain-keyframes | source-map | lessjs-native | pass | output matched |
| sourcemap-import-chain-keyframes-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-import-chain-media-bubble | source-map | lessjs-native | pass | output matched |
| sourcemap-import-chain-media-bubble-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-multi-import-cross-dir | source-map | lessjs-native | pass | output matched |
| sourcemap-multi-import-cross-dir-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-import-parent-traversal-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-import-parent-traversal | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-multi-import-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-multi-import | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-media-bubble-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-media-bubble | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-media-supports-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-media-supports | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-keyframes-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-keyframes | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-mixed-multi-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-mixed-multi | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-basename-multi-dir | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-basename-multi-dir-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-parent-traversal | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-parent-traversal-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-keyframes | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-keyframes-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-supports | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-supports-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-media | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-media-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-media-and | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-media-and-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-media-and | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-media-and-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-media-only-not | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-media-only-not-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-media-comma-not | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-media-comma-not-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-media-not-only-supports | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-media-not-only-supports-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-media-print-not-only | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-media-print-not-only-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-media-calc | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-media-calc-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-media-calc | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-media-calc-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-media-env-var | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-media-env-var-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-media-func-mix | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-media-func-mix-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-media-url-var | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-media-url-var-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-media-url-var | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-media-url-var-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-media-orientation-resolution | source-map | lessjs-native | pass | output matched |
| sourcemap-duplicate-chain-media-orientation-resolution-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-media-custom-func | source-map | lessjs-native | pass | output matched |
| sourcemap-parent-deep-media-custom-func-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-triple-chain-mixed-atrule | source-map | lessjs-native | pass | output matched |
| sourcemap-triple-chain-mixed-atrule-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-detached-ruleset-import | source-map | lessjs-native | pass | output matched |
| sourcemap-detached-ruleset-import-source-root | source-map | lessjs-native | pass | output matched |
| sourcemap-media-prelude-var | source-map | lessjs-native | pass | output matched |

## 失败/阻塞/不支持详情

### maps-map-merge-shallow

- 状态: unsupported
- 原因: less.js does not natively support rust-less map extension functions
- rust 命令: `/Users/cc/projects/rust-less/target/debug/rust-less /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat/map-merge.less -o /Users/cc/projects/rust-less/target/lessjs-compat/maps-map-merge-shallow/rust/map-merge.css --compress --include-path /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat`

### maps-map-deep-merge-order

- 状态: unsupported
- 原因: less.js does not natively support rust-less map extension functions
- rust 命令: `/Users/cc/projects/rust-less/target/debug/rust-less /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat/map-deep-merge.less -o /Users/cc/projects/rust-less/target/lessjs-compat/maps-map-deep-merge-order/rust/map-deep-merge.css --compress --include-path /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat`

### maps-map-key-normalization-extension

- 状态: unsupported
- 原因: less.js does not natively support rust-less map extension functions
- rust 命令: `/Users/cc/projects/rust-less/target/debug/rust-less /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat/map-key-normalization.less -o /Users/cc/projects/rust-less/target/lessjs-compat/maps-map-key-normalization-extension/rust/map-key-normalization.css --compress --include-path /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat`

### maps-map-update-missing-error

- 状态: unsupported
- 原因: less.js does not natively support rust-less map extension functions
- rust 命令: `/Users/cc/projects/rust-less/target/debug/rust-less /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat/map-update-missing.less -o /Users/cc/projects/rust-less/target/lessjs-compat/maps-map-update-missing-error/rust/map-update-missing.css --include-path /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat`
- rust 错误: Error: FunctionError { function: "map-update", message: "key 'missing' not found in map", line: 5, column: 8 }

### maps-map-replace-missing-error

- 状态: unsupported
- 原因: less.js does not natively support rust-less map extension functions
- rust 命令: `/Users/cc/projects/rust-less/target/debug/rust-less /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat/map-replace-missing.less -o /Users/cc/projects/rust-less/target/lessjs-compat/maps-map-replace-missing-error/rust/map-replace-missing.css --include-path /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat`
- rust 错误: Error: FunctionError { function: "map-update", message: "key 'missing' not found in map", line: 5, column: 8 }

### maps-map-intermediate-not-map-error

- 状态: unsupported
- 原因: less.js does not natively support rust-less map extension functions
- rust 命令: `/Users/cc/projects/rust-less/target/debug/rust-less /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat/map-intermediate-not-map.less -o /Users/cc/projects/rust-less/target/lessjs-compat/maps-map-intermediate-not-map-error/rust/map-intermediate-not-map.css --include-path /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat`
- rust 错误: Error: FunctionError { function: "map-deep-remove", message: "Intermediate key 'a' is not a map", line: 5, column: 8 }

### maps-map-deep-nested-read-write-delete

- 状态: unsupported
- 原因: less.js does not natively support rust-less map extension functions
- rust 命令: `/Users/cc/projects/rust-less/target/debug/rust-less /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat/map-deep-nested-combo.less -o /Users/cc/projects/rust-less/target/lessjs-compat/maps-map-deep-nested-read-write-delete/rust/map-deep-nested-combo.css --compress --include-path /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat`

### maps-map-deep-remove-prune-branch

- 状态: unsupported
- 原因: less.js does not natively support rust-less map extension functions
- rust 命令: `/Users/cc/projects/rust-less/target/debug/rust-less /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat/map-deep-remove-prune-branch.less -o /Users/cc/projects/rust-less/target/lessjs-compat/maps-map-deep-remove-prune-branch/rust/map-deep-remove-prune-branch.css --compress --include-path /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat`

### maps-map-key-normalization-write-path

- 状态: unsupported
- 原因: less.js does not natively support rust-less map extension functions
- rust 命令: `/Users/cc/projects/rust-less/target/debug/rust-less /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat/map-key-normalization-write.less -o /Users/cc/projects/rust-less/target/lessjs-compat/maps-map-key-normalization-write-path/rust/map-key-normalization-write.css --compress --include-path /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat`

### maps-map-update-deep-intermediate-error

- 状态: unsupported
- 原因: less.js does not natively support rust-less map extension functions
- rust 命令: `/Users/cc/projects/rust-less/target/debug/rust-less /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat/map-update-deep-intermediate.less -o /Users/cc/projects/rust-less/target/lessjs-compat/maps-map-update-deep-intermediate-error/rust/map-update-deep-intermediate.css --include-path /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat`
- rust 错误: Error: FunctionError { function: "map-update", message: "Intermediate key 'b' is not a map", line: 7, column: 8 }

### maps-map-set-non-map-argument-error

- 状态: unsupported
- 原因: less.js does not natively support rust-less map extension functions
- rust 命令: `/Users/cc/projects/rust-less/target/debug/rust-less /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat/map-set-non-map.less -o /Users/cc/projects/rust-less/target/lessjs-compat/maps-map-set-non-map-argument-error/rust/map-set-non-map.css --include-path /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat`
- rust 错误: Error: FunctionError { function: "map-set", message: "First argument must be a map", line: 3, column: 8 }

### maps-map-set-empty-path-error

- 状态: unsupported
- 原因: less.js does not natively support rust-less map extension functions
- rust 命令: `/Users/cc/projects/rust-less/target/debug/rust-less /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat/map-set-empty-path.less -o /Users/cc/projects/rust-less/target/lessjs-compat/maps-map-set-empty-path-error/rust/map-set-empty-path.css --include-path /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat`
- rust 错误: Error: FunctionError { function: "map-set", message: "Expected at least 3 arguments, got 2", line: 5, column: 8 }

### maps-map-update-non-map-argument-error

- 状态: unsupported
- 原因: less.js does not natively support rust-less map extension functions
- rust 命令: `/Users/cc/projects/rust-less/target/debug/rust-less /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat/map-update-non-map.less -o /Users/cc/projects/rust-less/target/lessjs-compat/maps-map-update-non-map-argument-error/rust/map-update-non-map.css --include-path /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat`
- rust 错误: Error: FunctionError { function: "map-update", message: "First argument must be a map", line: 3, column: 8 }

### maps-map-replace-intermediate-not-map-error

- 状态: unsupported
- 原因: less.js does not natively support rust-less map extension functions
- rust 命令: `/Users/cc/projects/rust-less/target/debug/rust-less /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat/map-replace-intermediate-not-map.less -o /Users/cc/projects/rust-less/target/lessjs-compat/maps-map-replace-intermediate-not-map-error/rust/map-replace-intermediate-not-map.css --include-path /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat`
- rust 错误: Error: FunctionError { function: "map-update", message: "Intermediate key 'a' is not a map", line: 5, column: 8 }

### maps-map-variable-key-access

- 状态: unsupported
- 原因: less.js does not natively support rust-less map extension functions
- rust 命令: `/Users/cc/projects/rust-less/target/debug/rust-less /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat/map-variable-key-access.less -o /Users/cc/projects/rust-less/target/lessjs-compat/maps-map-variable-key-access/rust/map-variable-key-access.css --compress --include-path /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat`
- less.js 错误: Name: variable @k not found (map-variable-key-access.less) line 7 col 12

### maps-map-nested-bracket-access

- 状态: unsupported
- 原因: less.js does not natively support rust-less map extension functions
- rust 命令: `/Users/cc/projects/rust-less/target/debug/rust-less /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat/map-nested-bracket-access.less -o /Users/cc/projects/rust-less/target/lessjs-compat/maps-map-nested-bracket-access/rust/map-nested-bracket-access.css --compress --include-path /Users/cc/projects/rust-less/tests/fixtures/lessjs-compat`
- less.js 错误: Syntax: Could not evaluate variable call @themes (map-nested-bracket-access.less) line 6 col 2

## Source Map 观测差异（非失败）

无。

## 结论

less.js 原生可比场景未发现失败；存在扩展语义场景（map-*）为 less.js 不支持，已标记 unsupported。
