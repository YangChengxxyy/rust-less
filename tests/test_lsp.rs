//! LSP 服务器的集成测试：通过内存连接驱动完整协议流程

#![cfg(feature = "lsp")]

use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    CompletionItem, DocumentSymbol, InitializeResult, Location, PublishDiagnosticsParams,
};
use serde_json::{json, Value};

fn start_server() -> Connection {
    let (client, server) = Connection::memory();
    std::thread::spawn(move || {
        rust_less::lsp::Server::new().run(&server).unwrap();
    });
    client
}

/// 发送请求并等待响应（跳过途中到达的通知）
fn request(client: &Connection, id: i32, method: &str, params: Value) -> Response {
    client
        .sender
        .send(Message::Request(Request::new(
            id.into(),
            method.to_string(),
            params,
        )))
        .unwrap();
    loop {
        match client.receiver.recv().unwrap() {
            Message::Response(r) => return r,
            Message::Notification(_) => continue,
            Message::Request(_) => continue,
        }
    }
}

fn notify(client: &Connection, method: &str, params: Value) {
    client
        .sender
        .send(Message::Notification(Notification::new(
            method.to_string(),
            params,
        )))
        .unwrap();
}

/// 等待下一条诊断通知
fn wait_diagnostics(client: &Connection) -> PublishDiagnosticsParams {
    loop {
        match client.receiver.recv().unwrap() {
            Message::Notification(n) if n.method == "textDocument/publishDiagnostics" => {
                return serde_json::from_value(n.params).unwrap();
            }
            _ => continue,
        }
    }
}

fn did_open(client: &Connection, uri: &str, text: &str) {
    notify(
        client,
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "less",
                "version": 1,
                "text": text,
            }
        }),
    );
}

fn did_change(client: &Connection, uri: &str, version: i32, text: &str) {
    notify(
        client,
        "textDocument/didChange",
        json!({
            "textDocument": { "uri": uri, "version": version },
            "contentChanges": [{ "text": text }],
        }),
    );
}

const URI: &str = "file:///workspace/styles/main.less";

#[test]
fn initialize_handshake() {
    let client = start_server();
    let resp = request(&client, 1, "initialize", json!({ "capabilities": {} }));
    assert!(resp.error.is_none(), "initialize failed: {:?}", resp.error);
    let init: InitializeResult = serde_json::from_value(resp.result.unwrap()).unwrap();
    let info = init.server_info.expect("serverInfo missing");
    assert_eq!(info.name, "rust-less-lsp");
    assert!(info.version.is_some());
    let sync = init
        .capabilities
        .text_document_sync
        .expect("sync capability");
    let kind = serde_json::to_value(&sync).unwrap();
    assert_eq!(kind, json!(1), "expected full text sync (kind 1)");
    // shutdown → null
    let resp = request(&client, 2, "shutdown", Value::Null);
    assert!(resp.error.is_none());
    assert_eq!(resp.result.unwrap(), Value::Null);
    notify(&client, "exit", Value::Null);
}

#[test]
fn diagnostics_lifecycle() {
    let client = start_server();
    let _ = request(&client, 1, "initialize", json!({}));
    notify(&client, "initialized", json!({}));

    // 有效文档 → 空诊断
    did_open(
        &client,
        URI,
        "@primary: #ff0000;\n.a { color: @primary; }\n",
    );
    let diags = wait_diagnostics(&client);
    assert_eq!(diags.uri.as_str(), URI);
    assert!(diags.diagnostics.is_empty(), "expected no diagnostics");

    // 未定义变量 → 一条错误诊断
    did_change(&client, URI, 2, ".a { color: @missing; }\n");
    let diags = wait_diagnostics(&client);
    assert_eq!(diags.diagnostics.len(), 1, "expected one diagnostic");
    let d = &diags.diagnostics[0];
    assert!(d.message.contains("missing"), "message: {}", d.message);
    let r = d.range;
    assert!(
        r.start.line == 0 && r.end.line == 0,
        "range should be in doc: {r:?}"
    );

    // 修复 → 空诊断
    did_change(
        &client,
        URI,
        3,
        "@primary: #ff0000;\n.a { color: @primary; }\n",
    );
    let diags = wait_diagnostics(&client);
    assert!(diags.diagnostics.is_empty(), "expected clean diagnostics");

    // completion 包含 @primary 与内置函数
    let resp = request(
        &client,
        10,
        "textDocument/completion",
        json!({
            "textDocument": { "uri": URI },
            "position": { "line": 1, "character": 13 },
        }),
    );
    let items: Vec<CompletionItem> = serde_json::from_value(resp.result.unwrap()).unwrap();
    assert!(
        items.iter().any(|i| i.label == "@primary"),
        "completion missing @primary"
    );
    assert!(
        items.iter().any(|i| i.label == "lighten"),
        "completion missing builtins"
    );

    // hover @primary → 声明文本
    let resp = request(
        &client,
        11,
        "textDocument/hover",
        json!({
            "textDocument": { "uri": URI },
            "position": { "line": 1, "character": 14 },
        }),
    );
    let hover = resp.result.unwrap();
    let text = hover.to_string();
    assert!(
        text.contains("less"),
        "hover not markdown code block: {text}"
    );
    assert!(text.contains("primary"), "hover missing var name: {text}");
    assert!(text.contains("#ff0000"), "hover missing value: {text}");

    // documentSymbol 包含 @primary
    let resp = request(
        &client,
        12,
        "textDocument/documentSymbol",
        json!({ "textDocument": { "uri": URI } }),
    );
    let symbols: Vec<DocumentSymbol> = serde_json::from_value(resp.result.unwrap()).unwrap();
    assert!(
        symbols.iter().any(|s| s.name.contains("primary")),
        "symbols missing variable: {symbols:?}"
    );

    // definition 指向声明行（第 0 行）
    let resp = request(
        &client,
        13,
        "textDocument/definition",
        json!({
            "textDocument": { "uri": URI },
            "position": { "line": 1, "character": 14 },
        }),
    );
    let locs: Vec<Location> = serde_json::from_value(resp.result.unwrap()).unwrap();
    assert_eq!(locs.len(), 1);
    assert_eq!(locs[0].uri.as_str(), URI);
    assert_eq!(
        locs[0].range.start.line, 0,
        "definition should point at declaration line"
    );

    // didClose → 清空诊断
    notify(
        &client,
        "textDocument/didClose",
        json!({ "textDocument": { "uri": URI } }),
    );
    let diags = wait_diagnostics(&client);
    assert!(
        diags.diagnostics.is_empty(),
        "didClose should clear diagnostics"
    );

    // shutdown + exit → 循环退出（由线程正常结束保证，这里再发一次 shutdown 验证仍可响应）
    let resp = request(&client, 20, "shutdown", Value::Null);
    assert!(resp.error.is_none());
    notify(&client, "exit", Value::Null);
}

