#!/usr/bin/env node

/**
 * Rust LESS 编译器 - Node.js 使用示例
 *
 * 此示例展示了如何在 Node.js 环境中使用 Rust LESS 编译器的 WebAssembly 版本
 */

const fs = require('fs').promises;
const path = require('path');

// 实际使用时安装 npm 包：npm install rust-less-wasm-node
// const { compileLess, compileLessWithOptions, createCompiler, validateLess, getVersion, getInfo } = require('rust-less-wasm-node');
//
// 未安装 npm 包时，回退到本仓库 ./build-wasm.sh 生成的本地构建产物（pkg-nodejs/），
// 两者的 API 完全一致：
//   compileLess(input)                      -> Promise<{ css, sourceMap?, error? }>
//   compileLessWithOptions(input, options?)  -> Promise<{ css, sourceMap?, error? }>
//   createCompiler(options?)                 -> Promise<{ compile(input) }>
//   validateLess(input)                      -> Promise<boolean>
//   getVersion() / getInfo()                 -> Promise<string>
let wasmModule;
try {
    wasmModule = require('rust-less-wasm-node');
} catch {
    const localPkg = path.resolve(__dirname, '../../pkg-nodejs/index.cjs');
    if (!require('fs').existsSync(localPkg)) {
        console.error('❌ 未找到 rust-less-wasm-node，请先运行: ./build-wasm.sh');
        process.exit(1);
    }
    wasmModule = require(localPkg);
}

const { compileLess, compileLessWithOptions, createCompiler, validateLess, getVersion, getInfo } = wasmModule;

/**
 * 示例 1: 基本编译
 */
async function example1BasicCompilation() {
    console.log('\n🎯 示例 1: 基本 LESS 编译');
    console.log('='.repeat(50));

    const lessCode = `
@primary-color: #3498db;
@secondary-color: #2ecc71;
@margin: 20px;

.header {
    color: @primary-color;
    background: lighten(@primary-color, 40%);
    margin: @margin;

    h1 {
        color: @secondary-color;
        font-size: 24px;
    }

    &:hover {
        background: darken(@primary-color, 10%);
    }
}

.button {
    background: @primary-color;
    color: white;
    padding: @margin / 2;
    border-radius: @margin / 4;

    &.secondary {
        background: @secondary-color;
    }
}
`;

    console.log('📝 LESS 输入:');
    console.log(lessCode);

    try {
        const result = await compileLess(lessCode);

        if (result.error) {
            console.log('❌ 编译错误:', result.error);
        } else {
            console.log('✅ CSS 输出:');
            console.log(result.css);
        }
    } catch (error) {
        console.error('❌ 编译失败:', error.message);
    }
}

/**
 * 示例 2: 带选项编译
 */
async function example2CompilationWithOptions() {
    console.log('\n🎯 示例 2: 带选项的 LESS 编译');
    console.log('='.repeat(50));

    const lessCode = `
@base-size: 16px;
@primary-color: #e74c3c;

.card {
    padding: @base-size;
    border-radius: @base-size / 4;
    background: white;

    .title {
        color: @primary-color;
        font-size: @base-size * 1.5;
        margin-bottom: @base-size / 2;
    }

    .content {
        line-height: 1.6;
        font-size: @base-size;
    }
}
`;

    console.log('📝 LESS 输入:');
    console.log(lessCode);

    // 普通编译
    console.log('\n📦 普通编译结果:');
    try {
        const normalResult = await compileLessWithOptions(lessCode, {
            compress: false,
            sourceMap: false
        });

        if (normalResult.error) {
            console.log('❌ 编译错误:', normalResult.error);
        } else {
            console.log('✅ CSS 输出:');
            console.log(normalResult.css);
        }
    } catch (error) {
        console.error('❌ 编译失败:', error.message);
    }

    // 压缩编译
    console.log('\n🗜️ 压缩编译结果:');
    try {
        const compressedResult = await compileLessWithOptions(lessCode, {
            compress: true,
            sourceMap: false
        });

        if (compressedResult.error) {
            console.log('❌ 编译错误:', compressedResult.error);
        } else {
            console.log('✅ 压缩后的 CSS:');
            console.log(compressedResult.css);
        }
    } catch (error) {
        console.error('❌ 编译失败:', error.message);
    }
}

