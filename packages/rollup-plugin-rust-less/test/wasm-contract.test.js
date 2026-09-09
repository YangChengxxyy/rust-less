import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { pathToFileURL } from "node:url";

const pkgDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = resolve(pkgDir, "..", "..");

// index.js 实际调用的 wasm 契约面：
// - 入口函数：compileLessWithOptions(input, options)
// - CompileResult 字段：css / sourceMap / error
const WASM_ENTRY_FN = "compileLessWithOptions";
const WASM_RESULT_FIELDS = ["css", "sourceMap", "error"];

// rust-less-wasm-node 应导出的完整 API（与 build-wasm.sh 生成的 index.d.ts 对齐）
const WASM_EXPORTS = [
  "compileLess",
  "compileLessWithOptions",
  "createCompiler",
  "validateLess",
  "getVersion",
  "getInfo",
  "getErrorDetails",
];

test("index.js calls the documented wasm contract surface", () => {
  const indexJs = readFileSync(join(pkgDir, "index.js"), "utf8");

  // 入口函数：直接调用或注入的 wasm 对象上调用
  assert.match(
    indexJs,
    new RegExp(`wasm\\.${WASM_ENTRY_FN}\\s*\\(`),
    `index.js 必须调用 wasm.${WASM_ENTRY_FN}(input, options)`
  );

  // CompileResult 字段：以 result.<field> 形式解构/读取
  for (const field of WASM_RESULT_FIELDS) {
    assert.match(
      indexJs,
      new RegExp(`result\\.${field}\\b`),
      `index.js 必须读取编译结果字段 result.${field}`
    );
  }
});

// 定位真实 dist（已构建的 pkg-nodejs/ 或已安装的 node_modules）；
// 返回 { kind: 'built' | 'installed', dir } 或 null。
function locateWasmDist() {
  const candidates = [
    // 本仓库 build-wasm.sh 产物
    { kind: "built", dir: join(repoRoot, "pkg-nodejs") },
    // 已安装的 npm 依赖（包根直接含 index.d.ts / index.cjs，无 dist/ 子目录）
    { kind: "installed", dir: join(pkgDir, "node_modules", "rust-less-wasm-node") },
    { kind: "installed", dir: join(repoRoot, "node_modules", "rust-less-wasm-node") },
    // 兼容历史 dist/ 布局
    { kind: "installed", dir: join(pkgDir, "node_modules", "rust-less-wasm-node", "dist") },
    { kind: "installed", dir: join(repoRoot, "node_modules", "rust-less-wasm-node", "dist") },
  ];
  for (const c of candidates) {
    if (existsSync(join(c.dir, "index.d.ts"))) {
      return c;
    }
  }
  return null;
}

test("wasm dist type declarations export the full contract", async (t) => {
  const dist = locateWasmDist();
  if (!dist) {
    console.warn(
      "[wasm-contract] rust-less-wasm-node 未构建/未安装（跳过 dist 契约比对）；" +
        "仅执行了 index.js 静态契约检查。运行 ./build-wasm.sh 后可启用本项。"
    );
    t.skip("rust-less-wasm-node dist not built");
    return;
  }

  const dtsContent = readFileSync(join(dist.dir, "index.d.ts"), "utf8");
  for (const name of WASM_EXPORTS) {
    assert.match(
      dtsContent,
      new RegExp(`\\b${name}\\b`),
      `dist/index.d.ts 必须导出 ${name}（来源: ${dist.dir}）`
    );
  }
});

test("wasm dist runtime exports work and produce the documented result shape", async (t) => {
  const dist = locateWasmDist();
  if (!dist) {
    t.skip("rust-less-wasm-node dist not built");
    return;
  }

  const entry = join(dist.dir, "index.mjs");
  if (!existsSync(entry)) {
    t.skip(`dist 入口 index.mjs 不存在（${dist.dir}）`);
    return;
  }
  const api = await import(pathToFileURL(entry).href);

  // 运行时导出与类型声明一致
  for (const name of WASM_EXPORTS) {
    assert.equal(
      typeof api[name],
      "function",
      `dist 运行时必须导出函数 ${name}（来源: ${dist.dir}）`
    );
  }

  // 真实编译：验证 CompileResult 字段形状与 index.js 读取方式一致
  const result = await api.compileLessWithOptions("@c:#333;.a{color:@c}", {
    compress: false,
    sourceMap: true,
  });
  assert.equal(result.error, undefined, `编译应成功，实际 error: ${result.error}`);
  assert.match(result.css, /\.a/);
  assert.match(result.css, /#333/);
  assert.equal(typeof result.sourceMap, "string", "sourceMap 开启时应为 JSON 字符串");

  const map = JSON.parse(result.sourceMap);
  assert.equal(map.version, 3, "source map 必须为 v3");
});
