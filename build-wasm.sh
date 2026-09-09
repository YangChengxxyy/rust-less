#!/bin/bash

# rust-less WASM 发布构建脚本（标准化发布流程）
#
# 构建三个 wasm-pack 目标并后处理为可发布的 npm 包：
#   pkg/         -> rust-less-wasm      （浏览器 / bundler，主发布目标）
#   pkg-nodejs/  -> rust-less-wasm-node （Node.js，供构建工具插件使用）
#   pkg-web/     -> rust-less-wasm-web  （直接 <script> 引入的 Web 目标）
#
# 用法:
#   ./build-wasm.sh                    # 完整构建 + 冒烟测试
#   ./build-wasm.sh --skip-build       # 复用已有 pkg*/ 产物，仅后处理 + 冒烟测试
#   ./build-wasm.sh --smoke            # 仅运行冒烟测试
#   ./build-wasm.sh -h                 # 帮助

set -euo pipefail

cd "$(dirname "$0")"

SKIP_BUILD=0
SMOKE_ONLY=0

usage() {
    sed -n '3,15p' "$0" | sed 's/^# \{0,1\}//'
    exit 0
}

for arg in "$@"; do
    case "$arg" in
        --skip-build) SKIP_BUILD=1 ;;
        --smoke)      SMOKE_ONLY=1 ;;
        -h|--help)    usage ;;
        *) echo "❌ 未知参数: $arg（使用 -h 查看帮助）" >&2; exit 1 ;;
    esac
done

# ---------------------------------------------------------------------------
# 工具检查
# ---------------------------------------------------------------------------
check_tool() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "❌ 缺少工具: $1"
        echo "   安装方式: $2"
        exit 1
    fi
}

echo "🔍 检查必要工具..."
check_tool "cargo" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
check_tool "wasm-pack" "cargo install wasm-pack --version 0.13.1"
check_tool "wasm-bindgen" "cargo install wasm-bindgen-cli --version 0.2.100"
check_tool "jq" "brew install jq（或 https://jqlang.github.io/jq/download/）"

# 读取 crate 版本（包版本必须与 crate 版本锁步）
VERSION="$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].version')"
if [ -z "$VERSION" ] || [ "$VERSION" = "null" ]; then
    echo "❌ 无法从 cargo metadata 读取版本号" >&2
    exit 1
fi
echo "📌 版本: ${VERSION}"
export RUST_LESS_VERSION="$VERSION"

if [ "$SMOKE_ONLY" -eq 1 ]; then
    echo "🏃 仅运行冒烟测试（跳过构建与后处理）"
    exec node --input-type=module -e "$(cat <<'SMOKE_EOF'
import { createRequire } from 'node:module';
const require = createRequire(process.cwd() + '/');
const VERSION = process.env.RUST_LESS_VERSION;
const api = require('./pkg-nodejs/index.cjs');

// 1. 基本编译
const r1 = await api.compileLess('@c:#333;.a{color:@c}');
if (!(r1.css.includes('.a') && r1.css.includes('#333'))) throw new Error('基本编译失败: ' + r1.css);
if (r1.error) throw new Error('不应有错误: ' + r1.error);
console.log('✓ compileLess 基本编译');

// 2. 压缩选项
const r2 = await api.compileLessWithOptions('@c:#333;.a{color:@c}', { compress: true });
if (!r2.css.includes('.a{')) throw new Error('压缩输出不符合预期: ' + JSON.stringify(r2.css));
console.log('✓ compileLessWithOptions compress:true');

// 3. source map 选项
const r3 = await api.compileLessWithOptions('@c:#333;.a{color:@c}', { sourceMap: true });
if (typeof r3.sourceMap !== 'string') throw new Error('缺少 sourceMap');
const sm = JSON.parse(r3.sourceMap);
if (sm.version !== 3) throw new Error('source map version !== 3');
console.log('✓ compileLessWithOptions sourceMap:true');

// 4. 错误处理
const r4 = await api.compileLess('.a{color:@nope}');
if (!r4.error || !/nope/i.test(r4.error)) throw new Error('错误信息未提及变量: ' + r4.error);
if (r4.css !== '') throw new Error('失败时 css 应为空');
console.log('✓ 编译错误报告');

