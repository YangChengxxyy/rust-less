//! LESS 的 Language Server Protocol 服务器实现
//!
//! 此模块在 `lsp` feature 下提供基于 `rust_less` 编译器与解析器的
//! 语言服务器核心逻辑。服务器以可测试的 `Server` 结构体实现，
//! 通过泛型 [`lsp_server::Connection`] 通信（既支持 stdio 也支持内存连接）。
//!
//! 支持的能力：
//! - 全量文本同步（didOpen / didChange / didClose）与诊断发布
//! - 补全（文档内变量、混入、命名空间 + 内置函数表）
//! - 悬停（变量声明、混入签名、内置函数签名）
//! - 文档符号（层级化的变量 / 混入 / 命名空间 / 规则 / at-rule）
//! - 定义跳转（当前文档与简单的相对 `@import` 文件解析）

use std::collections::HashMap;
use std::panic::AssertUnwindSafe;

use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::CompletionParams;
use lsp_types::{
    CompletionItem, CompletionItemKind, Diagnostic, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, GotoDefinitionParams, Hover,
    HoverContents, HoverParams, InitializeParams, InitializeResult, Location, MarkedString,
    MarkedString::LanguageString, OneOf, Position, PublishDiagnosticsParams, Range,
    ServerCapabilities, ServerInfo, SymbolKind, TextDocumentSyncCapability, TextDocumentSyncKind,
};
use regex::Regex;
use serde_json::Value;

use crate::ast::{Expression, ImportType, Selector, SimpleSelector, Statement};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::{compile_with_options, CompilerOptions};

/// 内置 LESS 函数表：(名称, 简短签名与说明)
static BUILTIN_FUNCTIONS: &[(&str, &str)] = &[
    ("lighten", "lighten(@color, @amount) — 按比例增加颜色亮度"),
    ("darken", "darken(@color, @amount) — 按比例降低颜色亮度"),
    ("saturate", "saturate(@color, @amount) — 增加颜色饱和度"),
    ("desaturate", "desaturate(@color, @amount) — 降低颜色饱和度"),
    ("fade", "fade(@color, @amount) — 设置颜色的透明度"),
    (
        "fadein",
        "fadein(@color, @amount) — 降低颜色透明度（更不透明）",
    ),
    (
        "fadeout",
        "fadeout(@color, @amount) — 增加颜色透明度（更透明）",
    ),
    ("mix", "mix(@color1, @color2, @weight) — 按权重混合两个颜色"),
    ("spin", "spin(@color, @angle) — 旋转颜色的色相角度"),
    ("greyscale", "greyscale(@color) — 完全去饱和，得到灰度色"),
    (
        "contrast",
        "contrast(@color, @dark, @light, @threshold) — 挑选对比度更高的颜色",
    ),
    ("round", "round(@number, @decimalPlaces) — 四舍五入"),
    ("ceil", "ceil(@number) — 向上取整"),
    ("floor", "floor(@number) — 向下取整"),
    ("percentage", "percentage(@number) — 将小数转换为百分比"),
    ("abs", "abs(@number) — 绝对值"),
    ("min", "min(@a, @b, ...) — 取最小值"),
    ("max", "max(@a, @b, ...) — 取最大值"),
    ("pow", "pow(@base, @exponent) — 幂运算"),
    ("sqrt", "sqrt(@number) — 平方根"),
    ("mod", "mod(@a, @b) — 取模"),
    ("length", "length(@list) — 列表长度"),
    ("extract", "extract(@list, @index) — 按下标提取列表元素"),
    ("range", "range(@start, @end, @step) — 生成数值列表"),
    ("e", "e(@expression) — 原样输出 CSS，不做引号处理"),
    ("escape", "escape(@string) — URL 转义字符串"),
    (
        "replace",
        "replace(@string, @pattern, @replacement) — 正则替换字符串",
    ),
    ("unit", "unit(@number, @unit) — 修改或移除数字的单位"),
    ("color", "color(@string) — 将字符串解析为颜色"),
    ("red", "red(@color) — 提取红色通道 (0-255)"),
    ("green", "green(@color) — 提取绿色通道 (0-255)"),
    ("blue", "blue(@color) — 提取蓝色通道 (0-255)"),
    ("hue", "hue(@color) — 提取色相角度 (0-360)"),
    ("saturation", "saturation(@color) — 提取饱和度百分比"),
    ("lightness", "lightness(@color) — 提取亮度百分比"),
    ("alpha", "alpha(@color) — 提取透明度 (0-1)"),
    ("luma", "luma(@color) — 计算感知亮度 (luma) 值"),
    ("map-get", "map-get(@map, @key) — 读取映射中指定键的值"),
    ("map-merge", "map-merge(@map1, @map2) — 合并两个映射"),
    ("each", "each(@list, @rules) — 遍历列表并展开规则块"),
    ("if", "if(@condition, @then, @else) — 编译期条件表达式"),
    ("boolean", "boolean(@condition) — 构造布尔值"),
    ("isnumber", "isnumber(@value) — 判断值是否为数字"),
    ("isstring", "isstring(@value) — 判断值是否为字符串"),
    ("iscolor", "iscolor(@value) — 判断值是否为颜色"),
    ("isurl", "isurl(@value) — 判断值是否为 url() 表达式"),
    ("ispixel", "ispixel(@value) — 判断值是否为 px 单位"),
    ("isem", "isem(@value) — 判断值是否为 em 单位"),
    ("ispercentage", "ispercentage(@value) — 判断值是否为百分比"),
    ("isunit", "isunit(@value, @unit) — 判断值是否为指定单位"),
];

