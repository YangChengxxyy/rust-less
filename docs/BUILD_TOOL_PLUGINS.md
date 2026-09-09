# 构建工具插件（webpack / Vite / Rollup）

状态：v1.0.0 已落地。三个 npm 包将 WASM 版 rust-less 接入主流 JS 构建工具：

| 包 | 目标工具 | peer 依赖 |
|---|---|---|
| `rust-less-loader` | webpack | `webpack >= 5` |
| `vite-plugin-rust-less` | Vite | `vite >= 4 \|\| ^5.0.0` |
| `rollup-plugin-rust-less` | Rollup | `rollup >= 3` |

三者均依赖 npm 包 `rust-less-wasm-node`（`dependencies`），即 rust-less 编译为
wasm32-unknown-unknown WASM 后的 Node 包装，暴露异步 API：

```ts
compileLessWithOptions(input, { compress, sourceMap, sourceMapLessjsCompat })
  -> Promise<{ css: string, sourceMap?: string, error?: string }>
```

编译失败时不抛异常而是返回 `{ css: "", error: "<message>" }`；各插件负责把
`error` 字段转成构建工具的错误通道（loader callback / 插件 `this.error`），
并带上 `[rust-less] <文件id>: <消息>` 前缀以便定位。

## 架构：工厂 + 懒加载默认导出

每个包导出两层 API，保证无需安装真实 wasm 包即可测试：

- 命名工厂（如 `createLoader(wasm)`、`createRustLessPlugin(wasm, options)`）：
  显式注入 wasm API 对象，单元测试注入桩实现。
- 默认导出：首次实际使用时才 `await import("rust-less-wasm-node")`（Promise
  缓存，模块只在真正编译 .less 时加载），再委托给工厂产物。

## 选项

映射到 rust-less `CompilerOptions`：

| 选项 | 类型 | webpack loader 默认 | vite/rollup 默认 | CompilerOptions 对应 |
|---|---|---|---|---|
| `compress` | `boolean` | `false` | `false` | `compress`（压缩输出） |
| `sourceMap` | `boolean` | `this.sourceMap ?? false`（跟随 webpack devtool） | `true` | `source_map`（生成 less→css 映射） |
| `sourceMapLessjsCompat` | `boolean` | `false` | `false` | `source_map_lessjs_compat`（less.js 兼容格式） |

## 最小配置

webpack：

```js
module.exports = {
  module: {
    rules: [
      { test: /\.less$/, use: ["style-loader", "css-loader", "rust-less-loader"] },
    ],
  },
};
```

Vite：

```js
import rustLess from "vite-plugin-rust-less";
export default { plugins: [rustLess()] };
```

Rollup：

```js
import rustLess from "rollup-plugin-rust-less";
export default { input: "src/main.js", plugins: [rustLess()] };
```

## FAQ

**Q: source map 的意图是什么？**
生成的是 less→css 的映射，交给下游 loader/plugin（css-loader、postcss、Vite 内置
CSS 管线）继续串联到最终的 bundle map；上游为压缩输出关闭 source map 时该选项
自动失效。`sourceMapLessjsCompat` 用于与期望 less.js map 格式的工具对接。

**Q: 为什么报错不直接抛异常？**
wasm 契约约定 LESS 编译错误以 `error` 字符串返回（不抛异常），由 JS 侧统一
包装为带文件定位的错误并走构建工具的错误通道。

**Q: v1.0.0 有哪些限制？**
WASM 版暂未暴露 `includePaths` 与 Rust 进程内插件 API（自定义函数、解析钩子、
visitor、导入解析器，见 `docs/PLUGIN_HOOKS_DESIGN.md`）。这些能力当前仅原生
Rust 编译器可用。

**Q: 测试需要安装 wasm 包吗？**
不需要。三个包的测试均通过命名工厂注入桩 wasm（node:test），`rust-less-wasm-node`
仅在被默认导出真正使用时才会加载。