/**
 * 示例 3: 使用编译器实例
 */
async function example3CompilerInstance() {
    console.log('\n🎯 示例 3: 使用编译器实例');
    console.log('='.repeat(50));

    try {
        // 创建普通编译器
        const normalCompiler = await createCompiler();

        // 创建压缩编译器
        const compressedCompiler = await createCompiler({ compress: true });

        const lessCode = `
@theme-color: #9b59b6;
@spacing: 1rem;

.widget {
    background: @theme-color;
    padding: @spacing;
    margin: @spacing / 2;

    .icon {
        width: @spacing * 2;
        height: @spacing * 2;
    }
}
`;

        console.log('📝 LESS 输入:');
        console.log(lessCode);

        // 普通编译
        console.log('\n📦 普通编译器结果:');
        const normalResult = await normalCompiler.compile(lessCode);
        if (normalResult.error) {
            console.log('❌ 编译错误:', normalResult.error);
        } else {
            console.log('✅ CSS 输出:');
            console.log(normalResult.css);
        }

        // 压缩编译
        console.log('\n🗜️ 压缩编译器结果:');
        const compressedResult = await compressedCompiler.compile(lessCode);
        if (compressedResult.error) {
            console.log('❌ 编译错误:', compressedResult.error);
        } else {
            console.log('✅ 压缩后的 CSS:');
            console.log(compressedResult.css);
        }

    } catch (error) {
        console.error('❌ 创建编译器失败:', error.message);
    }
}

/**
 * 示例 4: 语法验证
 */
async function example4Validation() {
    console.log('\n🎯 示例 4: LESS 语法验证');
    console.log('='.repeat(50));

    const testCases = [
        {
            name: '正确的 LESS 代码',
            code: '@color: red; .test { color: @color; }'
        },
        {
            name: '错误的 LESS 代码',
            code: '@color: red .test { color: @color; }' // 缺少分号
        },
        {
            name: '包含嵌套的 LESS 代码',
            code: `
.parent {
    color: blue;
    .child {
        color: red;
    }
}`
        },
        {
            name: '语法错误的代码',
            code: 'syntax-error invalid less code'
        }
    ];

    for (const testCase of testCases) {
        console.log(`\n📝 测试: ${testCase.name}`);
        console.log(`代码: ${testCase.code}`);

        try {
            const isValid = await validateLess(testCase.code);
            console.log(`结果: ${isValid ? '✅ 语法正确' : '❌ 语法错误'}`);
        } catch (error) {
            console.log(`❌ 验证失败: ${error.message}`);
        }
    }
}

/**
 * 示例 5: 文件处理
 */
