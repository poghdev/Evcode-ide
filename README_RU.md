<p align="center">
  <img src="assets/evcode-ascii.jpg" alt="Evcode Logo" width="1200">
</p>

<h1 align="center">Evcode</h1>
<p align="center">
  <b>Ghost-powered терминальная IDE на Rust</b><br>
  Быстрая | Лёгкая | Минималистичная | Для разработчиков
</p>

---

# Обзор

**Evcode** — современная терминальная IDE, написанная на **Rust** для разработчиков, которым важны скорость, концентрация и точность.

Evcode объединяет отзывчивость терминальных редакторов с IDE‑возможностями:

- ⚡ Быстрое редактирование
- 🧠 LSP интеграция
- 👻 Ghost snapshots и rollback
- 📁 Дерево файлов
- 🔍 Teleport fuzzy search
- 🖥 Встроенный терминал
- 🎨 Темы
- 📋 Нативная работа с clipboard

Evcode создаётся вокруг минималистичного workflow и остаётся лёгкой, сохраняя при этом инструменты для реальной разработки.

---

# Почему Evcode?

Большинство IDE становятся тяжёлыми и перегруженными.

Evcode следует другой философии:

> Оставайся в терминале.  
> Держи редактор быстрым.  
> Сохраняй концентрацию.

Благодаря Rust и Ratatui Evcode обеспечивает мгновенную реакцию и чистый developer experience.

### Производительность

- ~7-10 MB RAM (idle)
- Лёгкий TUI rendering
- Быстрый запуск
- Эффективная работа с текстовым буфером
- Минимальная нагрузка на систему

---

# Ghost System

Evcode использует собственный workflow под названием **Ghost**.

Вместо временных Git‑коммитов во время экспериментов Ghost позволяет мгновенно сохранять, сравнивать и восстанавливать состояние кода.

## Ghost Snap — `Alt + S`

Создаёт лёгкий snapshot текущего буфера.

Это можно представить как save state для кода.

Ghost snapshots:

- Создаются мгновенно
- Хранятся в RAM
- Дополнительно сохраняются в:

```bash
~/.local/share/evcode/snaps/
```

для безопасности и persistence.

---

## Ghost Diff — `Alt + D`

Сравнение текущего кода с Ghost snapshot.

Ghost Diff показывает:

- Добавленные строки
- Удалённый код
- Изменённые участки

без выхода из редактора.

Подходит для:

- Refactoring
- Экспериментов
- Проверки логики
- Быстрого сравнения идей

---

## Ghost Rollback — `Alt + R`

Мгновенно возвращает код к последнему Ghost snapshot.

Без:

- Git reset
- Цепочек Undo
- Потери времени

---

# Возможности

## Редактирование

- Multi-tab editing
- Syntax highlighting
- Selection support
- Undo / Redo
- Clipboard integration
- Быстрая навигация курсором

---

## Language Intelligence

Встроенный **LSP** предоставляет:

- Diagnostics
- Error highlighting
- Language awareness
- Real-time feedback

Поддерживаются языки с доступными LSP:

- Rust
- Python
- JavaScript / TypeScript
- C / C++
- И другие

---

## Teleport Search

### `Ctrl + P`

Teleport — быстрый fuzzy finder для файлов.

Позволяет мгновенно перемещаться между файлами проекта без ручного поиска.

Подходит для:

- Больших проектов
- Быстрой навигации
- Keyboard-first workflow

---

## Встроенный терминал

Запускай команды прямо внутри Evcode.

Компиляция, тестирование и запуск без выхода из IDE.

Подходит для:

- Cargo
- Python
- Node
- Build systems
- Shell commands

---

## File Tree

Просмотр структуры проекта прямо внутри редактора.

Полезно для:

- Открытия файлов
- Навигации по проекту
- Обзора директорий

---

# Установка

## Сборка из исходников

```bash
git clone <repo-url>
cd evcode
cargo build --release
```

Запуск:

```bash
./target/release/evcode
```

---

## Cargo

```bash
cargo install evcode
```

---

# Горячие клавиши

## Файлы и редактор

| Shortcut | Action |
|---|---|
| Ctrl + S | Сохранить файл |
| Ctrl + W | Закрыть вкладку |
| Ctrl + Q | Выйти из Evcode |
| Ctrl + Z | Undo |
| Ctrl + Y | Redo |
| Ctrl + A | Выделить всё |
| Ctrl + C | Copy |
| Ctrl + X | Cut |
| Ctrl + V | Paste |

---

## Навигация

| Shortcut | Action |
|---|---|
| Ctrl + P | Teleport search |
| Ctrl + B | File tree |
| Ctrl + E | Следующая вкладка |
| Ctrl + Shift + A | Предыдущая вкладка |
| Shift + Arrows | Выделение текста |

---

## Ghost System

| Shortcut | Action |
|---|---|
| Alt + S | Ghost Snap |
| Alt + D | Ghost Diff |
| Alt + R | Ghost Rollback |

---

## Интерфейс

| Shortcut | Action |
|---|---|
| Ctrl + T | Смена темы |
| Ctrl + ` | Терминал |

---

# Цели проекта

Evcode строится вокруг нескольких идей:

- Terminal-first development
- Высокая производительность
- Чистый интерфейс
- Keyboard-driven workflow
- Минимальное трение
- Быстрые эксперименты

Цель Evcode — не стать перегруженным редактором.

Цель — оставаться быстрым.

---

# Tech Stack

- Rust
- Ratatui
- Rope-based editing
- LSP architecture
- Terminal UI rendering

---

# Примеры

## Домашний экран

<p align="center">
  <img src="assets/examples/home.png" alt="Главный экран Evcode" width="1000">
</p>

## Рабочее пространство редактора

<p align="center">
  <img src="assets/examples/one.png" alt="Интерфейс редактора" width="1000">
</p>

## Дерево файлов

<p align="center">
  <img src="assets/examples/two.png" alt="Дерево файлов" width="1000">
</p>

## Навигация

<p align="center">
  <img src="assets/examples/three.png" alt="Навигация по проекту" width="1000">
</p>

## Снимок (Snapshot)

<p align="center">
  <img src="assets/examples/four.png" alt="Ghost Snapshot" width="1000">
</p>

## Терминал

<p align="center">
  <img src="assets/examples/five.png" alt="Встроенный терминал" width="1000">
</p>

<p align="center">
  <i>Evcode в работе внутри терминала</i>
</p>

# License

MIT License

---

<p align="center">
Made with Rust by <b>Eeive</b>
</p>
