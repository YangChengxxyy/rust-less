# less.js 对照脚本

在仓库根目录执行：

```bash
npm install --prefix tools/lessjs-compat
npm --prefix tools/lessjs-compat run run
```

输出：
- `docs/LESSJS_DIFF_REPORT.md`
- `docs/LESSJS_DIFF_REPORT.json`

可选参数：

```bash
node tools/lessjs-compat/run-lessjs-compat.js --case sourcemap --strict
node tools/lessjs-compat/run-lessjs-compat.js --case sourcemap --observe-mappings
node tools/lessjs-compat/run-lessjs-compat.js --case sourcemap --strict-mappings --strict
```
