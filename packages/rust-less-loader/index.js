// rust-less-loader —— 基于 WASM 版 rust-less 的 webpack loader。
//
// 通过工厂函数 `createLoader(wasm)` 注入 `rust-less-wasm-node` 的 API 对象，
// 便于在不安装真实包的情况下做单元测试；默认导出则在首次调用时才懒加载
// 真实模块。

/**
 * 解析 loader 选项（含 webpack 约定的 sourceMap 回退）。
 * @param {object} options loader 选项
 * @param {{sourceMap?: boolean}} ctx loader this 上下文
 */
function resolveOptions(options, ctx) {
  const opts = options ?? {};
  return {
    compress: opts.compress ?? false,
    sourceMap: opts.sourceMap ?? ctx.sourceMap ?? false,
    sourceMapLessjsCompat: opts.sourceMapLessjsCompat ?? false,
  };
}

/**
 * 创建 rust-less webpack loader。
 * @param {object} wasm `rust-less-wasm-node` 导出的 API 对象
 *   （需要 `compileLessWithOptions(input, options)`）。
 * @returns {(this: object, source: string, ...) => void} webpack loader 函数
 */
export function createLoader(wasm) {
  return async function rustLessLoader(source) {
    const callback = this.callback;
    try {
      const options = resolveOptions(this.getOptions?.(), this);
      const result = await wasm.compileLessWithOptions(source, {
        compress: options.compress,
        sourceMap: options.sourceMap,
        sourceMapLessjsCompat: options.sourceMapLessjsCompat,
      });
      if (result.error) {
        callback(new Error(`[rust-less] ${this.resourcePath}: ${result.error}`));
        return;
      }
      const map = result.sourceMap ? JSON.parse(result.sourceMap) : null;
      callback(null, result.css, map);
    } catch (err) {
      callback(err instanceof Error ? err : new Error(String(err)));
    }
  };
}

// 懒加载真实 wasm 模块；仅在实际使用 loader 时才 resolve。
let wasmPromise = null;
function loadWasm() {
  if (!wasmPromise) {
    wasmPromise = import("rust-less-wasm-node");
  }
  return wasmPromise;
}

/** 默认导出：webpack 直接引用的 loader。 */
export default async function rustLessLoader(source) {
  const wasm = await loadWasm();
  return createLoader(wasm).call(this, source);
}
