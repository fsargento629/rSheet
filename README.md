# rSheet

A **WIP** ,fast, lightweight, terminal-based spreadsheet application written in **Rust**, featuring a real-time reactive calculation engine powered by a **Directed Acyclic Graph (DAG)** and **Ratatui** for rich terminal UI.

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
    Manages the interactive user session. It maintains the current mode (Normal vs. Edit), tracks cursor coordinates, and computes the auto-scrolling viewport boundaries to ensure the active cell remains visible during navigation.

  * src/domain/sheet.rs (Core Domain & Graph Engine):
    Encapsulates all business logic and data persistence. This module manages the 2D grid matrix, tracks adjacency lists for cellular dependencies, parses raw text into mathematical formulas, and executes the topological evaluation algorithms.

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