// 5. 版本一致
const v = await api.getVersion();
if (v !== VERSION) throw new Error(`getVersion()=${v} !== ${VERSION}`);
console.log('✓ getVersion() === ' + VERSION);

// 6. ESM 入口
const esm = await import('./pkg-nodejs/index.mjs');
const r6 = await esm.compileLess('@c:#333;.a{color:@c}');
if (!(r6.css.includes('.a') && r6.css.includes('#333'))) throw new Error('ESM 入口编译失败');
console.log('✓ index.mjs ESM 入口');

console.log('🎉 冒烟测试全部通过');
SMOKE_EOF
)" || { echo "❌ 冒烟测试失败" >&2; exit 1; }
    exit 0
fi

if [ "$SKIP_BUILD" -eq 0 ]; then
    # 确保 wasm32 目标已安装
    if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
        echo "📥 安装 wasm32-unknown-unknown 目标..."
        rustup target add wasm32-unknown-unknown
    fi

    echo "🧹 清理之前的构建产物..."
    rm -rf pkg/ pkg-nodejs/ pkg-web/

    # 注意: `--features wasm` 必须放在 `--` 之后，否则会被 wasm-pack 误传给 cargo build
    echo "🔨 构建 bundler 目标 (pkg/ -> rust-less-wasm)..."
    wasm-pack build --mode no-install --target bundler --out-dir pkg -- --features wasm

    echo "🔨 构建 nodejs 目标 (pkg-nodejs/ -> rust-less-wasm-node)..."
    wasm-pack build --mode no-install --target nodejs --out-dir pkg-nodejs -- --features wasm

    echo "🔨 构建 web 目标 (pkg-web/ -> rust-less-wasm-web)..."
    wasm-pack build --mode no-install --target web --out-dir pkg-web -- --features wasm
fi

if [ ! -f pkg/rust_less.js ] || [ ! -f pkg-nodejs/rust_less.js ] || [ ! -f pkg-web/rust_less.js ]; then
    echo "❌ 缺少构建产物（请去掉 --skip-build 重新构建）" >&2
    exit 1
fi

# ---------------------------------------------------------------------------
# pkg/ 后处理 —— rust-less-wasm（浏览器 / bundler 主发布目标）
# ---------------------------------------------------------------------------
echo "📦 后处理 pkg/ (rust-less-wasm)..."

cat > pkg/index.js << 'EOF'
/**
 * rust-less-wasm —— Rust LESS 编译器的 WebAssembly 绑定（bundler / 浏览器）
 *
 * bundler 目标在模块导入时自动完成 WASM 初始化，无需手动调用 init()。
 * 所有 API 均为异步（Promise）形式；编译失败不会抛出异常，
 * 而是返回 { css: "", error: "<信息>" }。
 */
import {
    init,
    compile_less,
    compile_less_with_options,
    WasmCompiler,
    WasmCompilerOptions,
    version,
    info,
    validate_less,
    get_error_details,
} from './rust_less.js';

/** 将 wasm-bindgen 的 CompileResult 转换为普通对象（包含 sourceMap） */
function toResult(result) {
    const out = { css: result.css };
    if (typeof result.sourceMap === 'string' && result.sourceMap.length > 0) {
        out.sourceMap = result.sourceMap;
    }
    if (result.error != null) {
        out.error = result.error;
    }
    return out;
}

/** 将普通选项对象映射到 WasmCompilerOptions */
function makeOptions(options) {
    const opts = new WasmCompilerOptions();
    if (options) {
        if (options.compress !== undefined) opts.compress = !!options.compress;
        if (options.sourceMap !== undefined) opts.source_map = !!options.sourceMap;
        if (options.sourceMapLessjsCompat !== undefined) opts.sourceMapLessjsCompat = !!options.sourceMapLessjsCompat;
    }
    return opts;
}

/** 编译 LESS 源码（默认选项） */
export function compileLess(input) {
    return Promise.resolve().then(() => toResult(compile_less(input)));
}

