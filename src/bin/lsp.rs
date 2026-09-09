//! Rust LESS 编译器的 LSP 服务器二进制文件
//!
//! 通过 stdio 与 LSP 客户端通信，核心逻辑见 [`rust_less::lsp`]。

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (connection, io_threads) = lsp_server::Connection::stdio();
    let mut server = rust_less::lsp::Server::new();
    server.run(&connection)?;
    // LSP 约定：`exit` 前收到过 `shutdown` 时以 0 退出，否则以 1 退出。
    let code = if server.shutdown_received() { 0 } else { 1 };
    // 先释放连接持有的 sender，写入线程才能随通道关闭而结束；
    // 否则 `io_threads.join()` 会永久阻塞。
    drop(connection);
    io_threads.join()?;
    std::process::exit(code);
}
