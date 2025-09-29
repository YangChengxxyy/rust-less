//! WebAssembly 绑定模块
//!
//! 此模块提供了将 Rust LESS 编译器暴露给 JavaScript 的 WebAssembly 接口。

#[cfg(feature = "wasm")]
pub mod bindings;

#[cfg(feature = "wasm")]
pub use bindings::*;
