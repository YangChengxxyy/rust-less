//! WebAssembly 绑定实现
//!
//! 此模块实现了将 Rust LESS 编译器功能暴露给 JavaScript 的具体绑定。

use crate::{compile, Compiler};
use wasm_bindgen::prelude::*;

// 当 `console_error_panic_hook` 功能启用时，我们可以调用 `set_panic_hook` 函数
// 来获得更好的错误消息调试体验
/// 设置 panic hook 以获得更好的错误报告
#[cfg(feature = "console_error_panic_hook")]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

// 当 `wee_alloc` 功能启用时，使用 `wee_alloc` 作为全局分配器
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// 初始化 WASM 模块
/// 设置 panic hook 以获得更好的错误报告
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    set_panic_hook();
}

/// JavaScript 中可用的编译选项
#[wasm_bindgen]
#[derive(Default)]
pub struct WasmCompilerOptions {
    compress: bool,
    source_map: bool,
    source_map_lessjs_compat: bool,
}

#[wasm_bindgen]
impl WasmCompilerOptions {
    /// 创建新的编译选项
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmCompilerOptions {
        WasmCompilerOptions::default()
    }

    /// 设置是否压缩输出
    #[wasm_bindgen(setter)]
    pub fn set_compress(&mut self, compress: bool) {
        self.compress = compress;
    }

    /// 获取压缩设置
    #[wasm_bindgen(getter)]
    pub fn compress(&self) -> bool {
        self.compress
    }

    /// 设置是否生成源码映射
    #[wasm_bindgen(setter)]
    pub fn set_source_map(&mut self, source_map: bool) {
        self.source_map = source_map;
    }

    /// 获取源码映射设置
    #[wasm_bindgen(getter)]
    pub fn source_map(&self) -> bool {
        self.source_map
    }

    /// 设置 source map 的 less.js 兼容模式
    #[wasm_bindgen(setter, js_name = "sourceMapLessjsCompat")]
    pub fn set_source_map_lessjs_compat(&mut self, enabled: bool) {
        self.source_map_lessjs_compat = enabled;
    }

    /// 获取 source map less.js 兼容模式
    #[wasm_bindgen(getter, js_name = "sourceMapLessjsCompat")]
    pub fn source_map_lessjs_compat(&self) -> bool {
        self.source_map_lessjs_compat
    }
}

/// 编译结果
#[wasm_bindgen]
pub struct CompileResult {
    css: String,
    source_map: Option<String>,
    error: Option<String>,
}

#[wasm_bindgen]
impl CompileResult {
    /// 获取编译后的 CSS
    #[wasm_bindgen(getter)]
    pub fn css(&self) -> String {
        self.css.clone()
    }

    /// 获取 source map JSON（如果启用了 source map 生成）
    #[wasm_bindgen(getter, js_name = "sourceMap")]
    pub fn source_map(&self) -> Option<String> {
        self.source_map.clone()
    }

    /// 获取错误信息（如果有）
    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }

    /// 检查编译是否成功
    #[wasm_bindgen]
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }
}

/// 简单的 LESS 编译函数
///
/// # 参数
/// * `input` - LESS 源代码字符串
///
/// # 返回值
/// 返回编译结果，包含 CSS 或错误信息
#[wasm_bindgen]
pub fn compile_less(input: &str) -> CompileResult {
    match compile(input) {
        Ok(css) => CompileResult {
            css,
            source_map: None,
            error: None,
        },
        Err(e) => CompileResult {
            css: String::new(),
            source_map: None,
            error: Some(e.to_string()),
        },
    }
}

/// 带选项的 LESS 编译函数
///
/// # 参数
/// * `input` - LESS 源代码字符串
/// * `options` - 编译选项
///
/// # 返回值
/// 返回编译结果，包含 CSS 或错误信息
#[wasm_bindgen]
pub fn compile_less_with_options(input: &str, options: &WasmCompilerOptions) -> CompileResult {
    let mut compiler = if options.compress {
        Compiler::compressed()
    } else {
        Compiler::new()
    };

    if options.source_map {
        compiler = compiler.with_source_map(true);
        compiler.set_source_map_lessjs_compat(options.source_map_lessjs_compat);
    }

    match compiler.compile(input) {
        Ok(css) => {
            let source_map = compiler.generate_source_map();
            CompileResult {
                css,
                source_map,
                error: None,
            }
        }
        Err(e) => CompileResult {
            css: String::new(),
            source_map: None,
            error: Some(e.to_string()),
        },
    }
}

/// WASM 编译器类
#[wasm_bindgen]
pub struct WasmCompiler {
    inner: Compiler,
}

impl Default for WasmCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WasmCompiler {
    /// 创建新的编译器实例
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmCompiler {
        WasmCompiler {
            inner: Compiler::new(),
        }
    }

    /// 创建压缩模式的编译器
    #[wasm_bindgen]
    pub fn compressed() -> WasmCompiler {
        WasmCompiler {
            inner: Compiler::compressed(),
        }
    }

    /// 启用源码映射生成
    #[wasm_bindgen(js_name = "enableSourceMap")]
    pub fn enable_source_map(&mut self) {
        self.inner = Compiler::new().with_source_map(true);
    }

    /// 编译 LESS 代码
    #[wasm_bindgen]
    pub fn compile(&mut self, input: &str) -> CompileResult {
        match self.inner.compile(input) {
            Ok(css) => {
                let source_map = self.inner.generate_source_map();
                CompileResult {
                    css,
                    source_map,
                    error: None,
                }
            }
            Err(e) => CompileResult {
                css: String::new(),
                source_map: None,
                error: Some(e.to_string()),
            },
        }
    }
}

/// 获取编译器版本信息
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 获取编译器信息
#[wasm_bindgen]
pub fn info() -> String {
    format!(
        "Rust LESS 编译器 v{}\n用 Rust 编写的高性能 LESS 到 CSS 编译器",
        env!("CARGO_PKG_VERSION")
    )
}

// 为 JavaScript 提供的实用工具函数

/// 验证 LESS 语法是否正确（不进行完整编译）
#[wasm_bindgen]
pub fn validate_less(input: &str) -> bool {
    compile(input).is_ok()
}

/// 获取 LESS 编译错误的详细信息
#[wasm_bindgen]
pub fn get_error_details(input: &str) -> Option<String> {
    match compile(input) {
        Ok(_) => None,
        Err(e) => Some(format!("错误详情: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_less() {
        let less = "@color: red; .test { color: @color; }";
        let result = compile_less(less);
        assert!(result.is_success());
        assert!(result.css().contains("color: red"));
    }

    #[test]
    fn test_compile_with_options() {
        let less = "@color: red; .test { color: @color; }";
        let mut options = WasmCompilerOptions::new();
        options.set_compress(true);

        let result = compile_less_with_options(less, &options);
        assert!(result.is_success());
    }

    #[test]
    fn test_compiler_class() {
        let mut compiler = WasmCompiler::new();
        let less = "@color: blue; .header { color: @color; }";
        let result = compiler.compile(less);

        assert!(result.is_success());
        assert!(result.css().contains("color: blue"));
    }

    #[test]
    fn test_validation() {
        assert!(validate_less("@color: red; .test { color: @color; }"));
        assert!(!validate_less("@color red; .test { color: @color; }")); // 变量声明缺少冒号
    }

    #[test]
    fn test_version() {
        let version = version();
        assert!(!version.is_empty());
    }
}
