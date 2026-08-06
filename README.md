# rSheet

A **WIP**, fast, lightweight, terminal-based spreadsheet application written in **Rust**, featuring a real-time reactive calculation engine powered by a **Directed Acyclic Graph (DAG)** and **Ratatui** for rich terminal UI.

## Features

- **Reactive Formula Engine**: Formulas automatically recalculate when dependencies change
- **Built-in Functions**: POW, SUM, AVG, MAX, MIN, COUNT, SQRT, MEDIAN
- **Cell Ranges & Multi-Ranges**: Support for single references (`A1`), cell ranges (`A1:A10`), and multiple ranges (`A1:A10, B1:B10`)
- **DAG-based Dependency Tracking**: Efficient change propagation using topological sorting
- **Circular Dependency Detection**: Prevents infinite loops with cycle detection
- **Terminal UI**: Vi-style keyboard navigation with visual cursor feedback
- **CSV Import/Export**: Load and save spreadsheets in CSV format

## Quick Start

```bash
# Run with default test file
cargo run

# Open a specific CSV file
cargo run path/to/file.csv
```

### Modal Editing — Three Modes

rSheet uses a three-mode editing model inspired by vi. The current mode is always visible in the status bar (bottom of the screen), which also changes colour for instant feedback.

| Mode | Status bar colour | Description |
|------|-------------------|-------------|
| **NORMAL** | Dark grey | Default navigation mode |
| **VISUAL** | Magenta | Focused navigation, ready to insert |
| **INSERT** | Green | Active cell editing (modal overlay) |

#### Normal mode

##### Navigation

| Key | Action |
|-----|--------|
| `h` / `j` / `k` / `l` or Arrow keys | Move cursor left / down / up / right (count supported, e.g. `3j`) |
| `w` | Move right one cell (alias for `l`) |
| `b` | Move left one cell (alias for `h`) |
| `0` or `Home` | Jump to column A of current row |
| `$` | Jump to last column of current row |
| `Shift+Home` | Jump to row 1 of current column |
| `Ctrl+Home` | Jump to cell A1 |

##### Mode transitions

| Key | Action |
|-----|--------|
| `a` or `F2` | Enter **Visual** mode |
| `i` | Enter **Insert** mode with blank buffer |
| `=` | Enter **Insert** mode seeded with `=` (formula shortcut) |
| `Enter` | Enter **Insert** mode with current cell content |

##### Cell / file

| Key | Action |
|-----|--------|
| `Delete` / `Backspace` | Clear current cell |
| `s` / `S` | Save spreadsheet |
| `q` / `Q` | Quit |
| `Esc` | Cancel pending command |

##### Row-axis delete (`d`)

Operates on entire rows for vertical motions, and on partial rows for horizontal motions. Counts are supported (e.g. `3dh`, `2dj`).

| Key sequence | Action |
|--------------|--------|
| `dh` | Clear cells to the left of the cursor in the current row |
| `dl` | Clear cells to the right of the cursor in the current row |
| `dj` | Clear the current row and rows below (per count) |
| `dk` | Clear the current row and rows above (per count) |
| `d$` | Clear from cursor to end of row |
| `d0` | Clear from start of row to the cell before the cursor |
| `dd` | Clear entire current row; `Ndd` clears N rows |

##### Column-axis delete (`gd`)

Mirror of `d` operating on the column axis. Counts are supported (e.g. `3gdh`, `2gdj`).

| Key sequence | Action |
|--------------|--------|
| `gdj` | Clear cells below the cursor in the current column |
| `gdk` | Clear cells above the cursor in the current column |
| `gdl` | Clear entire columns to the right of the cursor |
| `gdh` | Clear entire columns to the left of the cursor |
| `gd$` | Clear current column from cursor row to the bottom |
| `gd0` | Clear current column from the top to the cursor row |
| `gdd` | Clear entire current column; `Ngdd` clears N columns |

##### Yank (`y`)