/// 文档中收集到的变量符号
#[derive(Debug, Clone)]
struct VarSymbol {
    /// 变量名（不含 @）
    name: String,
    /// 声明右侧的源文本
    value_text: String,
    /// 声明位置（1-based）
    line: usize,
    column: usize,
}

/// 文档中收集到的混入 / 命名空间符号
#[derive(Debug, Clone)]
struct MixinSymbol {
    /// 前缀：`.` 表示混入，`#` 表示命名空间
    prefix: char,
    /// 名称
    name: String,
    /// 参数文本（命名空间为空）
    params_text: String,
    /// 声明位置（1-based）
    line: usize,
    column: usize,
}

/// 从文档中提取的符号集合
#[derive(Debug, Default)]
struct DocumentSymbols {
    /// 变量声明
    vars: Vec<VarSymbol>,
    /// 混入与命名空间
    mixins: Vec<MixinSymbol>,
}

/// LESS 语言服务器状态
pub struct Server {
    /// 已打开文档：URI -> (文本, 版本)
    docs: HashMap<lsp_types::Url, (String, i32)>,
    /// 是否已收到 `shutdown` 请求（决定 `exit` 后的进程退出码）
    shutdown_received: bool,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

/// 将 crate 的 1-based 位置转换为 LSP 0-based 位置
fn lsp_pos(line: usize, column: usize) -> Position {
    Position::new(
        (line as u32).saturating_sub(1),
        (column as u32).saturating_sub(1),
    )
}

/// 获取某一行文本
fn line_text(text: &str, line0: u32) -> Option<&str> {
    text.lines().nth(line0 as usize)
}

/// 由字节偏移计算 1-based (line, column)，与解析器位置约定一致
fn offset_to_pos(text: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    for (i, ch) in text.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

/// 提取光标处（含边界）的标识符单词，保留 `@`/`.`/`#` 前缀
fn word_at(text: &str, line: u32, character: u32) -> Option<String> {
    let l = line_text(text, line)?;
    let chars: Vec<char> = l.chars().collect();
    let mut idx = (character as usize).min(chars.len().saturating_sub(1));
    let is_word = |c: char| c.is_alphanumeric() || c == '_' || c == '-';

    // 光标可能位于单词之后
    if idx < chars.len() && !is_word(chars[idx]) && (idx == 0 || !is_word(chars[idx - 1])) {
        // 光标在前缀符上（如 @ 或 .），向后看
        if !matches!(chars[idx], '@' | '.' | '#') {
            return None;
        }
    }
    if idx < chars.len() && !is_word(chars[idx]) && !matches!(chars[idx], '@' | '.' | '#') {
        if idx > 0 && (is_word(chars[idx - 1]) || matches!(chars[idx - 1], '@' | '.' | '#')) {
            idx -= 1;
        } else {
            return None;
        }
    }

    let mut start = idx;
    while start > 0 && is_word(chars[start - 1]) {
        start -= 1;
    }
    // 向前纳入前缀符
    if start > 0 && matches!(chars[start - 1], '@' | '.' | '#') {
        start -= 1;
    }
    if matches!(chars[start], '@' | '.' | '#')
        && start + 1 < chars.len()
        && !is_word(chars[start + 1])
    {
        return None; // 只有孤立的前缀符
    }
    let mut end = idx;
    while end < chars.len() && is_word(chars[end]) {
        end += 1;
    }
    if end <= start {
        return None;
    }
    Some(chars[start..end].iter().collect())
}

/// 将表达式渲染回近似的 LESS 源文本
fn expr_to_string(expr: &Expression) -> String {
    use Expression as E;
    match expr {
        E::String { value, quoted, .. } => {
            if *quoted {
                format!("\"{value}\"")
            } else {
                value.clone()
            }
        }
        E::Number { value, unit, .. } => match unit {
            Some(u) => format!("{}{}", trim_f64(*value), u),
            None => trim_f64(*value),
        },
        E::Color {
            original,
            red,
            green,
            blue,
            ..
        } => original
            .clone()
            .unwrap_or_else(|| format!("#{:02x}{:02x}{:02x}", red, green, blue)),
        E::Boolean(b, _) => b.to_string(),
        E::Variable(name, _) => format!("@{name}"),
        E::BinaryOp {
            left,
            operator,
            right,
            ..
        } => format!(
            "{} {} {}",
            expr_to_string(left),
            operator,
            expr_to_string(right)
        ),
        E::UnaryOp {
            operator, operand, ..
        } => format!("{}{}", operator, expr_to_string(operand)),
        E::FunctionCall {
            name, arguments, ..
        } => {
            let args: Vec<String> = arguments.iter().map(expr_to_string).collect();
            format!("{}({})", name, args.join(", "))
        }
        E::Parenthesized(inner, _) => format!("({})", expr_to_string(inner)),
        E::List { values, .. } => values
            .iter()
            .map(expr_to_string)
            .collect::<Vec<_>>()
            .join(" "),
        E::Interpolation(name, _) => format!("@{{{name}}}"),
        E::PropertyInterpolation(name, _) => format!("@{{{name}}}"),
        E::SelectorInterpolation(name, _) => format!("@{{{name}}}"),
        E::Url(url, _) => format!("url(\"{url}\")"),
        E::TemplateString { parts, .. } => {
            use crate::ast::TemplateStringPart as P;
            let mut out = String::new();
            for p in parts {
                match p {
                    P::Text(t) => out.push_str(t),
                    P::Interpolation(name) => out.push_str(&format!("@{{{name}}}")),
                }
            }
            out
        }
        E::Dimension {
            value,
            from_unit,
            to_unit,
            ..
        } => {
            format!(
                "{}{} -> {}{}",
                trim_f64(*value),
                from_unit,
                trim_f64(*value),
                to_unit
            )
        }
        E::Percentage(v, _) => format!("{}%", trim_f64(*v)),
        E::Null(_) => "null".to_string(),
        E::MapAccess { map, key, .. } => {
            format!("{}[{}]", expr_to_string(map), expr_to_string(key))
        }
        E::Conditional {
            condition,
            true_value,
            false_value,
            ..
        } => format!(
            "if({}, {}, {})",
            expr_to_string(condition),
            expr_to_string(true_value),
            expr_to_string(false_value)
        ),
        E::PropertyAccess {
            object, property, ..
        } => {
            format!("{}.{}", expr_to_string(object), property)
        }
        E::Escaped(s, _) => format!("~\"{s}\""),
        E::Anonymous(s, _) => s.clone(),
        E::JavaScript(s, _) => format!("`{s}`"),
        E::MapLiteral { entries, .. } => {
            let items: Vec<String> = entries
                .iter()
                .map(|(k, v, _)| format!("{}: {}", k, expr_to_string(v)))
                .collect();
            format!("{{ {} }}", items.join("; "))
        }
        E::DetachedRuleset { .. } => "{ ... }".to_string(),
    }
}

/// 格式化 f64，去掉多余的小数零
fn trim_f64(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

/// 将混入参数列表渲染为 `(@a: 1; @b)` 形式
fn params_to_string(params: &[crate::ast::MixinParameter]) -> String {
    let items: Vec<String> = params
        .iter()
        .map(|p| {
            let mut s = format!("@{}", p.name);
            if p.variadic {
                s.push_str("...");
            }
            if let Some(d) = &p.default_value {
                s.push_str(&format!(": {}", expr_to_string(d)));
            }
            s
        })
        .collect();
    if items.is_empty() {
        String::new()
    } else {
        format!("({})", items.join("; "))
    }
}

/// 判断规则是否为 `#ns { ... }` 形式的命名空间，返回其名称
fn namespace_name(selectors: &[Selector]) -> Option<String> {
    if selectors.len() != 1 {
        return None;
    }
    let sel = &selectors[0];
    if sel.parts.len() != 1 {
        return None;
    }
    let part = &sel.parts[0];
    if part.simple_selectors.len() != 1 {
        return None;
    }
    match &part.simple_selectors[0] {
        SimpleSelector::Id { name, .. } => Some(name.clone()),
        _ => None,
    }
}

/// 递归收集语句列表中的符号
fn collect_symbols(stmts: &[Statement], out: &mut DocumentSymbols) {
    for stmt in stmts {
        match stmt {
            Statement::Variable(v) => out.vars.push(VarSymbol {
                name: v.name.clone(),
                value_text: expr_to_string(&v.value),
                line: v.position.line,
                column: v.position.column,
            }),
            Statement::MixinDefinition(m) => {
                out.mixins.push(MixinSymbol {
                    prefix: '.',
                    name: m.name.clone(),
                    params_text: params_to_string(&m.parameters),
                    line: m.position.line,
                    column: m.position.column,
                });
                collect_symbols(&m.body, out);
            }
            Statement::Rule(r) => {
                if let Some(ns) = namespace_name(&r.selectors) {
                    out.mixins.push(MixinSymbol {
                        prefix: '#',
                        name: ns,
                        params_text: String::new(),
                        line: r.position.line,
                        column: r.position.column,
                    });
                }
                collect_symbols(&r.nested_rules, out);
            }
            Statement::AtRule(a) => {
                if let Some(block) = &a.block {
                    collect_symbols(block, out);
                }
            }
            Statement::EachCall(e) => collect_symbols(&e.body, out),
            _ => {}
        }
    }
}

/// 解析文档并收集符号；解析失败时回退到正则扫描
fn symbols_for_text(text: &str) -> DocumentSymbols {
    let mut out = DocumentSymbols::default();
    let parsed = Parser::new(Lexer::new(text.to_string())).and_then(|mut p| p.parse());
    if let Ok(stylesheet) = parsed {
        collect_symbols(&stylesheet.statements, &mut out);
    } else {
        // 语法错误（正在输入）时用正则兜底，保证补全不中断
        let var_re = Regex::new(r"@([\w-]+)\s*:").unwrap();
        let mixin_re = Regex::new(r"(?m)^\s*\.([\w-]+)\s*\(").unwrap();
        let ns_re = Regex::new(r"(?m)^\s*#([\w-]+)\s*\{").unwrap();
        for cap in var_re.captures_iter(text) {
            let (line, column) = offset_to_pos(text, cap.get(0).unwrap().start());
            out.vars.push(VarSymbol {
                name: cap[1].to_string(),
                value_text: String::new(),
                line,
                column,
            });
        }
        for cap in mixin_re.captures_iter(text) {
            let (line, column) = offset_to_pos(text, cap.get(1).unwrap().start());
            out.mixins.push(MixinSymbol {
                prefix: '.',
                name: cap[1].to_string(),
                params_text: String::new(),
                line,
                column,
            });
        }
        for cap in ns_re.captures_iter(text) {
            let (line, column) = offset_to_pos(text, cap.get(1).unwrap().start());
            out.mixins.push(MixinSymbol {
                prefix: '#',
                name: cap[1].to_string(),
                params_text: String::new(),
                line,
                column,
            });
        }
        out.vars.sort_by(|a, b| a.name.cmp(&b.name));
        out.vars.dedup_by(|a, b| a.name == b.name);
        out.mixins
            .sort_by(|a, b| (a.prefix, &a.name).cmp(&(b.prefix, &b.name)));
        out.mixins
            .dedup_by(|a, b| a.prefix == b.prefix && a.name == b.name);
    }
    out
}

impl Server {
    /// 创建新的服务器实例
    pub fn new() -> Self {
        Self {
            docs: HashMap::new(),
            shutdown_received: false,
        }
    }

    /// 是否已收到 `shutdown` 请求。
    ///
    /// LSP 约定：`exit` 前收到过 `shutdown` 时进程以 0 退出，否则以 1 退出。
    pub fn shutdown_received(&self) -> bool {
        self.shutdown_received
    }

    /// 运行主消息循环，直到收到 `exit` 通知或连接关闭。
    ///
    /// 返回 `Err` 仅当底层通道关闭（对端退出）。
    pub fn run(&mut self, connection: &Connection) -> std::io::Result<()> {
        for msg in &connection.receiver {
            match msg {
                Message::Request(req) => self.handle_request(connection, req),
                Message::Notification(not) => {
                    if not.method == "exit" {
                        return Ok(());
                    }
                    self.handle_notification(connection, not);
                }
                Message::Response(_) => {}
            }
        }
        Ok(())
    }

    /// 处理一条请求；handler panic 时返回内部错误，不让循环崩溃
    fn handle_request(&mut self, connection: &Connection, req: Request) {
        let id = req.id.clone();
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            self.dispatch_request(req.method.as_str(), req.params.clone())
        }));
        match result {
            Ok(Ok(Some(value))) => {
                let _ = connection
                    .sender
                    .send(Message::Response(Response::new_ok(id, value)));
            }
            Ok(Ok(None)) => {
                let _ = connection
                    .sender
                    .send(Message::Response(Response::new_ok(id, Value::Null)));
            }
            Ok(Err(msg)) => {
                let _ = connection.sender.send(Message::Response(Response::new_err(
                    id,
                    lsp_server::ErrorCode::MethodNotFound as i32,
                    msg,
                )));
            }
            Err(_) => {
                let _ = connection.sender.send(Message::Response(Response::new_err(
                    id,
                    lsp_server::ErrorCode::InternalError as i32,
                    "internal server error (panic caught)".to_string(),
                )));
            }
        }
    }

