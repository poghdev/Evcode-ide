# 🌑 Evcode IDE

Evcode is a high—performance terminal development environment (IDE) created on Rust for those who value speed, minimalism and reliability. The project is focused on efficient work with code through the terminal using modern graphical capabilities of TUI.

## 🚀 Main features

* Ghost Snap: Capture the working status of the code in one click (Alt+S). All snapshots are stored in RAM for maximum speed and duplicated to disk in .evcode/snaps/ for reliability.
* Ghost Mode & Diff: View previous versions of files directly on top of the current code (Alt+D) using the built-in string comparison algorithm.
* Ultra-Fast Performance: Built on Ratatui and Crossterm, which provides instant response and minimal resource consumption (20-30 MB RAM).
* Clipboard integration: Full support for the system clipboard (Ctrl+C, Ctrl+V, Ctrl+X) and visual text selection.
* Built-in functionality: Asynchronous LSP client, file tree, terminal and session manager.

## 🛠 Technical stack

* Language: Rust (Safe, Fast, Concurrent)
* Interface: Ratatui (TUI)
* Editor core: tui-textarea
* System buffer: arboard
* Architecture: Modular (Editor, LSP, Terminal, Session, Snap Manager)

## ⌨️ Keyboard shortcuts

* Alt + S — Create an instant snapshot
* Alt + D — Enable comparison mode (Ghost Diff)
* Alt + R — Roll back to the last snapshot (Rollback)
* Ctrl + C/ V — Copy/Paste (System Clipboard)
* Shift + Arrows — Text selection
* Ctrl + Q — Exit the IDE

## 📦 Installation and assembly

1. Clone the repository:
   git clone https://github.com/poghdev/Evcode-ide
2. Go to the project folder:
   cd Evcode-ide
3. Assemble the project:
   cargo build --release