| Key sequence | Action |
|--------------|--------|
| `yh` | Yank cells to the left of the cursor |
| `yl` | Yank cells to the right of the cursor |
| `yj` | Yank current cell + N cells below in the same column (vertical strip) |
| `yk` | Yank current cell + N cells above in the same column (vertical strip) |
| `y$` | Yank from cursor to end of row |
| `y0` | Yank from start of row to cursor |
| `yy` | Yank entire current row (remembers cursor column for offset paste); `Nyy` yanks N rows |

##### Paste

| Key | Cells / Rows clipboard | Column clipboard |
|-----|------------------------|------------------|
| `p` | Insert after cursor / below current row — shifts existing content | Overwrite current column downward from cursor+1 |
| `P` | Insert before cursor / above current row — shifts existing content | Overwrite current column downward from cursor |
| `gp` | Overwrite starting at cursor+1 (cells) or row+1 (rows) — no shifting; row paste uses yank column offset | Identical to `p` for column clips |
| `gP` | Overwrite starting at cursor / current row — no shifting; row paste uses yank column offset | Identical to `P` for column clips |

> **Column clipboard** is produced by `yj`, `yk`, `gdj`, `gdk`, `gd$`, and `gd0`. For column clips, `gp`/`gP` behave identically to `p`/`P` respectively.

##### State machine grammar

Normal mode uses a multi-state parser. The general grammar is `[count1] operator [count2] motion`. There are four states: **Idle**, **OperatorPending** (entered after `d` or `y`), **GPrefix** (entered after `g`), and **GdOperatorPending** (entered after `gd`). The status bar displays pending keys while a command is being built. `Esc` cancels at any point.

See [NORMAL_MODE.md](NORMAL_MODE.md) for a complete command reference with detailed examples.

#### Visual mode

Arrow keys navigate the grid; `hjkl` are **not** active here.

| Key | Action |
|-----|--------|
| Arrow keys | Navigate cells |
| `Enter` | Navigate down one row |
| Any printable character | Enter **Insert** mode seeded with that character |
| `Delete` / `Backspace` | Clear current cell |
| `Esc` | Return to **Normal** mode |

#### Insert mode

A modal overlay shows the cell reference and the edit buffer.

| Key | Action |
|-----|--------|
| Any character / `Backspace` / `Delete` | Edit the buffer |
| `Left` / `Right` / `Home` / `End` | Move cursor within buffer |
| `Enter` or `F2` | Commit buffer and return to **Visual** mode |
| `Esc` | Discard buffer and return to **Visual** mode |

### Function Examples

```
=sum(A1:A10)                 // Sum of a single range
=sum(A1:A10, B1:B10)         // Sum of multiple ranges
=avg(A1:C5)                  // Average of a 2D range
=count(A1:A10)               // Count numeric values
=sqrt(16)                    // Square root: returns 4
=median(A1:A5)               // Median of range values
=sum(pow(2,2), sqtr(16))     // Nested function calls
```

See [FUNCTIONS.md](FUNCTIONS.md) for complete function documentation.

## rSheet System Architecture & Technical Summary

### 1. High-Level Architecture

`rSheet` is designed using a strict separation of concerns, isolating the core computational engine from the terminal presentation layer. The architecture follows a layered state-management pattern, ensuring that UI rendering is entirely stateless and driven exclusively by the underlying domain data.

