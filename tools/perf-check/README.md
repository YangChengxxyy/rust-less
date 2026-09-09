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

## 3. 基线溯源（provenance）

`--update-baseline` 写入的基线包含以下溯源字段：

- `generated_at`：生成时刻的完整 ISO 8601 时间戳
- `host`：宿主架构信息 `{ arch, platform }`（来自 `process.arch` / `process.platform`）
- `commit`：生成时对应的 commit SHA——优先取环境变量 `CI_COMMIT`，
  否则执行 `git rev-parse HEAD`；两者都失败时记为 `null`（尽力而为，不阻断）

对比（回归检查）不使用这些字段，它们仅用于追溯基线是在哪台机器、哪个提交上产生的。
仓库中已检入的基线，其溯源字段为事后补录（backfill），时间戳精确到生成日期当天。
