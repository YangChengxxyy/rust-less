//! LSP 补全上下文过滤 / 损坏文档回退符号位置 / 诊断 include_paths 注入的回归测试。
//! 通过内存连接驱动完整协议流程（与 test_lsp.rs 相同方式）。

#![cfg(feature = "lsp")]

use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::Location;
use lsp_types::PublishDiagnosticsParams;
use serde_json::{json, Value};

fn start_server() -> Connection {
    let (client, server) = Connection::memory();
    std::thread::spawn(move || {
        rust_less::lsp::Server::new().run(&server).unwrap();
    });
    client
}

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
            _ => continue,
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

fn open(client: &Connection, uri: &str, text: &str) {
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

/// 设置文档文本后在 (line, character) 处请求补全，返回标签列表。
/// `|` 在文本中仅表示一个占位非标识符字符，便于构造“紧跟触发字符”的场景。
fn completion(client: &Connection, uri: &str, line: u32, ch: u32, text: &str) -> Vec<String> {
    notify(
        client,
        "textDocument/didChange",
        json!({
            "textDocument": { "uri": uri, "version": 1 },
            "contentChanges": [{ "text": text }],
        }),
    );
    let _ = client.receiver.try_recv();
    let resp = request(
        client,
        10,
        "textDocument/completion",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": ch },
        }),
    );
    let items: Vec<Value> = serde_json::from_value(resp.result.unwrap()).unwrap();
    items
        .into_iter()
        .map(|i| i["label"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn completion_context_filters_by_trigger_char() {
    let client = start_server();
    let _ = request(&client, 1, "initialize", json!({}));
    notify(&client, "initialized", json!({}));
    let uri = "file:///workspace/styles/main.less";

    // 紧跟 `@`（行尾）→ 仅变量
    let labels = completion(&client, uri, 1, 13, "@a: 1;\n.b { color: @");
    assert!(labels.iter().all(|l| l.starts_with('@')), "{labels:?}");
    assert!(labels.contains(&"@a".to_string()));

    // 紧跟 `.` → 仅混入
    let labels = completion(&client, uri, 2, 6, ".mx() {}\n#ns {}\n.a { .| }");
    assert!(labels.iter().any(|l| l == ".mx"), "{labels:?}");
    assert!(!labels.iter().any(|l| l == "lighten"), "{labels:?}");

    // 紧跟 `#` → 仅命名空间混入
    let labels = completion(&client, uri, 2, 6, ".mx() {}\n#ns {}\n.a { #| }");
    assert!(labels.iter().any(|l| l == "#ns"), "{labels:?}");
    assert!(!labels.iter().any(|l| l == "lighten"), "{labels:?}");

    // 光标在已输入的标识符中间（@pr|imary）→ 不过滤，保持原行为
    let labels = completion(&client, uri, 1, 13, "@a: 1;\n.a { color: @primary; }");
    assert!(labels.iter().any(|l| l == "@a"), "{labels:?}");
    assert!(labels.iter().any(|l| l == "lighten"), "{labels:?}");

    // 其他位置 → 全部候选
    let labels = completion(&client, uri, 1, 3, "@a: 1;\n  |");
    assert!(labels.iter().any(|l| l == "@a"), "{labels:?}");
    assert!(labels.iter().any(|l| l == "lighten"), "{labels:?}");
}

#[test]
fn broken_doc_fallback_symbols_carry_real_positions() {
    // 未闭合块 → 解析失败 → 正则兜底；符号应带真实行列而非 (1,1)
    let client = start_server();
    let _ = request(&client, 1, "initialize", json!({}));
    notify(&client, "initialized", json!({}));
    let uri = "file:///workspace/styles/main.less";
    let text = ".broken {\n  @myvar: 5px;\n  color: @myvar;\n";
    open(&client, uri, text);
    let _ = client.receiver.try_recv();
    let resp = request(
        &client,
        20,
        "textDocument/definition",
        json!({"textDocument": {"uri": uri}, "position": {"line": 2, "character": 11}}),
    );
    let locs: Vec<Location> = serde_json::from_value(resp.result.unwrap()).unwrap();
    assert!(!locs.is_empty(), "fallback symbols should resolve @myvar");
    let r = locs[0].range;
    assert_eq!(r.start.line, 1, "var declared on line 2 (0-based 1): {r:?}");
    assert_eq!(r.start.character, 2, "column of '@myvar' on line 2: {r:?}");
}

#[test]
fn diagnostics_resolves_sibling_imports_via_include_paths() {
    // file:// URI 的文档：其所在目录应被注入 include path，
    // 使 @import "_dep.less" 找到同目录文件并发布空诊断
    let dir = std::env::temp_dir().join("rl_lsp_include_smoke");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("_dep.less"), "@c: blue;\n").unwrap();
    let uri = format!("file://{}", dir.join("main.less").display());
    let client = start_server();
    let _ = request(&client, 1, "initialize", json!({}));
    notify(&client, "initialized", json!({}));
    open(
        &client,
        &uri,
        "@import (less) \"_dep.less\";\n.a { color: @c; }\n",
    );
    loop {
        match client.receiver.recv().unwrap() {
            Message::Notification(n) if n.method == "textDocument/publishDiagnostics" => {
                let d: PublishDiagnosticsParams = serde_json::from_value(n.params).unwrap();
                assert!(
                    d.diagnostics.is_empty(),
                    "sibling @import should resolve via include path: {:?}",
                    d.diagnostics
                );
                break;
            }
            _ => continue,
        }
    }
}
