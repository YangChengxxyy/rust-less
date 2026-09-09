# rust-less 构建工具插件

本目录收录基于 WASM 版 rust-less（npm 包 `rust-less-wasm-node`）的构建工具集成：

| 包 | 用途 |
|---|---|
| [`rust-less-loader`](./rust-less-loader/) | webpack loader（webpack ≥ 5） |
| [`vite-plugin-rust-less`](./vite-plugin-rust-less/) | Vite 插件（Vite ≥ 4） |
| [`rollup-plugin-rust-less`](./rollup-plugin-rust-less/) | Rollup 插件（Rollup ≥ 3） |

三个包共享同一套选项（`compress` / `sourceMap` / `sourceMapLessjsCompat`，映射到
rust-less `CompilerOptions` 的对应字段），详见 [docs/BUILD_TOOL_PLUGINS.md](../docs/BUILD_TOOL_PLUGINS.md)
与各包 README。各包测试不依赖真实 wasm 包安装（通过工厂注入桩实现）。

每个包另含 `test/wasm-contract.test.js`：静态校验 index.js 调用的 wasm 契约面
（入口函数 `compileLessWithOptions` 与结果字段 `css` / `sourceMap` / `error`）。
若本机已构建/安装 `rust-less-wasm-node` 的 dist（`dist/index.d.ts`），会进一步比对
dist 导出名；dist 未构建时该比对跳过并输出 `console.warn`，不算失败。
