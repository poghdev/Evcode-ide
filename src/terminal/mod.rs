use tokio::process::Command;
use tokio::sync::mpsc;

pub struct TerminalState {
    pub input: String,
    pub history: Vec<TerminalEntry>,
    #[allow(dead_code)]
    pub scroll: usize,
    tx: mpsc::UnboundedSender<TerminalEntry>,
    rx: mpsc::UnboundedReceiver<TerminalEntry>,
}
pub struct TerminalEntry {
    pub command: String,
    pub output: String,
    pub is_error: bool,
}

impl TerminalState {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            input: String::new(),
            history: Vec::new(),
            scroll: 0,
            tx,
            rx,
        }
    }

    pub fn execute(&mut self, cmd: &str) -> String {
        let cmd = cmd.to_string();
        let tx = self.tx.clone();
        self.history.push(TerminalEntry {
            command: cmd.clone(),
            output: "Running...".into(),
            is_error: false,
        });

        tokio::spawn(async move {
            let output = if cfg!(target_os = "windows") {
                Command::new("cmd").args(["/C", &cmd]).output().await
            } else {
                Command::new("sh").args(["-c", &cmd]).output().await
            };

            let entry = match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    let is_error = !out.status.success();
                    let result = if is_error && !stderr.is_empty() {
                        stderr
                    } else {
                        stdout
                    };
                    TerminalEntry {
                        command: cmd,
                        output: result,
                        is_error,
                    }
                }
                Err(e) => TerminalEntry {
                    command: cmd,
                    output: format!("Execution error: {e}"),
                    is_error: true,
                },
            };

            let _ = tx.send(entry);
        });

        "Running...".into()
    }

    pub fn poll_completed(&mut self) {
        while let Ok(entry) = self.rx.try_recv() {
            if let Some(existing) = self
                .history
                .iter_mut()
                .rev()
                .find(|item| item.command == entry.command && item.output == "Running...")
            {
                *existing = entry;
            } else {
                self.history.push(entry);
            }
        }
    }

    #[allow(dead_code)]
    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    #[allow(dead_code)]
    pub fn scroll_down(&mut self) {
        if self.scroll + 1 < self.history.len() {
            self.scroll += 1;
        }
    }
}