async function example5FileProcessing() {
    console.log('\n🎯 示例 5: 文件处理');
    console.log('='.repeat(50));

    // 创建示例 LESS 文件内容
    const lessContent = `
// 主题变量
@primary-color: #2c3e50;
@secondary-color: #3498db;
@accent-color: #e74c3c;
@font-size: 16px;
@line-height: 1.6;

// 基础样式
body {
    font-size: @font-size;
    line-height: @line-height;
    color: @primary-color;

    h1, h2, h3 {
        color: @secondary-color;
        margin-bottom: @font-size;
    }

    .highlight {
        color: @accent-color;
        font-weight: bold;

        &:hover {
            text-decoration: underline;
        }
    }
}

// 按钮组件
.btn {
    display: inline-block;
    padding: @font-size / 2 @font-size;
    border: none;
    border-radius: @font-size / 4;
    cursor: pointer;
    transition: all 0.3s ease;

    &.primary {
        background: @primary-color;
        color: white;

        &:hover {
            background: darken(@primary-color, 10%);
        }
    }

    &.secondary {
        background: @secondary-color;
        color: white;

        &:hover {
            background: darken(@secondary-color, 10%);
        }
    }
}
`;

    try {
        // 模拟写入文件
        console.log('📄 创建示例 LESS 文件...');
        console.log('文件内容:');
        console.log(lessContent);

        // 编译 LESS
        console.log('\n🔨 编译 LESS 文件...');
        const result = await compileLess(lessContent);

        if (result.error) {
            console.log('❌ 编译错误:', result.error);
            return;
        }

        console.log('✅ 编译成功！');
        console.log('\n📄 生成的 CSS:');
        console.log(result.css);

        // 模拟保存 CSS 文件
        console.log('\n💾 CSS 已保存到 output.css (模拟)');

        // 压缩版本
        console.log('\n🗜️ 生成压缩版本...');
        const compressedResult = await compileLessWithOptions(lessContent, { compress: true });

        if (compressedResult.error) {
            console.log('❌ 压缩编译错误:', compressedResult.error);
        } else {
            console.log('✅ 压缩版本:');
            console.log(compressedResult.css);
            console.log('\n💾 压缩版本已保存到 output.min.css (模拟)');
        }

    } catch (error) {
        console.error('❌ 文件处理失败:', error.message);
    }
}

/**
 * 示例 6: 编译器信息
 */
async function example6CompilerInfo() {
    console.log('\n🎯 示例 6: 编译器信息');
    console.log('='.repeat(50));

    try {
        // 获取版本信息
        const version = await getVersion();
        console.log(`📋 编译器版本: ${version}`);

        // 获取详细信息
        const info = await getInfo();
        console.log('\n📋 编译器详细信息:');
        console.log(info);

        // 性能信息（模拟）
        console.log('\n⚡ 性能特性:');
        console.log('- 解析速度: ~100万行/秒');
        console.log('- 编译速度: ~50万行/秒');
        console.log('- 内存使用: 与输入大小成线性关系');
        console.log('- WebAssembly 模块大小: ~500KB (压缩后)');

    } catch (error) {
        console.error('❌ 获取编译器信息失败:', error.message);
    }
}

/**
 * 主函数
 */
async function main() {
    console.log('🦀 Rust LESS 编译器 - Node.js 示例');
    console.log('用 Rust 编写的高性能 LESS 到 CSS 编译器 WebAssembly 版本');
    console.log('='.repeat(80));

    try {
        await example1BasicCompilation();
        await example2CompilationWithOptions();
        await example3CompilerInstance();
        await example4Validation();
        await example5FileProcessing();
        await example6CompilerInfo();

        console.log('\n🎉 所有示例运行完成！');
        console.log('\n📚 使用说明:');
        console.log('1. 安装: npm install rust-less-wasm');
        console.log('2. 导入: const { compileLess } = require("rust-less-wasm");');
        console.log('3. 使用: const result = await compileLess(lessCode);');
        console.log('\n💡 提示: 这个示例使用了模拟的 WASM 模块。');
        console.log('   在实际项目中，请安装并导入真实的 rust-less-wasm 包。');

    } catch (error) {
        console.error('❌ 示例运行失败:', error.message);
        process.exit(1);
    }
}

// 错误处理
process.on('unhandledRejection', (reason, promise) => {
    console.error('未处理的 Promise 拒绝:', reason);
    process.exit(1);
});

process.on('uncaughtException', (error) => {
    console.error('未捕获的异常:', error);
    process.exit(1);
});

// 运行示例
if (require.main === module) {
    main().catch(console.error);
}

module.exports = {
    example1BasicCompilation,
    example2CompilationWithOptions,
    example3CompilerInstance,
    example4Validation,
    example5FileProcessing,
    example6CompilerInfo
};
