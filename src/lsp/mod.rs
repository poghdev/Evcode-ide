use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum LspMessage {
    Diagnostic { file: String, message: String, line: u32 },
    Ready { server: String },
    Error(String),
}

pub struct LspState {
    pub available: bool,
    pub server_name: Option<String>,
    pub diagnostics: Vec<LspMessage>,
    pub rx: Option<mpsc::Receiver<LspMessage>>,
}

impl LspState {
    pub fn new() -> Self {
        let (available, name) = Self::detect_lsp_sync();
        Self {
            available,
            server_name: name,
            diagnostics: Vec::new(),
            rx: None,
        }
    }

    fn detect_lsp_sync() -> (bool, Option<String>) {
        let servers = [
            "pyright", "pylsp", "rust-analyzer",
            "clangd", "typescript-language-server", "gopls",
        ];
        for server in &servers {
            if Self::binary_exists(server) {
                return (true, Some(server.to_string()));
            }
        }
        (false, None)
    }

    fn binary_exists(name: &str) -> bool {
        std::process::Command::new("which")
            .arg(name)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            || std::process::Command::new("where")
                .arg(name)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
    }

    pub async fn start_async(&mut self) -> Option<mpsc::Sender<String>> {
        let server = self.server_name.clone()?;
        let (ui_tx, ui_rx) = mpsc::channel::<LspMessage>(64);
        let (req_tx, mut req_rx) = mpsc::channel::<String>(64);

        self.rx = Some(ui_rx);

        tokio::spawn(async move {
            if let Err(e) = lsp_worker(&server, ui_tx.clone(), &mut req_rx).await {
                let _ = ui_tx.send(LspMessage::Error(format!("LSP ошибка: {e}"))).await;
            }
        });

        Some(req_tx)
    }

    pub fn poll_messages(&mut self) {
        if let Some(rx) = &mut self.rx {
            while let Ok(msg) = rx.try_recv() {
                self.diagnostics.push(msg);
            }
        }
    }

    pub fn status_text(&self) -> String {
        match &self.server_name {
            Some(name) => format!("LSP: {name}"),
            None => "LSP: none".into(),
        }
    }

    pub fn diagnostic_strings(&self) -> Vec<String> {
        self.diagnostics
            .iter()
            .filter_map(|m| match m {
                LspMessage::Diagnostic { file, message, line } => {
                    Some(format!("{file}:{line} — {message}"))
                }
                LspMessage::Error(e) => Some(format!("[LSP ERR] {e}")),
                _ => None,
            })
            .collect()
    }
}

async fn lsp_worker(
    server: &str,
    tx: mpsc::Sender<LspMessage>,
    req_rx: &mut mpsc::Receiver<String>,
) -> Result<()> {
    let mut child: Child = Command::new(server)
        .arg("--stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("Failed to start LSP: {server}"))?;

    let stdin: ChildStdin = child.stdin.take().context("stdin unavailable")?;
    let stdout: ChildStdout = child.stdout.take().context("stdout unavailable")?;

    let init_req = json_rpc_request(1, "initialize", json!({
        "processId": std::process::id(),
        "rootUri": null,
        "capabilities": {}
    }));

    let mut stdin = tokio::io::BufWriter::new(stdin);
    send_rpc(&mut stdin, &init_req).await?;

    let _ = tx.send(LspMessage::Ready { server: server.to_string() }).await;

    let mut reader = BufReader::new(stdout).lines();

    loop {
        tokio::select! {
            line = reader.next_line() => {
                match line {
                    Ok(Some(raw)) => {
                        if raw.starts_with("Content-Length:") {
                            let _empty = reader.next_line().await;
                            if let Ok(Some(body)) = reader.next_line().await {
                                handle_lsp_response(&body, &tx).await;
                            }
                        }
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
            req = req_rx.recv() => {
                match req {
                    Some(msg) => { let _ = send_rpc(&mut stdin, &msg).await; }
                    None => break,
                }
            }
        }
    }

    Ok(())
}

async fn send_rpc(
    stdin: &mut tokio::io::BufWriter<ChildStdin>,
    body: &str,
) -> Result<()> {
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    stdin.write_all(header.as_bytes()).await?;
    stdin.write_all(body.as_bytes()).await?;
    stdin.flush().await?;
    Ok(())
}

fn json_rpc_request(id: u64, method: &str, params: Value) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params
    })
    .to_string()
}

async fn handle_lsp_response(body: &str, tx: &mpsc::Sender<LspMessage>) {
    let Ok(val): Result<Value, _> = serde_json::from_str(body) else { return };

    if val.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics") {
        if let Some(params) = val.get("params") {
            let uri = params
                .get("uri")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            if let Some(diags) = params.get("diagnostics").and_then(Value::as_array) {
                for d in diags {
                    let msg = d
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                        .to_string();
                    let line = d
                        .get("range")
                        .and_then(|r| r.get("start"))
                        .and_then(|s| s.get("line"))
                        .and_then(Value::as_u64)
                        .unwrap_or(0) as u32;
                    let _ = tx
                        .send(LspMessage::Diagnostic {
                            file: uri.clone(),
                            message: msg,
                            line,
                        })
                        .await;
                }
            }
        }
    }
}