    /// 分发请求；`Err` 表示 MethodNotFound
    #[allow(clippy::too_many_lines)]
    fn dispatch_request(&mut self, method: &str, params: Value) -> Result<Option<Value>, String> {
        match method {
            "initialize" => {
                let _params: InitializeParams =
                    serde_json::from_value(params).map_err(|e| e.to_string())?;
                let result = InitializeResult {
                    capabilities: ServerCapabilities {
                        text_document_sync: Some(TextDocumentSyncCapability::Kind(
                            TextDocumentSyncKind::FULL,
                        )),
                        completion_provider: Some(lsp_types::CompletionOptions::default()),
                        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
                        document_symbol_provider: Some(OneOf::Left(true)),
                        definition_provider: Some(OneOf::Left(true)),
                        ..ServerCapabilities::default()
                    },
                    server_info: Some(ServerInfo {
                        name: "rust-less-lsp".to_string(),
                        version: Some(env!("CARGO_PKG_VERSION").to_string()),
                    }),
                };
                serde_json::to_value(result)
                    .map(Some)
                    .map_err(|e| e.to_string())
            }
            "shutdown" => {
                self.shutdown_received = true;
                Ok(Some(Value::Null))
            }
            "textDocument/completion" => {
                let params: CompletionParams =
                    serde_json::from_value(params).map_err(|e| e.to_string())?;
                let uri = params.text_document_position.text_document.uri;
                let pos = params.text_document_position.position;
                let items = self.completion(&uri, pos);
                serde_json::to_value(items)
                    .map(Some)
                    .map_err(|e| e.to_string())
            }
            "textDocument/hover" => {
                let params: HoverParams =
                    serde_json::from_value(params).map_err(|e| e.to_string())?;
                let uri = params.text_document_position_params.text_document.uri;
                let pos = params.text_document_position_params.position;
                let hover = self.hover(&uri, pos);
                serde_json::to_value(hover)
                    .map(Some)
                    .map_err(|e| e.to_string())
            }
            "textDocument/documentSymbol" => {
                let params: DocumentSymbolParams =
                    serde_json::from_value(params).map_err(|e| e.to_string())?;
                let uri = params.text_document.uri;
                let symbols = self.document_symbols(&uri);
                serde_json::to_value(symbols)
                    .map(Some)
                    .map_err(|e| e.to_string())
            }
            "textDocument/definition" => {
                let params: GotoDefinitionParams =
                    serde_json::from_value(params).map_err(|e| e.to_string())?;
                let uri = params.text_document_position_params.text_document.uri;
                let pos = params.text_document_position_params.position;
                let locs = self.definition(&uri, pos);
                serde_json::to_value(locs)
                    .map(Some)
                    .map_err(|e| e.to_string())
            }
            _ => Err(format!("method not found: {method}")),
        }
    }
    /// 处理通知消息
    fn handle_notification(&mut self, connection: &Connection, not: Notification) {
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| match not.method.as_str() {
            "textDocument/didOpen" => {
                if let Ok(params) = serde_json::from_value::<DidOpenTextDocumentParams>(not.params)
                {
                    self.docs.insert(
                        params.text_document.uri.clone(),
                        (
                            params.text_document.text.clone(),
                            params.text_document.version,
                        ),
                    );
                    self.publish_diagnostics(connection, &params.text_document.uri);
                }
            }
            "textDocument/didChange" => {
                if let Ok(params) =
                    serde_json::from_value::<DidChangeTextDocumentParams>(not.params)
                {
                    let uri = params.text_document.uri.clone();
                    let version = params.text_document.version;
                    // 全量同步：取最后一个变更事件的完整文本
                    let text = params
                        .content_changes
                        .last()
                        .map(|c| c.text.clone())
                        .or_else(|| self.docs.get(&uri).map(|(t, _)| t.clone()));
                    if let Some(text) = text {
                        self.docs.insert(uri.clone(), (text, version));
                        self.publish_diagnostics(connection, &uri);
                    }
                }
            }
            "textDocument/didClose" => {
                if let Ok(params) = serde_json::from_value::<DidCloseTextDocumentParams>(not.params)
                {
                    let uri = params.text_document.uri.clone();
                    self.docs.remove(&uri);
                    // 关闭时清空诊断
                    let _ = connection
                        .sender
                        .send(Message::Notification(Notification::new(
                            "textDocument/publishDiagnostics".to_string(),
                            serde_json::to_value(PublishDiagnosticsParams {
                                uri,
                                diagnostics: Vec::new(),
                                version: None,
                            })
                            .unwrap_or(Value::Null),
                        )));
                }
            }
            _ => {}
        }));
        // 通知处理 panic 时仅记录并继续循环
        if result.is_err() {
            eprintln!(
                "rust-less-lsp: notification handler panicked: {}",
                not.method
            );
        }
    }

    /// 编译文档并向客户端发布诊断
    fn publish_diagnostics(&self, connection: &Connection, uri: &lsp_types::Url) {
        let text = match self.docs.get(uri) {
            Some((t, _)) => t.clone(),
            None => return,
        };
        // 文件 URI 时把文档所在目录与进程工作目录注入导入搜索路径，
        // 使 `@import "sibling.less"` 能找到同级文件（仅工作区根级，不做项目配置发现）
        let mut options = CompilerOptions::default();
        if let Ok(file) = uri.to_file_path() {
            if let Some(dir) = file.parent() {
                options.include_paths.push(dir.display().to_string());
            }
        }
        if let Ok(cwd) = std::env::current_dir() {
            options.include_paths.push(cwd.display().to_string());
        }
        let diagnostics = match compile_with_options(&text, options) {
            Ok(_) => Vec::new(),
            Err(e) => vec![diagnostic_from_error(&e, &text)],
        };
        let params = PublishDiagnosticsParams {
            uri: uri.clone(),
            diagnostics,
            version: None,
        };
        let _ = connection
            .sender
            .send(Message::Notification(Notification::new(
                "textDocument/publishDiagnostics".to_string(),
                serde_json::to_value(params).unwrap_or(Value::Null),
            )));
    }

    /// 补全：文档符号 + 内置函数。
    ///
    /// 按光标前紧邻的字符过滤上下文：`@` → 仅变量，
    /// `.`/`#` → 仅混入/命名空间，其余 → 全部候选。
    fn completion(&self, uri: &lsp_types::Url, pos: Position) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        if let Some((text, _)) = self.docs.get(uri) {
            // 仅当光标紧邻触发字符且尚未输入标识符时按上下文过滤；
            // `@` → 仅变量，`.`/`#` → 仅混入/命名空间
            let chars_of = |l: &str| {
                let chars: Vec<char> = l.chars().collect();
                let prev = (pos.character > 0 && (pos.character as usize) <= chars.len())
                    .then(|| chars[pos.character as usize - 1]);
                let next = chars.get(pos.character as usize).copied();
                (prev, next)
            };
            let (prev, next) = line_text(text, pos.line)
                .map(chars_of)
                .unwrap_or((None, None));
            let typing_ident = next.map(|c| c.is_alphanumeric() || c == '-' || c == '_');
            let fresh_trigger = typing_ident != Some(true);
            let want_vars = !(matches!(prev, Some('.') | Some('#')) && fresh_trigger);
            let want_mixins = !(prev == Some('@') && fresh_trigger);
            let symbols = symbols_for_text(text);
            if want_vars {
                for v in &symbols.vars {
                    items.push(CompletionItem {
                        label: format!("@{}", v.name),
                        kind: Some(CompletionItemKind::VARIABLE),
                        detail: (!v.value_text.is_empty())
                            .then(|| format!("{}: {}", v.name, v.value_text)),
                        ..CompletionItem::default()
                    });
                }
            }
            if want_mixins {
                for m in &symbols.mixins {
                    items.push(CompletionItem {
                        label: format!("{}{}", m.prefix, m.name),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: (!m.params_text.is_empty())
                            .then(|| format!("{}{}{}", m.prefix, m.name, m.params_text)),
                        ..CompletionItem::default()
                    });
                }
            }
            if want_vars && want_mixins {
                for (name, doc) in BUILTIN_FUNCTIONS {
                    items.push(CompletionItem {
                        label: (*name).to_string(),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some((*doc).to_string()),
                        ..CompletionItem::default()
                    });
                }
            }
        }
        items
    }

    /// 悬停：变量声明 / 混入签名 / 内置函数签名
    fn hover(&self, uri: &lsp_types::Url, pos: Position) -> Option<Hover> {
        let (text, _) = self.docs.get(uri)?;
        let word = word_at(text, pos.line, pos.character)?;
        let symbols = symbols_for_text(text);
        let content: MarkedString = if let Some(name) = word.strip_prefix('@') {
            let v = symbols.vars.iter().find(|v| v.name == name)?;
            LanguageString(lsp_types::LanguageString {
                language: "less".to_string(),
                value: if v.value_text.is_empty() {
                    format!("@{name};")
                } else {
                    format!("@{}: {};", v.name, v.value_text)
                },
            })
        } else if let Some(name) = word.strip_prefix('.') {
            let m = symbols
                .mixins
                .iter()
                .find(|m| m.prefix == '.' && m.name == name)?;
            LanguageString(lsp_types::LanguageString {
                language: "less".to_string(),
                value: format!("{}{}{} {{ ... }}", m.prefix, m.name, m.params_text),
            })
        } else if let Some(name) = word.strip_prefix('#') {
            let m = symbols
                .mixins
                .iter()
                .find(|m| m.prefix == '#' && m.name == name)?;
            LanguageString(lsp_types::LanguageString {
                language: "less".to_string(),
                value: format!("#{} {{ ... }}", m.name),
            })
        } else if let Some((_, doc)) = BUILTIN_FUNCTIONS.iter().find(|(n, _)| *n == word) {
            MarkedString::String((*doc).to_string())
        } else {
            return None;
        };
        Some(Hover {
            contents: HoverContents::Scalar(content),
            range: None,
        })
    }

    /// 文档符号（层级化）
    fn document_symbols(&self, uri: &lsp_types::Url) -> Option<DocumentSymbolResponse> {
        let (text, _) = self.docs.get(uri)?;
        let parsed = Parser::new(Lexer::new(text.clone()))
            .and_then(|mut p| p.parse())
            .ok()?;
        let symbols = statements_to_symbols(&parsed.statements, text);
        Some(DocumentSymbolResponse::Nested(symbols))
    }

    /// 定义跳转：当前文档，必要时解析相对 @import 文件
    fn definition(&self, uri: &lsp_types::Url, pos: Position) -> Vec<Location> {
        let Some((text, _)) = self.docs.get(uri) else {
            return Vec::new();
        };
        let Some(word) = word_at(text, pos.line, pos.character) else {
            return Vec::new();
        };
        let symbols = symbols_for_text(text);

        // 当前文档内的定义
        let mut found = find_declaration(&word, &symbols, uri, text);

        // 未找到时尝试解析简单的相对 @import
        if found.is_empty() {
            if let Ok(stylesheet) =
                Parser::new(Lexer::new(text.clone())).and_then(|mut p| p.parse())
            {
                let imports = collect_imports(&stylesheet.statements);
                let base = uri
                    .to_file_path()
                    .ok()
                    .and_then(|p| p.parent().map(|d| d.to_path_buf()));
                for path in imports {
                    let Some(base) = &base else { break };
                    let candidate = base.join(&path);
                    if !candidate.is_file() {
                        continue;
                    }
                    let Ok(imported) = std::fs::read_to_string(&candidate) else {
                        continue;
                    };
                    let Ok(target) = lsp_types::Url::from_file_path(&candidate) else {
                        continue;
                    };
                    let imported_symbols = symbols_for_text(&imported);
                    found = find_declaration(&word, &imported_symbols, &target, &imported);
                    if !found.is_empty() {
                        break;
                    }
                }
            }
        }
        found
    }
}