/** 编译 LESS 源码（自定义选项） */
export function compileLessWithOptions(input, options) {
    return Promise.resolve().then(() => {
        const opts = makeOptions(options);
        try {
            return toResult(compile_less_with_options(input, opts));
        } finally {
            opts.free();
        }
    });
}

/** 创建可复用的编译器实例 */
export async function createCompiler(options) {
    const compiler = options?.compress ? WasmCompiler.compressed() : new WasmCompiler();
    if (options?.sourceMap) compiler.enableSourceMap();
    return {
        compile(input) {
            return Promise.resolve().then(() => toResult(compiler.compile(input)));
        },
    };
}

/** 验证 LESS 语法是否正确 */
export function validateLess(input) {
    return Promise.resolve(validate_less(input));
}

/** 获取编译器版本号 */
export function getVersion() {
    return Promise.resolve(version());
}

/** 获取编译器描述信息 */
export function getInfo() {
    return Promise.resolve(info());
}

/** 获取编译错误的详细信息（无错误时为 undefined） */
export function getErrorDetails(input) {
    return Promise.resolve(get_error_details(input));
}

// 原始 wasm-bindgen 导出（高级用法）
export { init, compile_less, compile_less_with_options, WasmCompiler, WasmCompilerOptions };
EOF

cat > pkg/index.d.ts << 'EOF'
/** rust-less-wasm 类型定义 */
export interface RustLessCompileResult {
    css: string;
    sourceMap?: string;
    error?: string;
}

export interface RustLessCompileOptions {
    compress?: boolean;
    sourceMap?: boolean;
    sourceMapLessjsCompat?: boolean;
}

export interface RustLessCompiler {
    compile(input: string): Promise<RustLessCompileResult>;
}

export declare function compileLess(input: string): Promise<RustLessCompileResult>;
export declare function compileLessWithOptions(
    input: string,
    options?: RustLessCompileOptions,
): Promise<RustLessCompileResult>;
export declare function createCompiler(options?: RustLessCompileOptions): Promise<RustLessCompiler>;
export declare function validateLess(input: string): Promise<boolean>;
export declare function getVersion(): Promise<string>;
export declare function getInfo(): Promise<string>;
export declare function getErrorDetails(input: string): Promise<string | undefined>;

// 原始 wasm-bindgen 导出（高级用法）
export { init, compile_less, compile_less_with_options, WasmCompiler, WasmCompilerOptions } from './rust_less.js';
EOF

cat > pkg/package.json << EOF
{
  "name": "rust-less-wasm",
  "version": "${VERSION}",
  "description": "Rust 编写的高性能 LESS 到 CSS 编译器（WebAssembly，浏览器 / bundler）",
  "type": "module",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/YangChengxxyy/rust-less"
  },
  "main": "index.js",
  "types": "index.d.ts",
  "exports": {
    ".": {
      "types": "./index.d.ts",
      "import": "./index.js"
    }
  },
  "files": [
    "index.js",
    "index.d.ts",
    "rust_less.js",
    "rust_less_bg.js",
    "rust_less_bg.wasm",
    "rust_less.d.ts",
    "SHA256SUMS.txt",
    "VERSION"
  ],
  "sideEffects": false,
  "engines": {
    "node": ">=18"
  },
  "keywords": [
    "less",
    "css",
    "compiler",
    "preprocessor",
    "wasm",
    "webassembly",
    "rust"
  ]
}
EOF

cat > pkg/README.md << 'EOF'
# rust-less-wasm

Rust 编写的高性能 LESS 到 CSS 编译器 —— WebAssembly 版本（浏览器 / bundler 目标）。

## 安装

```bash
npm install rust-less-wasm
```

## 浏览器 / bundler 使用

bundler 目标在模块导入时自动完成 WASM 初始化，**无需手动调用 init()**：

```js
import { compileLess, compileLessWithOptions, createCompiler } from 'rust-less-wasm';

const { css, error } = await compileLess('@c: #333; .a { color: @c; }');
if (error) console.error(error);

// 带选项编译
const result = await compileLessWithOptions(source, {
    compress: true,             // 压缩输出
    sourceMap: true,            // 生成 source map（JSON 字符串）
    sourceMapLessjsCompat: true // less.js 兼容的 source map 字段名
});
```

## 结果结构

