# vite-plugin-rust-less

基于 WASM 版 [rust-less](../../README.md) 的 Vite 插件，用 Rust 实现的 LESS 编译器替换 Vite 内置的 less 支持。

## 安装

```bash
npm i -D rust-less-wasm-node vite-plugin-rust-less
```

## 配置

vite.config.js：

```js
import { defineConfig } from "vite";
import rustLess from "vite-plugin-rust-less";

export default defineConfig({
  plugins: [rustLess()],
  // 或带选项：
  // plugins: [rustLess({ compress: true, sourceMap: true })],
});
```

插件以 `enforce: "pre"` 运行，拦截所有 `.less` 模块（含带 query 的 id），返回编译后的 CSS 与 source map。

## 选项

| 选项 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `compress` | `boolean` | `false` | 压缩输出（对应 rust-less `CompilerOptions::compress`） |
| `sourceMap` | `boolean` | `true` | 生成 less→css 的 source map |
| `sourceMapLessjsCompat` | `boolean` | `false` | 生成与 less.js 兼容的 source map 格式（对应 `CompilerOptions::source_map_lessjs_compat`） |

错误处理：LESS 编译错误经插件的 `this.error("[rust-less] <id>: <message>")` 上抛给 Vite。

## 说明 / 限制（v1.0.0）

- WASM 版暂未暴露 `includePaths` 与 Rust 进程内插件 API（自定义函数、解析钩子、visitor、导入解析器）。
- 编译发生在 Node 的 WASM 运行时中，无原生依赖。
