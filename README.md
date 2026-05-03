# 🌑 Evcode IDE (English)

Evcode is a high-performance terminal development environment (IDE) built with Rust for those who value speed, minimalism, and reliability. The project focuses on efficient workflow through the terminal using modern TUI capabilities.

## 🚀 Main Features

* **Ghost Snap**: Capture the working status of the code in one click (**Alt+S**). Snapshots are stored in RAM for speed and duplicated to `.evcode/snaps/` for reliability.
* **Ghost Mode & Diff**: View previous versions of files directly on top of the current code (**Alt+D**) using a built-in string comparison algorithm.
* **Ultra-Fast Performance**: Built on **Ratatui** and **Crossterm**, providing instant response and minimal resource consumption (20-30 MB RAM).
* **Clipboard Integration**: Full support for system clipboard (**Ctrl+C, Ctrl+V, Ctrl+X**) and visual text selection.
* **Built-in Functionality**: Asynchronous LSP client, file tree, terminal, and session manager.

## 📦 Download & Installation

### Option 1: Via Cargo (Recommended)
cargo install evcode

### Option 2: Pre-built Binaries
Download the executable for your OS from the Releases page:
* **Linux**: evcode-linux-x86_64
* **Windows**: evcode-windows-x64.exe
* **macOS**: evcode-macos-x86_64

## ⌨️ Keyboard Shortcuts

| Category | Action | Shortcut |
| :--- | :--- | :--- |
| **Ghost Snap** | **Capture Instant Snapshot** | `Alt + S` |
| | **Toggle Ghost Diff Mode** | `Alt + D` |
| | **Restore Last Snapshot** | `Alt + R` |
| **Editing** | **Select All Code** | `Ctrl + A` |
| | **Copy to System Clipboard** | `Ctrl + C` |
| | **Paste from System Clipboard** | `Ctrl + V` |
| | **Cut to System Clipboard** | `Ctrl + X` |
| | **Undo / Redo** | `Ctrl + Z / Ctrl + Y` |
| | **Select Text** | `Shift + Arrows` |
| **Navigation** | **Switch Tabs / Buffers** | `Ctrl + E` |
| | **Move Cursor by Word** | `Ctrl + Left / Right` |
| | **Jump to Line Start / End** | `Home / End` |
| | **Toggle File Tree** | `Ctrl + B` |
| **System** | **Exit Evcode** | `Ctrl + Q` |
| | **Save File** | `Ctrl + S` |

---

# 🌑 Evcode IDE (Русский)

Evcode — это высокопроизводительная среда разработки для терминала (IDE), созданная на Rust для тех, кто ценит скорость, минимализм и надежность. Проект ориентирован на эффективную работу с кодом через терминал с использованием современных графических возможностей TUI.

## 🚀 Основные возможности

* **Ghost Snap**: Мгновенное сохранение состояния кода в один клик (**Alt+S**). Снимки хранятся в оперативной памяти для максимальной скорости и дублируются на диск в `.evcode/snaps/` для надежности.
* **Ghost Mode & Diff**: Просмотр предыдущих версий файлов прямо поверх текущего кода (**Alt+D**) с использованием встроенного алгоритма сравнения строк.
* **Сверхбыстрая производительность**: Работает на базе **Ratatui** и **Crossterm**, обеспечивая мгновенный отклик и минимальное потребление ресурсов (20-30 МБ ОЗУ).
* **Интеграция с буфером**: Полная поддержка системного буфера обмена (**Ctrl+C, Ctrl+V, Ctrl+X**) и визуальное выделение текста.
* **Встроенный функционал**: Асинхронный LSP-клиент, дерево файлов, терминал и менеджер сессий.

## 📦 Загрузка и установка

### Вариант 1: Через Cargo (Для разработчиков)
cargo install evcode

### Вариант 2: Готовые файлы (Прямая загрузка)
Скачайте исполняемый файл для вашей ОС со страницы Releases:
* **Linux**: evcode-linux-x86_64
* **Windows**: evcode-windows-x64.exe
* **macOS**: evcode-macos-x86_64

## ⌨️ Горячие клавиши

| Категория | Действие | Клавиши |
| :--- | :--- | :--- |
| **Ghost Snap** | **Сделать мгновенный снимок** | `Alt + S` |
| | **Режим сравнения (Ghost Diff)** | `Alt + D` |
| | **Откат к последнему снимку** | `Alt + R` |
| **Редактирование** | **Выделить весь код** | `Ctrl + A` |
| | **Копировать в буфер ОС** | `Ctrl + C` |
| | **Вставить из буфера ОС** | `Ctrl + V` |
| | **Вырезать в буфер ОС** | `Ctrl + X` |
| | **Отмена / Повтор** | `Ctrl + Z / Ctrl + Y` |
| | **Выделение текста** | `Shift + Стрелки` |
| **Навигация** | **Переключение вкладок** | `Ctrl + E` |
| | **Перемещение по словам** | `Ctrl + Left / Right` |
| | **В начало / Конец строки** | `Home / End` |
| | **Скрыть/Показать дерево файлов** | `Ctrl + B` |
| **Системные** | **Выход из IDE** | `Ctrl + Q` |
| | **Сохранить файл** | `Ctrl + S` |
