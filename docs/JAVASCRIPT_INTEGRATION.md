# JavaScript 集成指南

本文档详细介绍了如何将 Rust LESS 编译器集成到 JavaScript 项目中。

## 📋 目录

- [概述](#概述)
- [WebAssembly 集成（推荐）](#webassembly-集成推荐)
- [Node.js 原生模块](#nodejs-原生模块)
- [命令行集成](#命令行集成)
- [构建工具插件](#构建工具插件)
- [性能对比](#性能对比)
- [故障排除](#故障排除)

## 概述

Rust LESS 编译器提供多种方式与 JavaScript 生态系统集成：

1. **WebAssembly (WASM)** - 推荐方案，支持浏览器和 Node.js
2. **Node.js 原生模块** - 最高性能，仅支持 Node.js
3. **命令行集成** - 通过子进程调用
4. **构建工具插件** - 集成到 Webpack、Vite 等

## WebAssembly 集成（推荐）

### 🚀 快速开始

#### 1. 构建 WASM 模块

```bash
# 安装必要工具
cargo install wasm-pack

# 构建 WebAssembly 模块
cd rust-less
./build-wasm.sh
```

#### 2. 安装到项目

```bash
# 方式 1: 本地安装
npm install ./rust-less/pkg

# 方式 2: 从 npm 安装（发布后）
npm install rust-less-wasm
```

#### 3. 基本使用

```javascript
import { compileLess, compileLessWithOptions } from 'rust-less-wasm';

// 简单编译
const lessCode = `
@primary: #3498db;
.header { 
  color: @primary; 
  &:hover { color: darken(@primary, 10%); }
}`;

const result = await compileLess(lessCode);
if (result.error) {
  console.error('编译错误:', result.error);
} else {
  console.log('CSS 输出:', result.css);
}

// 带选项编译
const compressedResult = await compileLessWithOptions(lessCode, {
  compress: true,
  sourceMap: false
});
```

### 🌐 浏览器集成

#### HTML 中使用

```html
<!DOCTYPE html>
<html>
<head>
    <title>LESS 在线编译器</title>
</head>
<body>
    <textarea id="less-input" placeholder="输入 LESS 代码"></textarea>
    <button onclick="compile()">编译</button>
    <pre id="css-output"></pre>

    <script type="module">
        import { compileLess } from './rust-less-wasm/index.js';

        window.compile = async function() {
            const input = document.getElementById('less-input').value;
            const output = document.getElementById('css-output');
            
            try {
                const result = await compileLess(input);
                output.textContent = result.error || result.css;
            } catch (error) {
                output.textContent = '编译失败: ' + error.message;
            }
        };
    </script>
</body>
</html>
```

#### React 组件示例

```jsx
import React, { useState, useEffect } from 'react';
import { compileLess } from 'rust-less-wasm';

function LessCompiler() {
    const [lessCode, setLessCode] = useState('@color: blue;\n.test { color: @color; }');
    const [cssOutput, setCssOutput] = useState('');
    const [error, setError] = useState('');

    const handleCompile = async () => {
        try {
            const result = await compileLess(lessCode);
            if (result.error) {
                setError(result.error);
                setCssOutput('');
            } else {
                setError('');
                setCssOutput(result.css);
            }
        } catch (err) {
            setError(err.message);
        }
    };

    useEffect(() => {
        handleCompile();
    }, [lessCode]);

    return (
        <div className="less-compiler">
            <div className="input-section">
                <h3>LESS 输入</h3>
                <textarea
                    value={lessCode}
                    onChange={(e) => setLessCode(e.target.value)}
                    rows={10}
                    cols={50}
                />
            </div>
            
            <div className="output-section">
                <h3>CSS 输出</h3>
                {error ? (
                    <div className="error">{error}</div>
                ) : (
                    <pre>{cssOutput}</pre>
                )}
            </div>
        </div>
    );
}

export default LessCompiler;
```

### 🖥️ Node.js 集成

#### 基本文件处理

```javascript
const fs = require('fs').promises;
const { compileLess, createCompiler } = require('rust-less-wasm');

async function compileLessFile(inputPath, outputPath, options = {}) {
    try {
        // 读取 LESS 文件
        const lessContent = await fs.readFile(inputPath, 'utf8');
        
        // 编译
        const result = await compileLess(lessContent);
        
        if (result.error) {
            throw new Error(`编译错误: ${result.error}`);
        }
        
        // 写入 CSS 文件
        await fs.writeFile(outputPath, result.css);
        console.log(`✅ 成功编译: ${inputPath} -> ${outputPath}`);
        
        return result.css;
    } catch (error) {
        console.error(`❌ 编译失败: ${error.message}`);
        throw error;
    }
}

// 使用示例
async function main() {
    await compileLessFile('styles/main.less', 'dist/main.css');
    
    // 批量编译
    const files = ['header.less', 'footer.less', 'components.less'];
    
    for (const file of files) {
        await compileLessFile(
            `src/styles/${file}`,
            `dist/${file.replace('.less', '.css')}`
        );
    }
}

main().catch(console.error);
```

#### Express.js 中间件

```javascript
const express = require('express');
const { compileLess } = require('rust-less-wasm');

function lessMiddleware(options = {}) {
    return async (req, res, next) => {
        if (!req.path.endsWith('.css')) {
            return next();
        }

        const lessPath = req.path.replace('.css', '.less');
        const fullLessPath = `${options.src}${lessPath}`;

        try {
            const fs = require('fs').promises;
            const lessContent = await fs.readFile(fullLessPath, 'utf8');
            
            const result = await compileLess(lessContent);
            
            if (result.error) {
                return res.status(500).send(`LESS 编译错误: ${result.error}`);
            }

            res.type('text/css');
            res.send(result.css);
        } catch (error) {
            if (error.code === 'ENOENT') {
                return next(); // 文件不存在，继续下一个中间件
            }
            res.status(500).send(`服务器错误: ${error.message}`);
        }
    };
}

const app = express();

// 使用 LESS 中间件
app.use(lessMiddleware({ src: './src/styles' }));

// 静态文件服务
app.use(express.static('public'));

app.listen(3000, () => {
    console.log('服务器运行在 http://localhost:3000');
});
```

## Node.js 原生模块

对于需要最高性能的 Node.js 应用，可以创建原生模块。

### 1. 创建原生绑定

```toml
# Cargo.toml 添加
[dependencies]
neon = "0.10"

[[bin]]
name = "native"
path = "src/bin/native.rs"
```

```rust
// src/bin/native.rs
use neon::prelude::*;
use rust_less::{compile, Compiler};

fn compile_less(mut cx: FunctionContext) -> JsResult<JsObject> {
    let input = cx.argument::<JsString>(0)?.value(&mut cx);
    
    let result = cx.empty_object();
    
    match compile(&input) {
        Ok(css) => {
            let css_value = cx.string(css);
            result.set(&mut cx, "css", css_value)?;
        }
        Err(error) => {
            let error_value = cx.string(error.to_string());
            result.set(&mut cx, "error", error_value)?;
        }
    }
    
    Ok(result)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("compileLess", compile_less)?;
    Ok(())
}
```

### 2. 构建和使用

```bash
# 构建原生模块
npm install -g neon-cli
neon build --release

# 使用
const { compileLess } = require('./native');

const result = compileLess('@color: red; .test { color: @color; }');
console.log(result.css || result.error);
```

## 命令行集成

通过子进程调用 CLI 工具。

```javascript
const { spawn } = require('child_process');
const fs = require('fs').promises;

class LessCompilerCLI {
    constructor(binaryPath = 'rust-less') {
        this.binaryPath = binaryPath;
    }

    async compile(lessContent, options = {}) {
        return new Promise((resolve, reject) => {
            const args = [];
            
            if (options.compress) {
                args.push('--compress');
            }
            
            if (options.output) {
                args.push('-o', options.output);
            }

            const child = spawn(this.binaryPath, args, {
                stdio: ['pipe', 'pipe', 'pipe']
            });

            let stdout = '';
            let stderr = '';

            child.stdout.on('data', (data) => {
                stdout += data.toString();
            });

            child.stderr.on('data', (data) => {
                stderr += data.toString();
            });

            child.on('close', (code) => {
                if (code === 0) {
                    resolve({ css: stdout, error: null });
                } else {
                    reject(new Error(stderr || `进程退出码: ${code}`));
                }
            });

            child.on('error', reject);

            // 发送 LESS 内容到 stdin
            child.stdin.write(lessContent);
            child.stdin.end();
        });
    }

    async compileFile(inputPath, outputPath, options = {}) {
        return new Promise((resolve, reject) => {
            const args = [inputPath];
            
            if (outputPath) {
                args.push('-o', outputPath);
            }
            
            if (options.compress) {
                args.push('--compress');
            }

            const child = spawn(this.binaryPath, args);

            child.on('close', (code) => {
                if (code === 0) {
                    resolve({ success: true });
                } else {
                    reject(new Error(`编译失败，退出码: ${code}`));
                }
            });

            child.on('error', reject);
        });
    }
}

// 使用示例
async function example() {
    const compiler = new LessCompilerCLI();
    
    try {
        const result = await compiler.compile('@color: blue; .test { color: @color; }');
        console.log('CSS 输出:', result.css);
        
        await compiler.compileFile('input.less', 'output.css', { compress: true });
        console.log('文件编译完成');
    } catch (error) {
        console.error('编译错误:', error.message);
    }
}
```

## 构建工具插件

### Webpack 插件

```javascript
// webpack-rust-less-plugin.js
const { compileLessWithOptions } = require('rust-less-wasm');

class RustLessPlugin {
    constructor(options = {}) {
        this.options = {
            compress: false,
            sourceMap: true,
            ...options
        };
    }

    apply(compiler) {
        compiler.hooks.compilation.tap('RustLessPlugin', (compilation) => {
            compilation.hooks.additionalAssets.tapAsync('RustLessPlugin', async (callback) => {
                const lessAssets = Object.keys(compilation.assets)
                    .filter(name => name.endsWith('.less'));

                for (const assetName of lessAssets) {
                    const lessContent = compilation.assets[assetName].source();
                    
                    try {
                        const result = await compileLessWithOptions(lessContent, this.options);
                        
                        if (result.error) {
                            compilation.errors.push(new Error(`LESS 编译错误 (${assetName}): ${result.error}`));
                        } else {
                            const cssName = assetName.replace('.less', '.css');
                            compilation.assets[cssName] = {
                                source: () => result.css,
                                size: () => result.css.length
                            };
                        }
                    } catch (error) {
                        compilation.errors.push(new Error(`LESS 处理失败 (${assetName}): ${error.message}`));
                    }
                }

                callback();
            });
        });
    }
}

module.exports = RustLessPlugin;
```

```javascript
// webpack.config.js
const RustLessPlugin = require('./webpack-rust-less-plugin');

module.exports = {
    // ... 其他配置
    plugins: [
        new RustLessPlugin({
            compress: process.env.NODE_ENV === 'production',
            sourceMap: process.env.NODE_ENV === 'development'
        })
    ]
};
```

### Vite 插件

```javascript
// vite-plugin-rust-less.js
import { compileLessWithOptions } from 'rust-less-wasm';

export function rustLess(options = {}) {
    return {
        name: 'rust-less',
        async transform(code, id) {
            if (!id.endsWith('.less')) {
                return null;
            }

            try {
                const result = await compileLessWithOptions(code, {
                    compress: options.compress || false,
                    sourceMap: options.sourceMap !== false
                });

                if (result.error) {
                    this.error(`LESS 编译错误: ${result.error}`);
                    return null;
                }

                return {
                    code: `export default ${JSON.stringify(result.css)}`,
                    map: null // TODO: 实现 source map
                };
            } catch (error) {
                this.error(`LESS 处理失败: ${error.message}`);
                return null;
            }
        }
    };
}
```

```javascript
// vite.config.js
import { defineConfig } from 'vite';
import { rustLess } from './vite-plugin-rust-less';

export default defineConfig({
    plugins: [
        rustLess({
            compress: true,
            sourceMap: true
        })
    ]
});
```

## 性能对比

| 方案 | 初始化时间 | 编译速度 | 内存使用 | 包大小 | 兼容性 |
|------|------------|----------|----------|---------|---------|
| **WebAssembly** | ~100ms | 快 | 中等 | ~500KB | 浏览器 + Node.js |
| **原生模块** | ~10ms | 最快 | 最低 | ~2MB | 仅 Node.js |
| **CLI 调用** | ~200ms | 中等 | 高 | ~2MB | 需要二进制文件 |

### 性能测试代码

```javascript
const { performance } = require('perf_hooks');

async function benchmarkCompilation() {
    const lessCode = `
    @primary: #3498db;
    @secondary: #2ecc71;
    
    .component {
        color: @primary;
        background: lighten(@primary, 40%);
        
        .nested {
            color: @secondary;
            font-size: 14px;
            
            &:hover {
                color: darken(@secondary, 10%);
            }
        }
    }
    `.repeat(100); // 重复 100 次增加复杂度

    // WASM 性能测试
    const wasmStart = performance.now();
    await compileLess(lessCode);
    const wasmTime = performance.now() - wasmStart;

    console.log(`WASM 编译时间: ${wasmTime.toFixed(2)}ms`);
    console.log(`代码大小: ${lessCode.length} 字符`);
    console.log(`处理速度: ${(lessCode.length / wasmTime * 1000).toFixed(0)} 字符/秒`);
}
```

## 故障排除

### 常见问题

#### 1. WASM 模块加载失败

```javascript
// 检查 WASM 支持
if (typeof WebAssembly === 'undefined') {
    console.error('当前环境不支持 WebAssembly');
}

// 错误处理
try {
    await init(); // 初始化 WASM 模块
} catch (error) {
    console.error('WASM 初始化失败:', error);
    // 回退到其他方案
}
```

#### 2. 编译错误调试

```javascript
import { getErrorDetails, validateLess } from 'rust-less-wasm';

async function debugCompilation(lessCode) {
    // 首先验证语法
    const isValid = await validateLess(lessCode);
    if (!isValid) {
        const details = await getErrorDetails(lessCode);
        console.error('语法错误:', details);
        return;
    }

    // 尝试编译
    const result = await compileLess(lessCode);
    if (result.error) {
        console.error('编译错误:', result.error);
        // 可以尝试逐行编译来定位问题
    }
}
```

#### 3. 性能优化

```javascript
// 创建复用的编译器实例
const compiler = await createCompiler({ compress: true });

// 批量编译时复用实例
const results = await Promise.all(
    lessFiles.map(file => compiler.compile(file.content))
);

// 使用 Worker 进行后台编译
if (typeof Worker !== 'undefined') {
    const worker = new Worker('./less-compiler-worker.js');
    worker.postMessage({ lessCode, options });
}
```

### 调试工具

```javascript
// 开发模式辅助工具
class LessCompilerDebugger {
    constructor() {
        this.enableDebug = process.env.NODE_ENV === 'development';
    }

    async compileWithDebug(lessCode, options = {}) {
        if (this.enableDebug) {
            console.time('LESS 编译');
            console.log('输入代码长度:', lessCode.length);
        }

        const result = await compileLessWithOptions(lessCode, options);

        if (this.enableDebug) {
            console.timeEnd('LESS 编译');
            if (result.css) {
                console.log('输出 CSS 长度:', result.css.length);
                console.log('压缩率:', ((lessCode.length - result.css.length) / lessCode.length * 100).toFixed(1) + '%');
            }
        }

        return result;
    }
}
```

## 最佳实践

1. **选择合适的集成方案**
   - 浏览器项目：使用 WebAssembly
   - Node.js 高性能需求：使用原生模块
   - 简单脚本：使用 CLI 调用

2. **错误处理**
   - 始终检查编译结果的错误字段
   - 提供有意义的错误信息给用户
   - 实现编译失败的回退机制

3. **性能优化**
   - 复用编译器实例
   - 使用 Worker 进行后台编译
   - 缓存编译结果

4. **开发体验**
   - 集成到构建工具
   - 提供热重载支持
   - 添加语法高亮和错误提示

## 总结

Rust LESS 编译器提供了灵活的 JavaScript 集成选项，从简单的 WebAssembly 模块到高性能的原生绑定。选择合适的集成方案取决于你的具体需求：

- **学习和原型**：使用 WebAssembly 版本
- **生产环境**：根据性能需求选择 WebAssembly 或原生模块
- **构建工具集成**：开发专用插件

无论选择哪种方案，都能享受到 Rust 带来的高性能和内存安全优势。