```ts
interface RustLessCompileResult {
    css: string;         // 编译后的 CSS（失败时为空字符串）
    sourceMap?: string;  // source map JSON（启用 sourceMap 选项时存在）
    error?: string;      // 编译错误信息（成功时不存在）
}
```

编译失败**不会抛出异常**，而是返回 `{ css: "", error: "..." }`。

## API

| 函数 | 说明 |
| --- | --- |
| `compileLess(input)` | 使用默认选项编译 |
| `compileLessWithOptions(input, options?)` | 自定义选项编译 |
| `createCompiler(options?)` | 创建可复用的编译器实例 |
| `validateLess(input)` | 验证语法（不完整编译） |
| `getVersion()` | 编译器版本号 |
| `getInfo()` | 编译器描述信息 |
| `getErrorDetails(input)` | 编译错误详情（无错误时 undefined） |

另导出原始 wasm-bindgen 绑定：`init`、`compile_less`、`compile_less_with_options`、
`WasmCompiler`、`WasmCompilerOptions`。

Node.js 环境请使用 [`rust-less-wasm-node`](https://www.npmjs.com/package/rust-less-wasm-node)。
EOF

# ---------------------------------------------------------------------------
# pkg-nodejs/ 后处理 —— rust-less-wasm-node
# ---------------------------------------------------------------------------
echo "📦 后处理 pkg-nodejs/ (rust-less-wasm-node)..."

cat > pkg-nodejs/index.cjs << 'EOF'
/**
 * rust-less-wasm-node —— Rust LESS 编译器的 WebAssembly 绑定（Node.js 变体）
 *
 * 供构建工具插件（webpack/vite/rollup）使用。CommonJS 入口，
 * require() 时自动完成 WASM 初始化（同步）。所有包装 API 为异步（Promise）形式；
 * 编译失败不抛出异常，而是返回 { css: "", error: "<信息>" }。
 */
'use strict';

const wasm = require('./rust_less.js');

/** 将 wasm-bindgen 的 CompileResult 转换为普通对象（包含 sourceMap） */
function toResult(result) {
    const out = { css: result.css };
    if (typeof result.sourceMap === 'string' && result.sourceMap.length > 0) {
        out.sourceMap = result.sourceMap;
    }
    if (result.error != null) {
        out.error = result.error;
    }
    return out;
}

/** 将普通选项对象映射到 WasmCompilerOptions */
function makeOptions(options) {
    const opts = new wasm.WasmCompilerOptions();
    if (options) {
        if (options.compress !== undefined) opts.compress = !!options.compress;
        if (options.sourceMap !== undefined) opts.source_map = !!options.sourceMap;
        if (options.sourceMapLessjsCompat !== undefined) opts.sourceMapLessjsCompat = !!options.sourceMapLessjsCompat;
    }
    return opts;
}

/** 编译 LESS 源码（默认选项） */
function compileLess(input) {
    return Promise.resolve().then(() => toResult(wasm.compile_less(input)));
}

/** 编译 LESS 源码（自定义选项） */
function compileLessWithOptions(input, options) {
    return Promise.resolve().then(() => {
        const opts = makeOptions(options);
        try {
            return toResult(wasm.compile_less_with_options(input, opts));
        } finally {
            opts.free();
        }
    });
}

/** 创建可复用的编译器实例 */
async function createCompiler(options) {
    const compiler = options && options.compress ? wasm.WasmCompiler.compressed() : new wasm.WasmCompiler();
    if (options && options.sourceMap) compiler.enableSourceMap();
    return {
        compile(input) {
            return Promise.resolve().then(() => toResult(compiler.compile(input)));
        },
    };
}

/** 验证 LESS 语法是否正确 */
function validateLess(input) {
    return Promise.resolve(wasm.validate_less(input));
}

/** 获取编译器版本号 */
function getVersion() {
    return Promise.resolve(wasm.version());
}

/** 获取编译器描述信息 */
function getInfo() {
    return Promise.resolve(wasm.info());
}

/** 获取编译错误的详细信息（无错误时为 undefined） */
function getErrorDetails(input) {
    return Promise.resolve(wasm.get_error_details(input));
}

module.exports = {
    compileLess,
    compileLessWithOptions,
    createCompiler,
    validateLess,
    getVersion,
    getInfo,
    getErrorDetails,
};
EOF

cat > pkg-nodejs/index.mjs << 'EOF'
/**
 * rust-less-wasm-node —— ESM 入口（转发到 CommonJS 实现）
 */
import nodeCreateRequire from 'node:module';
const createRequire = nodeCreateRequire.createRequire;
const require = createRequire(import.meta.url);
const cjs = require('./index.cjs');

export const compileLess = cjs.compileLess;
export const compileLessWithOptions = cjs.compileLessWithOptions;
export const createCompiler = cjs.createCompiler;
export const validateLess = cjs.validateLess;
export const getVersion = cjs.getVersion;
export const getInfo = cjs.getInfo;
export const getErrorDetails = cjs.getErrorDetails;
export default cjs;
EOF

cat > pkg-nodejs/index.d.ts << 'EOF'
/** rust-less-wasm-node 类型定义 */
export interface RustLessCompileResult {
    css: string;
    sourceMap?: string;
    error?: string;
}

export interface RustLessCompileOptions {
    compress?: boolean;
    sourceMap?: boolean;
    sourceMapLessjsCompat?: boolean;
}

export interface RustLessCompiler {
    compile(input: string): Promise<RustLessCompileResult>;
}

export declare function compileLess(input: string): Promise<RustLessCompileResult>;
export declare function compileLessWithOptions(
    input: string,
    options?: RustLessCompileOptions,
): Promise<RustLessCompileResult>;
export declare function createCompiler(options?: RustLessCompileOptions): Promise<RustLessCompiler>;
export declare function validateLess(input: string): Promise<boolean>;
export declare function getVersion(): Promise<string>;
export declare function getInfo(): Promise<string>;
export declare function getErrorDetails(input: string): Promise<string | undefined>;

export as namespace RustLessWasmNode;
export = RustLessWasmNode;
declare const RustLessWasmNode: {
    compileLess: typeof compileLess;
    compileLessWithOptions: typeof compileLessWithOptions;
    createCompiler: typeof createCompiler;
    validateLess: typeof validateLess;
    getVersion: typeof getVersion;
    getInfo: typeof getInfo;
    getErrorDetails: typeof getErrorDetails;
};
EOF

cat > pkg-nodejs/package.json << EOF
{
  "name": "rust-less-wasm-node",
  "version": "${VERSION}",
  "description": "Rust 编写的高性能 LESS 到 CSS 编译器（WebAssembly，Node.js 变体，供构建工具插件使用）",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/YangChengxxyy/rust-less"
  },
  "main": "index.cjs",
  "module": "index.mjs",
  "types": "index.d.ts",
  "exports": {
    ".": {
      "types": "./index.d.ts",
      "import": "./index.mjs",
      "require": "./index.cjs"
    }
  },
  "files": [
    "index.cjs",
    "index.mjs",
    "index.d.ts",
    "rust_less.js",
    "rust_less_bg.wasm",
    "rust_less.d.ts",
    "SHA256SUMS.txt",
    "VERSION"
  ],
  "engines": {
    "node": ">=18"
  },
  "keywords": [
    "less",
    "css",
    "compiler",
    "preprocessor",
    "wasm",
    "webassembly",
    "rust",
    "node"
  ]
}
EOF

