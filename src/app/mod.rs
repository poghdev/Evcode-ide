mod action;
mod keymaps;
mod lsp_helpers;
mod mouse;
mod state;

pub use state::{Focus, NamingType, Theme, TeleportState};

use crate::editor::EditorState;
use crate::filesystem::FileTree;
use crate::lsp::LspState;
use crate::session::Session;
use crate::snap::{DiffLine, GhostSnapManager};
use crate::terminal::TerminalState;
use crate::ui::draw;
use anyhow::Result;
use crossterm::event::{self, Event};
use ratatui::{backend::Backend, Terminal};
use std::collections::HashSet;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub struct App {
    pub focus: Focus,
    pub theme: Theme,
    pub editor: EditorState,
    pub file_tree: FileTree,
    pub teleport: TeleportState,
    pub term: TerminalState,
    pub lsp: LspState,
    pub lsp_tx: Option<mpsc::Sender<String>>,
    pub snap: GhostSnapManager,
    pub snap_diff: Option<Vec<DiffLine>>,
    pub show_snap_diff: bool,
    pub naming_input: String,
    pub naming_type: NamingType,
    pub naming_target: String,
    pub problems: HashSet<String>,
    pub show_file_tree: bool,
    pub show_teleport: bool,
    pub show_terminal: bool,
    pub show_hidden: bool,
    pub ghost_mode: bool,
    pub should_quit: bool,
    pub status_msg: String,
    pub status_msg_time: Option<Instant>,
    pub(crate) clipboard: Option<arboard::Clipboard>,
}

impl App {
    pub async fn new() -> Self {
        let mut app = Self {
            focus: Focus::Editor,
            theme: Theme::Dark,
            editor: EditorState::new(),
            file_tree: FileTree::new("."),
            teleport: TeleportState::new(),
            term: TerminalState::new(),
            lsp: LspState::new(),
            lsp_tx: None,
            snap: GhostSnapManager::new(),
            snap_diff: None,
            show_snap_diff: false,
            naming_input: String::new(),
            naming_type: NamingType::CreateFile,
            naming_target: String::new(),
            problems: HashSet::new(),
            show_file_tree: false,
            show_teleport: false,
            show_terminal: false,
            show_hidden: false,
            ghost_mode: false,
            should_quit: false,
            status_msg: String::from(
                "Evcode — Ctrl+C copy | Ctrl+V paste | Ctrl+X cut | Ctrl+A select all | Ctrl+Q quit",
            ),
            status_msg_time: None,
            clipboard: None,
        };

        app.lsp_tx = app.lsp.start_async().await;

        if let Ok(session) = Session::load() {
            app.restore_session(session);
        }

        app
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_msg = msg.into();
        self.status_msg_time = Some(Instant::now());
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        const STATUS_TIMEOUT: Duration = Duration::from_secs(4);
        const FRAME_BUDGET: Duration = Duration::from_millis(16);

        loop {
            if let Some(t) = self.status_msg_time {
                if t.elapsed() >= STATUS_TIMEOUT {
                    self.status_msg.clear();
                    self.status_msg_time = None;
                }
            }
            
            self.term.poll_completed();
            self.lsp.poll_messages();
            for d in self.lsp.diagnostic_strings() {
                self.problems.insert(d);
            }

            terminal.draw(|f| draw(f, self))?;

            if event::poll(FRAME_BUDGET)? {
                match event::read()? {
                    Event::Key(key) => self.handle_key(key).await,
                    Event::Mouse(mouse) => self.handle_mouse(mouse),
                    _ => {}
                }
            }

            if self.should_quit {
                let _ = self.editor.save_all();
                self.snap.save_all_to_disk();
                self.save_session();
                break;
            }
        }

        Ok(())
    }
}