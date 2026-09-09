# rust-less-loader

基于 WASM 版 [rust-less](../../README.md) 的 webpack loader，用 Rust 实现的 LESS 编译器替换 less-loader。

## 安装

```bash
npm i -D rust-less-wasm-node rust-less-loader
```

## 配置

webpack.config.js（与 css-loader 串联）：

```js
module.exports = {
  module: {
    rules: [
      {
        test: /\.less$/,
        use: [
          "style-loader",
          "css-loader",
          {
            loader: "rust-less-loader",
            options: {
              compress: false,
              sourceMap: true,
              sourceMapLessjsCompat: true,
            },
          },
        ],
      },
    ],
  },
};
```

## 选项

| 选项 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `compress` | `boolean` | `false` | 压缩输出（对应 rust-less `CompilerOptions::compress`） |
| `sourceMap` | `boolean` | `this.sourceMap`（webpack 的 devtool 设置） | 生成 less→css 的 source map |
| `sourceMapLessjsCompat` | `boolean` | `false` | 生成与 less.js 兼容的 source map 格式（对应 `CompilerOptions::source_map_lessjs_compat`） |

错误处理：LESS 编译错误以 `Error: [rust-less] <resourcePath>: <message>` 形式经 loader callback 上抛给 webpack。

## 说明 / 限制（v1.0.0）

- WASM 版暂未暴露 `includePaths` 与 Rust 进程内插件 API（自定义函数、解析钩子、visitor、导入解析器）。
- 编译发生在 Node 的 WASM 运行时中，无原生依赖。