/// 根据错误信息构造 LSP 诊断
fn diagnostic_from_error(e: &crate::Error, text: &str) -> Diagnostic {
    let range = match (e.line(), e.column()) {
        (Some(line), Some(col)) => {
            let start = lsp_pos(line, col);
            let end = Position::new(start.line, start.character + 1);
            Range::new(start, end)
        }
        (Some(line), None) => {
            let l = (line as u32).saturating_sub(1);
            let len = line_text(text, l).map(|s| s.chars().count()).unwrap_or(0) as u32;
            Range::new(Position::new(l, 0), Position::new(l, len))
        }
        _ => {
            let len = line_text(text, 0).map(|s| s.chars().count()).unwrap_or(0) as u32;
            Range::new(Position::new(0, 0), Position::new(0, len))
        }
    };
    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        message: e.message(),
        ..Diagnostic::default()
    }
}

/// 在符号集合中查找单词对应的定义位置
fn find_declaration(
    word: &str,
    symbols: &DocumentSymbols,
    uri: &lsp_types::Url,
    text: &str,
) -> Vec<Location> {
    let location = |line: usize, column: usize, name_len: u32| -> Location {
        let start = lsp_pos(line, column);
        Location {
            uri: uri.clone(),
            range: Range::new(start, Position::new(start.line, start.character + name_len)),
        }
    };
    if let Some(name) = word.strip_prefix('@') {
        if let Some(v) = symbols.vars.iter().find(|v| v.name == name) {
            return vec![location(
                v.line,
                v.column,
                (name.chars().count() + 1) as u32,
            )];
        }
    } else if let Some(name) = word.strip_prefix('.') {
        if let Some(m) = symbols
            .mixins
            .iter()
            .find(|m| m.prefix == '.' && m.name == name)
        {
            return vec![location(
                m.line,
                m.column,
                (name.chars().count() + 1) as u32,
            )];
        }
    } else if let Some(name) = word.strip_prefix('#') {
        if let Some(m) = symbols
            .mixins
            .iter()
            .find(|m| m.prefix == '#' && m.name == name)
        {
            return vec![location(
                m.line,
                m.column,
                (name.chars().count() + 1) as u32,
            )];
        }
    }
    let _ = text;
    Vec::new()
}

