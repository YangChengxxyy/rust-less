// vite-plugin-rust-less —— 基于 WASM 版 rust-less 的 Vite 插件。
//
// `createRustLessPlugin(wasm, options)` 注入 `rust-less-wasm-node` 的 API
// 对象，便于单元测试；默认导出在首次 transform 时才懒加载真实模块。

/**
 * 创建 rust-less Vite 插件。
 * @param {object} wasm `rust-less-wasm-node` 导出的 API 对象
 *   （需要 `compileLessWithOptions(input, options)`）。
 * @param {{compress?: boolean, sourceMap?: boolean, sourceMapLessjsCompat?: boolean}} [options]
 */
export function createRustLessPlugin(wasm, options = {}) {
  return {
    name: "vite-plugin-rust-less",
    enforce: "pre",
    async transform(code, id) {
      const fileId = id.split("?")[0];
      if (!/\.less$/.test(fileId)) {
        return null;
      }
      try {
        const result = await wasm.compileLessWithOptions(code, {
          compress: options.compress ?? false,
          sourceMap: options.sourceMap ?? true,
          sourceMapLessjsCompat: options.sourceMapLessjsCompat ?? false,
        });
        if (result.error) {
          this.error(`[rust-less] ${id}: ${result.error}`);
          return null;
        }
        return {
          code: result.css,
          map: result.sourceMap ? JSON.parse(result.sourceMap) : null,
        };
      } catch (err) {
        this.error(`[rust-less] ${id}: ${err instanceof Error ? err.message : String(err)}`);
        return null;
      }
    },
  };
}

// 懒加载真实 wasm 模块；仅在首次 transform 时 resolve 并缓存。
let wasmPromise = null;
function loadWasm() {
  if (!wasmPromise) {
    wasmPromise = import("rust-less-wasm-node");
  }
  return wasmPromise;
}

/**
 * 默认导出：Vite 配置直接使用的工厂。
 * @param {{compress?: boolean, sourceMap?: boolean, sourceMapLessjsCompat?: boolean}} [options]
 */
export default function rustLessPlugin(options = {}) {
  let plugin = null;
  return {
    name: "vite-plugin-rust-less",
    enforce: "pre",
    async transform(code, id) {
      // 非 less 文件无需触发 wasm 加载
      if (!/\.less$/.test(id.split("?")[0])) {
        return null;
      }
      if (!plugin) {
        const wasm = await loadWasm();
        plugin = createRustLessPlugin(wasm, options);
      }
      return plugin.transform.call(this, code, id);
    },
  };
}
