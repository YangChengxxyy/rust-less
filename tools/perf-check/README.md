# perf-check 性能回归门禁

## 1. 执行性能门禁

在仓库根目录执行：

```bash
bash tools/perf-check/run-perf-check.sh
```

该脚本会：

1. 运行 `cargo bench --bench compiler_bench`
2. 将 `target/criterion/*/new/estimates.json` 与 `docs/PERF_BASELINE.json` 对比
3. 输出报告到：
   - `target/perf-check/report.json`
   - `target/perf-check/report.md`

若任一基准回归超过阈值（默认 20%），脚本返回非零退出码。

## 2. 更新性能基线

当你确认性能变化是预期行为后，可更新基线：

```bash
node tools/perf-check/check-perf-regression.js --update-baseline
```

可选覆盖默认阈值：

```bash
node tools/perf-check/check-perf-regression.js --update-baseline --default-threshold 15
```
