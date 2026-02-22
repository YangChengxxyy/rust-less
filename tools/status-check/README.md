# status-check 质量门禁脚本

在仓库根目录执行：

```bash
bash tools/status-check/run-status-check.sh
```

如需附带性能回归门禁：

```bash
bash tools/status-check/run-status-check.sh --with-perf
```

默认已启用 source map `mappings` 严格对齐（`--strict-mappings`），也可显式写出：

```bash
bash tools/status-check/run-status-check.sh --strict-mappings
```

如需仅观测 `mappings` 差异（不作为失败）：

```bash
bash tools/status-check/run-status-check.sh --observe-mappings
```

该脚本会依次执行：

1. `cargo test --quiet`
2. `cargo test --all-features --quiet`
3. `cargo clippy --all-targets --all-features -- -D warnings`
4. `node tools/lessjs-compat/run-lessjs-compat.js --strict --strict-mappings`（默认）
5. 校验 `docs/LESSJS_DIFF_REPORT.json` 中 `summary.fail == 0 && summary.blocked == 0`

任一步失败会返回非零退出码。

- `--with-perf` 模式会额外执行 `tools/perf-check/run-perf-check.sh`。
- `--strict-mappings`：less.js 对照阶段透传 `--strict-mappings`（默认行为）。
- `--observe-mappings`：less.js 对照阶段透传 `--observe-mappings`（不因 mappings hash 差异失败）。