/// 收集语句中的 LESS 相对导入路径
fn collect_imports(stmts: &[Statement]) -> Vec<String> {
    let mut out = Vec::new();
    for stmt in stmts {
        match stmt {
            Statement::Import(i) if matches!(i.import_type, ImportType::Less) => {
                out.push(i.path.clone());
            }
            Statement::Rule(r) => out.extend(collect_imports(&r.nested_rules)),
            Statement::MixinDefinition(m) => out.extend(collect_imports(&m.body)),
            Statement::AtRule(a) => {
                if let Some(block) = &a.block {
                    out.extend(collect_imports(block));
                }
            }
            Statement::EachCall(e) => out.extend(collect_imports(&e.body)),
            _ => {}
        }
    }
    out
}

/// 将语句递归转换为层级化文档符号
fn statements_to_symbols(stmts: &[Statement], text: &str) -> Vec<DocumentSymbol> {
    let mut out = Vec::new();
    for stmt in stmts {
        match stmt {
            Statement::Variable(v) => out.push(line_symbol(
                &format!("@{}", v.name),
                SymbolKind::VARIABLE,
                v.position.line,
                text,
            )),
            Statement::MixinDefinition(m) => out.push(line_symbol(
                &format!(".{}{}", m.name, params_to_string(&m.parameters)),
                SymbolKind::FUNCTION,
                m.position.line,
                text,
            )),
            Statement::Rule(r) => {
                let name = r
                    .selectors
                    .iter()
                    .map(selector_to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                let kind = if namespace_name(&r.selectors).is_some() {
                    SymbolKind::NAMESPACE
                } else {
                    SymbolKind::CLASS
                };
                let mut sym = line_symbol(&name, kind, r.position.line, text);
                sym.children = Some(statements_to_symbols(&r.nested_rules, text));
                out.push(sym);
            }
            Statement::AtRule(a) => {
                let name = match &a.prelude {
                    Some(p) => format!("@{} {}", a.name, p),
                    None => format!("@{}", a.name),
                };
                let mut sym = line_symbol(&name, SymbolKind::MODULE, a.position.line, text);
                if let Some(block) = &a.block {
                    sym.children = Some(statements_to_symbols(block, text));
                }
                out.push(sym);
            }
            _ => {}
        }
    }
    out
}

/// 构造覆盖某一行的文档符号（起始列对齐声明位置）
fn line_symbol(name: &str, kind: SymbolKind, line: usize, text: &str) -> DocumentSymbol {
    let line0 = (line as u32).saturating_sub(1);
    let len = line_text(text, line0)
        .map(|s| s.chars().count())
        .unwrap_or(name.chars().count()) as u32;
    let range = Range::new(Position::new(line0, 0), Position::new(line0, len));
    #[allow(deprecated)] // lsp-types 0.95 要求显式初始化该字段
    DocumentSymbol {
        name: name.to_string(),
        kind,
        range,
        selection_range: range,
        children: None,
        detail: None,
        tags: None,
        deprecated: None,
    }
}

/// 将选择器渲染为文本（用于符号名）
fn selector_to_string(sel: &Selector) -> String {
    let mut out = String::new();
    for part in &sel.parts {
        for simple in &part.simple_selectors {
            match simple {
                SimpleSelector::Universal(_) => out.push('*'),
                SimpleSelector::Type { name, .. } => out.push_str(name),
                SimpleSelector::Class { name, .. } => {
                    out.push('.');
                    out.push_str(name);
                }
                SimpleSelector::Id { name, .. } => {
                    out.push('#');
                    out.push_str(name);
                }
                SimpleSelector::Attribute { name, .. } => {
                    out.push_str(&format!("[{name}]"));
                }
                SimpleSelector::PseudoClass { name, argument, .. } => {
                    out.push_str(&format!(
                        ":{name}{}",
                        argument
                            .as_deref()
                            .map(|a| format!("({a})"))
                            .unwrap_or_default()
                    ));
                }
                SimpleSelector::PseudoElement { name, .. } => {
                    out.push_str(&format!("::{name}"));
                }
                SimpleSelector::Parent(_) => out.push('&'),
                SimpleSelector::Interpolation { variable, .. } => {
                    out.push_str(&format!("@{{{variable}}}"));
                }
            }
        }
        if let Some(c) = &part.combinator {
            out.push(' ');
            out.push_str(&c.to_string());
            out.push(' ');
        }
    }
    out
}