The system relies on a **Directed Acyclic Graph (DAG)** to form a reactive computation engine. Instead of re-evaluating the entire spreadsheet upon a modification, the engine computes precise structural dependencies and isolates updates strictly to the affected downstream cells.

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                           Terminal Application                          │
│                                                                         │
│  ┌──────────────────────┐                       ┌────────────────────┐  │
│  │    Event Loop &      │ ── Keyboard Input ──► │  Application State │  │
│  │  Lifecycle Manager   │                       │     (app.rs)       │  │
│  │      (main.rs)       │ ◄── Window Bounds ─── │ (Modes, Viewport,  │  │
│  └──────────┬───────────┘                       │  Cursor Tracking)  │  │
│             │                                   └─────────┬──────────┘  │
│      Terminal Setup                                       │             │
│    (Raw Mode, Screen)                               Mutates State       │
│             │                                             │             │
│             ▼                                             ▼             │
│  ┌──────────────────────┐                       ┌────────────────────┐  │
│  │ Presentation Layer   │ ◄── Projects View ─── │    Core Domain     │  │
│  │    (ui/mod.rs)       │                       │ (domain/sheet.rs)  │  │
│  │  (Ratatui Widgets,   │                       │ (DAG Engine, CSV,  │  │
│  │  Grid Rendering)     │ ◄──── Reads AST ───── │ Cell Evaluator)    │  │
│  └──────────────────────┘                       └────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2. Component Breakdown

  * src/main.rs (Application Entry Point & Lifecycle):
    Orchestrates the terminal lifecycle. It initializes crossterm for raw terminal output, processes command-line arguments to establish the target CSV file path, and executes the primary synchronous event loop at terminal frame rates.

  * src/app.rs (State & Interaction Layer):
    Manages the interactive user session. It maintains the current mode (Normal, Visual, or Insert), tracks cursor coordinates, and computes the auto-scrolling viewport boundaries to ensure the active cell remains visible during navigation. The three-mode design separates pure navigation (Normal), focused navigation ready for input (Visual), and active cell editing via a modal overlay (Insert).

  * src/domain/sheet.rs (Core Domain & Graph Engine):
    Encapsulates grid management and data persistence. This module manages the 2D grid matrix, tracks adjacency lists for cellular dependencies, parses raw text into mathematical formulas, and executes topological evaluation algorithms.

  * src/domain/functions.rs (Formula Functions):
    Implements all built-in spreadsheet functions (POW, SUM, AVG, MAX, MIN, COUNT, SQRT, MEDIAN) in a dedicated, modular file.

  * src/ui/ (Presentation Layer):
    Houses the custom ratatui widgets, primarily grid_widget.rs. It translates the viewport coordinates provided by app.rs into formatted terminal blocks, rendering row headers, column headers, cell values, and conditional formatting (such as error states and cursor highlights).

### 3. Core Algorithms & Features

  * Cycle Detection (Directed Graph Validation):
    Prior to committing a formula to the grid, the engine executes a Depth-First Search (DFS) against the dependency graph. This validates that the new input does not introduce a circular dependency (e.g., A1 relying on B1 while B1 relies on A1). If a cycle is detected, the operation is intercepted and the cell safely yields a #CIRCULAR! error, preventing recursive stack overflows.

  * Topological Sorting (Reactive Re-evaluation):
    When a cell is modified, the application isolates the downstream graph of affected cells. It applies Kahn's Algorithm to determine the precise topological ordering of these dependencies. This guarantees that every cell is recalculated only after its prerequisite cells have been updated, optimizing CPU cycles.

  * AST-Free Parsing (Performance Optimization):
    To minimize heap allocations, the mathematical evaluator avoids constructing a full Abstract Syntax Tree (AST). Instead, it utilizes a custom tokenizer combined with a recursive descent parser to evaluate expressions (e.g., =A1 + (B2 * 3)) on the fly, strictly adhering to standard operational precedence.

### 4. Primary Dependencies (Crates)

The architecture leverages two primary third-party libraries in the Rust ecosystem to handle cross-platform terminal UI constraints:

  **ratatui:**
  A Rust library for building rich terminal user interfaces. In rSheet, it is utilized to define the immediate-mode rendering layout. It provides the core structures for drawing the application's borders, text constraints, and layout calculations (via the Rect structures).

  **crossterm:**
  A cross-platform terminal manipulation library. It is responsible for low-level console interactions, including enabling "raw mode" (which prevents standard terminal line buffering), managing the alternate screen buffer, and capturing synchronous keyboard events (KeyCode events like arrow keys, enter, and escape).