cat > pkg-nodejs/README.md << 'EOF'
# rust-less-wasm-node

Rust 编写的高性能 LESS 到 CSS 编译器 —— WebAssembly 版本（**Node.js 变体**）。

本包是 [`rust-less-wasm`](https://www.npmjs.com/package/rust-less-wasm) 的 Node.js 对应版本，
主要供构建工具插件（webpack / vite / rollup 的 rust-less 插件）通过
`await import('rust-less-wasm-node')` 懒加载使用。

## 安装

```bash
npm install rust-less-wasm-node
```

## 使用

```js
// CommonJS
const { compileLess, compileLessWithOptions } = require('rust-less-wasm-node');

// ESM
import { compileLess } from 'rust-less-wasm-node';

const { css, error } = await compileLess('@c: #333; .a { color: @c; }');
if (error) console.error(error);
```

require / import 时自动完成 WASM 初始化（同步，无需手动 init）。

## 结果结构

```ts
interface RustLessCompileResult {
    css: string;         // 编译后的 CSS（失败时为空字符串）
    sourceMap?: string;  // source map JSON（启用 sourceMap 选项时存在）
    error?: string;      // 编译错误信息（成功时不存在）
}
```

编译失败**不会抛出异常**，而是返回 `{ css: "", error: "..." }`。

## API

与 `rust-less-wasm` 完全一致：`compileLess` / `compileLessWithOptions` /
`createCompiler` / `validateLess` / `getVersion` / `getInfo` / `getErrorDetails`。
选项支持 `{ compress, sourceMap, sourceMapLessjsCompat }`。
EOF

# ---------------------------------------------------------------------------
# pkg-web/ 后处理 —— rust-less-wasm-web
# ---------------------------------------------------------------------------
echo "📦 后处理 pkg-web/ (rust-less-wasm-web)..."

cat > pkg-web/package.json << EOF
{
  "name": "rust-less-wasm-web",
  "version": "${VERSION}",
  "description": "Rust LESS 编译器 WebAssembly 绑定（直接 <script> 引入的 Web 目标）",
  "type": "module",
  "license": "MIT",
  "files": [
    "rust_less.js",
    "rust_less_bg.wasm",
    "rust_less.d.ts",
    "SHA256SUMS.txt",
    "VERSION"
  ],
  "keywords": [
    "less",
    "css",
    "compiler",
    "wasm",
    "web"
  ]
}
EOF

cat > pkg-web/README.md << 'EOF'
# rust-less-wasm-web

Rust LESS 编译器的 WebAssembly 绑定（`wasm-pack --target web`，通过
`init()` + `default` 导出在浏览器中直接使用）。

> **注意**：浏览器场景的主发布目标是 [`rust-less-wasm`](https://www.npmjs.com/package/rust-less-wasm)
> （bundler 目标，含完整异步包装 API）。本包仅供无法使用打包器的
> 直接 `<script>` / ESM CDN 场景使用。

```js
import init, { compile_less } from './rust_less.js';
await init(); // web 目标需要手动初始化
const result = compile_less('@c: #333; .a { color: @c; }');
console.log(result.css);
```
EOF

# ---------------------------------------------------------------------------
# 通用后处理：LICENSE、SHA256SUMS.txt、VERSION
# ---------------------------------------------------------------------------
echo "📝 写入校验和与版本文件..."
for dir in pkg pkg-nodejs pkg-web; do
    if [ -f LICENSE ]; then
        cp LICENSE "$dir/LICENSE"
    fi
    echo "$VERSION" > "$dir/VERSION"
    (
        cd "$dir"
        shasum -a 256 $(ls | grep -v -E '^(SHA256SUMS\.txt|VERSION|LICENSE|\.gitignore)$') > SHA256SUMS.txt
    )
done

# ---------------------------------------------------------------------------
# 冒烟测试
# ---------------------------------------------------------------------------
echo "🧪 运行冒烟测试..."
"$0" --smoke

# ---------------------------------------------------------------------------
# 汇总
# ---------------------------------------------------------------------------
echo ""
echo "✅ WASM 构建完成！"
echo ""
echo "📦 产物汇总（版本 ${VERSION}）:"
printf '  %-14s %-20s %s\n' "目录" "npm 包名" "受众"
printf '  %-14s %-20s %s\n' "pkg/" "rust-less-wasm" "浏览器 / bundler（主目标）"
printf '  %-14s %-20s %s\n' "pkg-nodejs/" "rust-less-wasm-node" "Node.js / 构建工具插件"
printf '  %-14s %-20s %s\n' "pkg-web/" "rust-less-wasm-web" "直接 <script> 引入的 Web 目标"
echo ""
echo "🚀 发布方式:"
echo "  1. 预览: cd pkg && npm pack --dry-run（三个目录同理）"
echo "  2. 发布: cd pkg && npm publish（先 pkg，再 pkg-nodejs、pkg-web）"
echo "  3. 详细流程参见 docs/WASM_RELEASE.md"