#[test]
fn unknown_method_returns_method_not_found() {
    let client = start_server();
    let _ = request(&client, 1, "initialize", json!({}));
    let resp = request(&client, 2, "textDocument/unknownFeature", json!({}));
    let err = resp.error.expect("expected error");
    assert_eq!(err.code, -32601, "expected MethodNotFound");
    notify(&client, "exit", Value::Null);
}

#[test]
fn definition_resolves_relative_import() {
    let dir = std::env::temp_dir().join("rust-less-lsp-test");
    std::fs::create_dir_all(&dir).unwrap();
    let lib = dir.join("lib.less");
    std::fs::write(&lib, "@imported: #00ff00;\n").unwrap();
    let lib_uri = lsp_types::Url::from_file_path(&lib).unwrap().to_string();
    let main_uri = lsp_types::Url::from_file_path(dir.join("main.less"))
        .unwrap()
        .to_string();

    let client = start_server();
    let _ = request(&client, 1, "initialize", json!({}));
    did_open(
        &client,
        &main_uri,
        "@import \"lib.less\";\n.a { color: @imported; }\n",
    );
    let _ = wait_diagnostics(&client);

    let resp = request(
        &client,
        2,
        "textDocument/definition",
        json!({
            "textDocument": { "uri": main_uri },
            "position": { "line": 1, "character": 14 },
        }),
    );
    let locs: Vec<Location> = serde_json::from_value(resp.result.unwrap()).unwrap();
    assert_eq!(locs.len(), 1);
    assert_eq!(locs[0].uri.as_str(), lib_uri);
    assert_eq!(locs[0].range.start.line, 0);

    notify(&client, "shutdown", Value::Null);
    notify(&client, "exit", Value::Null);
}

/// 驱动真实二进制的协议级回归测试：
/// `shutdown` 后 `exit` 必须以 0 退出；未 `shutdown` 直接 `exit` 必须以 1 退出。
/// （回归：`run()` 返回后未释放 `Connection`，写入线程不退出，
/// `io_threads.join()` 永久阻塞导致进程悬挂。）
#[test]
fn stdio_binary_exit_lifecycle() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    fn run_scenario(with_shutdown: bool) -> i32 {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rust-less-lsp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn rust-less-lsp");
        let mut stdin = child.stdin.take().unwrap();
        let mut send = |msg: Value| {
            let body = msg.to_string();
            write!(stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
            stdin.flush().unwrap();
        };
        send(json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"processId": null, "rootUri": null, "capabilities": {}}
        }));
        if with_shutdown {
            send(json!({"jsonrpc": "2.0", "id": 2, "method": "shutdown"}));
        }
        send(json!({"jsonrpc": "2.0", "method": "exit"}));
        drop(stdin);

        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            match child.try_wait().expect("try_wait") {
                Some(status) => break status.code().unwrap_or(-1),
                None => {
                    assert!(
                        Instant::now() < deadline,
                        "LSP 进程在 exit 后未退出（悬挂）"
                    );
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    assert_eq!(run_scenario(true), 0, "shutdown 后 exit 必须以 0 退出");
    assert_eq!(run_scenario(false), 1, "未 shutdown 的 exit 必须以 1 退出");
}
