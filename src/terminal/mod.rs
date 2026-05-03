use std::process::Command;

pub struct TerminalState {
    pub input: String,
    pub history: Vec<TerminalEntry>,
    #[allow(dead_code)]
    pub scroll: usize,
}

pub struct TerminalEntry {
    pub command: String,
    pub output: String,
    pub is_error: bool,
}

impl TerminalState {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            history: Vec::new(),
            scroll: 0,
        }
    }

    pub fn execute(&mut self, cmd: &str) -> String {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", cmd]).output()
        } else {
            Command::new("sh").args(["-c", cmd]).output()
        };

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let is_error = !out.status.success();
                let result = if is_error && !stderr.is_empty() {
                    stderr.clone()
                } else {
                    stdout.clone()
                };
                self.history.push(TerminalEntry {
                    command: cmd.to_string(),
                    output: result.clone(),
                    is_error,
                });
                result
            }
            Err(e) => {
                let msg = format!("Execution error: {e}");
                self.history.push(TerminalEntry {
                    command: cmd.to_string(),
                    output: msg.clone(),
                    is_error: true,
                });
                msg
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
