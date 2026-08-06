# rSheet Normal Mode Reference

## Table of Contents

1. [Overview](#1-overview)
2. [Navigation](#2-navigation)
3. [Mode Transitions](#3-mode-transitions)
4. [Cell and File Operations](#4-cell-and-file-operations)
5. [Row-Axis Delete — `d`](#5-row-axis-delete--d)
6. [Column-Axis Delete — `gd`](#6-column-axis-delete--gd)
7. [Yank — `y`](#7-yank--y)
8. [Paste — `p` and `P`](#8-paste--p-and-p)
9. [Overwrite Paste — `gp` and `gP`](#9-overwrite-paste--gp-and-gp)
10. [The `g`-Prefix](#10-the-g-prefix)
11. [Quick Reference](#11-quick-reference)

---

## 1. Overview

Normal mode is rSheet's default mode. Every keypress is interpreted as a **command** — nothing is typed into a cell. To edit a cell, you must first transition to Insert or Visual mode (see [§3](#3-mode-transitions)).

### Command Grammar

```
[count1] operator [count2] motion
```

- **count1** — optional multiplier before the operator
- **operator** — the action to perform (`d`, `y`, `g`, …)
- **count2** — optional multiplier before the motion
- **motion** — the direction or range (`h`, `j`, `k`, `l`, `w`, `b`, `0`, `$`)

When both counts are present they are **multiplied** together to produce the effective count:

```
2d3j   →  count1=2, operator=d, count2=3, motion=j
           effective count = 2×3 = 6
```

While a command is being assembled, the status bar shows the keys typed so far (e.g. `2d3`). Press `Esc` at any point to cancel and return to Idle.

### Internal States

```
Idle
 ├─ d / y          →  OperatorPending   ──── motion ──→  resolved (Idle)
 └─ g              →  GPrefix
     ├─ p / P      →  resolved (Idle)
     └─ d          →  GdOperatorPending ──── motion ──→  resolved (Idle)
         └─ d      →  resolved (Idle)   (gdd shorthand)
```

| State | Status bar | What it means |
|---|---|---|
| Idle | _(empty)_ | Waiting for a new command |
| OperatorPending | `d` or `y` | Operator received, waiting for a motion |
| GPrefix | `g` | `g` received, waiting for second key |
| GdOperatorPending | `gd` | Column-axis delete, waiting for a motion |

---

## 2. Navigation

### 2.1 Basic Movement — `h` `j` `k` `l` and Arrow Keys

Arrow keys and `hjkl` are identical in behaviour. All accept a count prefix. Movement stops at the grid boundary — it never wraps or errors.

---

**`h` / Left arrow — move one cell left**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52

After `h`:
     A     B     C     D     E
1   10    20    30    40    50
2   11   [21]   31    41    51
3   12    22    32    42    52
```

---

**`l` / Right arrow — move one cell right**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52

After `l`:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31   [41]   51
3   12    22    32    42    52
```

---

**`3l` — move 3 cells right**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11   [21]   31    41    51
3   12    22    32    42    52

After `3l`:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41   [51]
3   12    22    32    42    52
```

---

**`j` / Down arrow — move one cell down**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52

After `j`:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22   [32]   42    52
```

---

**`5j` — move 5 rows down**

```
Before:
     A     B     C     D     E
1   10   [20]   30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24    34    44    54
6   15    25    35    45    55
7   16    26    36    46    56

After `5j`:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24    34    44    54
6   15   [25]   35    45    55
7   16    26    36    46    56
```

---

**`k` / Up arrow — move one cell up**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52

After `k`:
     A     B     C     D     E
1   10    20   [30]   40    50
2   11    21    31    41    51
3   12    22    32    42    52
```

---

**`10k` — move 10 rows up (clamps to row 1)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22   [32]   42    52

After `10k`  (only 2 rows above; clamps at row 1):
     A     B     C     D     E
1   10    20   [30]   40    50
2   11    21    31    41    51
3   12    22    32    42    52
```

---

**`2h` with cursor already at column A — stays at A1**

```
Before:
     A     B     C     D     E
1  [10]   20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52

After `2h`  (already at left edge; no movement):
     A     B     C     D     E
1  [10]   20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
```

---

### 2.2 Row Jumps — `0` and `$`

Neither `0` nor `$` accept a count prefix.

---

**`0` — jump to column A of the current row**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41   [51]
3   12    22    32    42    52

After `0`:
     A     B     C     D     E
1   10    20    30    40    50
2  [11]   21    31    41    51
3   12    22    32    42    52
```

---

**`$` — jump to the last column of the current row**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2  [11]   21    31    41    51
3   12    22    32    42    52

After `$`:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41   [51]
3   12    22    32    42    52
```

---

### 2.3 Word Motion — `w` and `b`

In a spreadsheet context, there are no word boundaries between characters, so:

- `w` — move one cell to the **right** (equivalent to `l`)
- `b` — move one cell to the **left** (equivalent to `h`)

Both accept counts (`3w` = move 3 cells right) and work as motions for operators (`dw` = `dl`, `d3w` = `d3l`).

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52

After `2w`:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41   [51]
3   12    22    32    42    52
```

---

### 2.4 Home Keys

| Key | Behaviour |
|---|---|
| `Home` | Jump to column A of the current row (same as `0`) |
| `Shift+Home` | Jump to row 1 of the current column |
| `Ctrl+Home` | Jump to cell A1 |

---

**`Home` — jump to column A of current row**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31   [41]   51
3   12    22    32    42    52

After `Home`:
     A     B     C     D     E
1   10    20    30    40    50
2  [11]   21    31    41    51
3   12    22    32    42    52
```

---

**`Shift+Home` — jump to row 1 of the current column**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22   [32]   42    52

After `Shift+Home`:
     A     B     C     D     E
1   10    20   [30]   40    50
2   11    21    31    41    51
3   12    22    32    42    52
```

---

**`Ctrl+Home` — jump to cell A1**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42   [52]

After `Ctrl+Home`:
     A     B     C     D     E
1  [10]   20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
```

---

## 3. Mode Transitions

| Key | Destination Mode | Edit buffer seeded with |
|---|---|---|
| `a` or `F2` | Visual | _(n/a — navigate within cell)_ |
| `i` | Insert | _(blank — replaces current content)_ |
| `=` | Insert | `=` (formula prefix already typed) |
| `Enter` | Insert | Current cell's raw content (for editing) |

**Visual mode** is a focused cell-navigation mode; the cursor is inside the cell and you can move between cells without entering text.

**Insert mode** shows a modal input overlay. Whatever you type replaces (or extends, when seeded) the cell's content. Confirm with `Enter`, cancel with `Esc`.

---

## 4. Cell and File Operations

### 4.1 Clear Current Cell

`Delete` or `Backspace` — clears the content of the cell under the cursor and leaves it empty. The cursor does not move. Row numbers are unaffected. No count accepted.

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52

After `Delete`:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [  ]   41    51
3   12    22    32    42    52
```

### 4.2 Save and Quit

| Key | Action |
|---|---|
| `s` or `S` | Save the current spreadsheet to the loaded CSV file path |
| `q` or `Q` | Quit the application |

---

## 5. Row-Axis Delete — `d`

`d` operates on the **row axis**. The axis of the motion determines what scope is cleared:

| Motion direction | What gets cleared |
|---|---|
| Horizontal (`h` `l` `w` `b` `0` `$`) | Individual cells in the **current row** |
| Vertical (`j` `k`) | **Entire rows** (all columns) |
| `dd` shorthand | **Entire current row** (all columns) |

**Nothing shifts** — row numbers never change. Cleared cells become empty. Deleted content is placed on the clipboard.

---

### 5.1 `dh` — Clear Cells to the Left

Clears `count` cells to the **LEFT** of the cursor, not including the cursor cell itself. The cursor moves to the first (leftmost) cleared cell.

Default count = 1.

---

**`dh` — clear 1 cell left (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52

After `dh`  (B2 cleared; cursor moves to B2):
     A     B     C     D     E
1   10    20    30    40    50
2   11   [  ]   31    41    51
3   12    22    32    42    52
```

---

**`2dh` — clear 2 cells left (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52

After `2dh`  (A2 and B2 cleared; cursor moves to A2):
     A     B     C     D     E
1   10    20    30    40    50
2  [  ]   ·     31    41    51
3   12    22    32    42    52
  (· = also empty)
```

---

**`3dh` — count exceeds available columns (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52

After `3dh`  (only A2, B2 exist left of C; clears both; cursor at A2):
     A     B     C     D     E
1   10    20    30    40    50
2  [  ]   ·     31    41    51
3   12    22    32    42    52
```

---

### 5.2 `dl` — Clear Cells to the Right

Clears `count` cells to the **RIGHT** of the cursor, not including the cursor cell. The cursor stays in place.

---

**`dl` — clear 1 cell right (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52

After `dl`  (D2 cleared; cursor stays at C2):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   ·     51
3   12    22    32    42    52
```

---

**`2dl` — clear 2 cells right (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52

After `2dl`  (D2 and E2 cleared; cursor stays at C2):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   ·     ·
3   12    22    32    42    52
```

---

**`d$` — clear from cursor to end of row (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52

After `d$`  (C2 through last column cleared; cursor stays at C2):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [  ]   ·     ·
3   12    22    32    42    52
```

---

### 5.3 `dj` — Clear Rows Downward

Clears the **current row** AND `count` rows below it. Total rows cleared = count + 1. The cursor stays on the current row.

---

**`dj` — clear current row + 1 row below (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24    34    44    54

After `dj`  (rows 2 and 3 fully cleared; cursor stays at C2):
     A     B     C     D     E
1   10    20    30    40    50
2   ·     ·    [  ]   ·     ·
3   ·     ·     ·     ·     ·
4   13    23    33    43    53
5   14    24    34    44    54
```

---

**`d2j` — clear current row + 2 rows below (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24    34    44    54

After `d2j`  (rows 2, 3, and 4 fully cleared; cursor stays at C2):
     A     B     C     D     E
1   10    20    30    40    50
2   ·     ·    [  ]   ·     ·
3   ·     ·     ·     ·     ·
4   ·     ·     ·     ·     ·
5   14    24    34    44    54
```

---

**`d3j` — clear 4 rows (cursor at C2)**

Rows 2, 3, 4, and 5 are cleared. Cursor stays at C2.

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24    34    44    54
6   15    25    35    45    55

After `d3j`  (rows 2–5 cleared):
     A     B     C     D     E
1   10    20    30    40    50
2   ·     ·    [  ]   ·     ·
3   ·     ·     ·     ·     ·
4   ·     ·     ·     ·     ·
5   ·     ·     ·     ·     ·
6   15    25    35    45    55
```

---

### 5.4 `dk` — Clear Rows Upward

Clears `count` rows **above** the cursor AND the current row. The cursor moves to the first (topmost) cleared row.

---

**`dk` — clear 1 row above + current row (cursor at C3)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22   [32]   42    52
4   13    23    33    43    53

After `dk`  (rows 2 and 3 cleared; cursor moves to C2):
     A     B     C     D     E
1   10    20    30    40    50
2   ·     ·    [  ]   ·     ·
3   ·     ·     ·     ·     ·
4   13    23    33    43    53
```

---

**`d2k` — clear 2 rows above + current row (cursor at C3)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22   [32]   42    52
4   13    23    33    43    53

After `d2k`  (rows 1, 2, and 3 cleared; cursor moves to C1):
     A     B     C     D     E
1   ·     ·    [  ]   ·     ·
2   ·     ·     ·     ·     ·
3   ·     ·     ·     ·     ·
4   13    23    33    43    53
```

---

### 5.5 `dd` — Clear Entire Current Row

Clears all cells in the current row. With a count `N`, clears N consecutive rows starting at the cursor row. The cursor stays on the starting row.

---

**`dd` — clear 1 row (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52

After `dd`  (row 2 fully cleared):
     A     B     C     D     E
1   10    20    30    40    50
2   ·     ·    [  ]   ·     ·
3   12    22    32    42    52
```

---

**`2dd` — clear 2 rows (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53

After `2dd`  (rows 2 and 3 cleared):
     A     B     C     D     E
1   10    20    30    40    50
2   ·     ·    [  ]   ·     ·
3   ·     ·     ·     ·     ·
4   13    23    33    43    53
```

---

**`3dd` — clear 3 rows (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24    34    44    54

After `3dd`  (rows 2, 3, and 4 cleared):
     A     B     C     D     E
1   10    20    30    40    50
2   ·     ·    [  ]   ·     ·
3   ·     ·     ·     ·     ·
4   ·     ·     ·     ·     ·
5   14    24    34    44    54
```

**Note — `2dd` vs `dj`:** Both clear 2 rows. The semantic difference is that `dd` counts rows directly (2dd = 2 rows), while `dj` clears the current row plus the motion target (dj = current row + 1 below = 2 rows). They produce identical results in this case; `Ndd` always equals `d(N-1)j`.

---

### 5.6 `d0` — Clear to Start of Row

Clears from **column A** up to (but not including) the cursor cell. The cursor moves to column A. If the cursor is already at column A, this is a no-op.

---

**`d0` — cursor at C2**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52

After `d0`  (A2 and B2 cleared; cursor moves to A2):
     A     B     C     D     E
1   10    20    30    40    50
2  [  ]   ·     31    41    51
3   12    22    32    42    52
```

---

**`d0` — cursor already at A2 (no-op)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2  [11]   21    31    41    51
3   12    22    32    42    52

After `d0`  (nothing to clear left of A; no change):
     A     B     C     D     E
1   10    20    30    40    50
2  [11]   21    31    41    51
3   12    22    32    42    52
```

---

### 5.7 `d$` — Clear to End of Row

Clears from the cursor cell (inclusive) to the last column of the row. The cursor stays in place.

---

**`d$` — cursor at C2**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52

After `d$`  (C2, D2, E2, … cleared; cursor stays at C2):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [  ]   ·     ·
3   12    22    32    42    52
```

---

### 5.8 Count Combinations

The grammar `[count1] d [count2] motion` multiplies the two counts:

```
effective_count = count1 × count2
```

---

**`2d3h` — clear 6 cells to the left (cursor at G2)**

count1=2, count2=3, effective=6. Cells F2, E2, D2, C2, B2, A2 are cleared. Cursor moves to A2.

```
Before:
     A     B     C     D     E     F     G
1   10    20    30    40    50    60    70
2   11    21    31    41    51    61   [71]
3   12    22    32    42    52    62    72

After `2d3h`  (6 cells left cleared; cursor moves to A2):
     A     B     C     D     E     F     G
1   10    20    30    40    50    60    70
2  [  ]   ·     ·     ·     ·     ·     71
3   12    22    32    42    52    62    72
```

---

**`2d3j` — clear 7 rows downward (cursor at C2)**

count1=2, count2=3, effective=6. Clears current row + 6 rows below = 7 rows total (rows 2–8).

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24    34    44    54
6   15    25    35    45    55
7   16    26    36    46    56
8   17    27    37    47    57
9   18    28    38    48    58

After `2d3j`  (rows 2–8 cleared; cursor stays at C2):
     A     B     C     D     E
1   10    20    30    40    50
2   ·     ·    [  ]   ·     ·
3   ·     ·     ·     ·     ·
4   ·     ·     ·     ·     ·
5   ·     ·     ·     ·     ·
6   ·     ·     ·     ·     ·
7   ·     ·     ·     ·     ·
8   ·     ·     ·     ·     ·
9   18    28    38    48    58
```

---

## 6. Column-Axis Delete — `gd`

`gd` is the **column-axis mirror** of `d`. The motion direction determines scope:

| Motion direction | What gets cleared |
|---|---|
| Vertical (`j` `k`) | Individual cells in the **current column** |
| Horizontal (`h` `l` `w` `b`) | **Entire columns** (all rows) |
| `gdd` shorthand | **Entire current column** (all rows) |

Nothing shifts. Row numbers never change. Cleared content is placed on the clipboard.

---

### 6.1 `gdj` — Clear Cells Below

Clears `count` cells **below** the cursor in the current column, not including the cursor cell. The cursor stays in place.

---

**`gdj` — clear 1 cell below (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24    34    44    54

After `gdj`  (C3 cleared; cursor stays at C2):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    ·     42    52
4   13    23    33    43    53
5   14    24    34    44    54
```

---

**`2gdj` — clear 2 cells below (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24    34    44    54

After `2gdj`  (C3 and C4 cleared; cursor stays at C2):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    ·     42    52
4   13    23    ·     43    53
5   14    24    34    44    54
```

---

**`3gdj` — clear 3 cells below (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24    34    44    54

After `3gdj`  (C3, C4, C5 cleared; cursor stays at C2):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    ·     42    52
4   13    23    ·     43    53
5   14    24    ·     44    54
```

---

### 6.2 `gdk` — Clear Cells Above

Clears `count` cells **above** the cursor in the current column, not including the cursor cell. The cursor moves to the first (topmost) cleared cell.

---

**`gdk` — clear 1 cell above (cursor at C4)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23   [33]   43    53
5   14    24    34    44    54

After `gdk`  (C3 cleared; cursor moves to C3):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22   [  ]   42    52
4   13    23    33    43    53
5   14    24    34    44    54
```

---

**`2gdk` — clear 2 cells above (cursor at C4)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23   [33]   43    53
5   14    24    34    44    54

After `2gdk`  (C2 and C3 cleared; cursor moves to C2):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [  ]   41    51
3   12    22    ·     42    52
4   13    23    33    43    53
5   14    24    34    44    54
```

---

**`3gdk` — clear 3 cells above (cursor at C4)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23   [33]   43    53
5   14    24    34    44    54

After `3gdk`  (C1, C2, C3 cleared; cursor moves to C1):
     A     B     C     D     E
1   10    20   [  ]   40    50
2   11    21    ·     41    51
3   12    22    ·     42    52
4   13    23    33    43    53
5   14    24    34    44    54
```

---

### 6.3 `gdl` — Clear Entire Columns to the Right

Clears `count` entire columns to the **right** of the cursor (all rows in those columns). The cursor stays in place.

---

**`gdl` — clear 1 column right (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53

After `gdl`  (entire column D cleared; cursor stays at C2):
     A     B     C     D     E
1   10    20    30    ·     50
2   11    21   [31]   ·     51
3   12    22    32    ·     52
4   13    23    33    ·     53
```

---

**`2gdl` — clear 2 columns right (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53

After `2gdl`  (entire columns D and E cleared; cursor stays at C2):
     A     B     C     D     E
1   10    20    30    ·     ·
2   11    21   [31]   ·     ·
3   12    22    32    ·     ·
4   13    23    33    ·     ·
```

---

### 6.4 `gdh` — Clear Entire Columns to the Left

Clears `count` entire columns to the **left** of the cursor (all rows in those columns). The cursor moves to the leftmost cleared column (same row).

---

**`gdh` — clear 1 column left (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53

After `gdh`  (entire column B cleared; cursor moves to B2):
     A     B     C     D     E
1   10    ·     30    40    50
2   11   [  ]   31    41    51
3   12    ·     32    42    52
4   13    ·     33    43    53
```

---

**`2gdh` — clear 2 columns left (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53

After `2gdh`  (entire columns A and B cleared; cursor moves to A2):
     A     B     C     D     E
1   ·     ·     30    40    50
2  [  ]   ·     31    41    51
3   ·     ·     32    42    52
4   ·     ·     33    43    53
```

---

### 6.5 `gdd` — Clear Entire Current Column

Clears **all rows** of the current column. With a count `N`, clears N consecutive columns starting at the cursor column. The cursor stays in place.

---

**`gdd` — clear 1 column (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53

After `gdd`  (entire column C cleared):
     A     B     C     D     E
1   10    20    ·     40    50
2   11    21   [  ]   41    51
3   12    22    ·     42    52
4   13    23    ·     43    53
```

---

**`2gdd` — clear 2 columns (cursor at C2)**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53

After `2gdd`  (entire columns C and D cleared):
     A     B     C     D     E
1   10    20    ·     ·     50
2   11    21   [  ]   ·     51
3   12    22    ·     ·     52
4   13    23    ·     ·     53
```

---

### 6.6 `gd$` — Clear Column to Bottom

Clears the current column from the **cursor row** to the last row (inclusive). The cursor stays in place.

---

**`gd$` — cursor at C2**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24    34    44    54

After `gd$`  (C2 through C_last cleared; cursor stays at C2):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [  ]   41    51
3   12    22    ·     42    52
4   13    23    ·     43    53
5   14    24    ·     44    54
```

---

### 6.7 `gd0` — Clear Column to Top

Clears the current column from **row 1** up to and including the cursor row. The cursor moves to row 1 of the same column.

---

**`gd0` — cursor at C4**

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23   [33]   43    53
5   14    24    34    44    54

After `gd0`  (C1 through C4 cleared; cursor moves to C1):
     A     B     C     D     E
1   10    20   [  ]   40    50
2   11    21    ·     41    51
3   12    22    ·     42    52
4   13    23    ·     43    53
5   14    24    34    44    54
```

---

### 6.8 Contrast: `d` vs `gd`

| Command | Axis | Motion type | Scope cleared |
|---|---|---|---|
| `dh` / `dl` | Row | Horizontal | Individual cells in the current row |
| `dj` / `dk` | Row | Vertical | Entire rows (all columns) |
| `dd` | Row | _(shorthand)_ | Entire current row(s) |
| `d0` / `d$` | Row | Boundary | Cells from A / to end in current row |
| `gdj` / `gdk` | Column | Vertical | Individual cells in the current column |
| `gdh` / `gdl` | Column | Horizontal | Entire columns (all rows) |
| `gdd` | Column | _(shorthand)_ | Entire current column(s) |
| `gd0` / `gd$` | Column | Boundary | Cells from row 1 / to last row in current column |

The key insight: **`d` + horizontal motion = cell scope; `d` + vertical motion = row scope.** For `gd` this is flipped: **`gd` + vertical motion = cell scope; `gd` + horizontal motion = column scope.**

---

## 7. Yank — `y`

Yank copies content to the clipboard **without clearing** it. The motion grammar is identical to `d`: `[count1] y [count2] motion`. Content remains in the grid unchanged.

| Command | Clipboard type | Shape |
|---------|---------------|-------|
| `yh`, `yl`, `y0`, `y$` | `Cells` | Horizontal strip |
| `yj`, `yk` | `Column` | Vertical strip |
| `yy`, `Nyy` | `Rows` | Full-row 2D grid |

---

### 7.1 `yh` / `yl` — Yank Cells Left / Right

Copies cells without moving the cursor or altering the grid.

---

**`yh` — yank 1 cell to the left (cursor at C2)**

Copies B2 to clipboard. Grid unchanged. Cursor stays at C2.

```
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51   ← copies B2 (value: 21)
3   12    22    32    42    52
```

---

**`2yl` — yank 2 cells to the right (cursor at C2)**

Copies D2 and E2 to clipboard. Grid unchanged.

```
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51   ← copies D2, E2 (values: 41, 51)
3   12    22    32    42    52
```

---

### 7.2 `yj` — Yank Cells Downward in Current Column

Yanks `count` cells starting at the cursor in the current column (cursor cell + count−1 cells below). Produces a **`Column`** clipboard.

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24    34    44    54

yj  (count=1) → clipboard Column: ["31"]              (cursor cell only)
y2j (count=2) → clipboard Column: ["31", "32"]        (cursor + 1 below)
y3j (count=3) → clipboard Column: ["31", "32", "33"]  (cursor + 2 below)
```

---

### 7.3 `yk` — Yank Cells Upward in Current Column

Yanks `count` cells ending at the cursor in the current column (count−1 cells above + cursor cell). Produces a **`Column`** clipboard.

```
Before (cursor at C4):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23   [33]   43    53
5   14    24    34    44    54

yk  (count=1) → clipboard Column: ["33"]              (cursor cell only)
y2k (count=2) → clipboard Column: ["32", "33"]        (1 above + cursor)
y3k (count=3) → clipboard Column: ["31", "32", "33"]  (2 above + cursor)
```

> Note: `yj` and `yk` produce a `Column` clipboard, not a `Rows` clipboard. Pasting restores to a single column, not entire rows.

---

### 7.4 `yy` — Yank Entire Current Row

Copies all cells of the current row. With a count, copies N consecutive rows.

`yy` also records the **cursor column as `col_offset`**, which is used by `gp`/`gP` to determine where to begin pasting in the target row (see [§9.3](#93-row-clipboard-with-col_offset)).

---

**`yy` — yank current row (cursor at C2, col_offset=2)**

```
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51   ← entire row 2 copied; col_offset = C (index 2)
3   12    22    32    42    52
```

---

**`2yy` — yank 2 rows (cursor at C2)**

Copies rows 2 and 3. col_offset=2 (column C).

```
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51   ← row 2 copied
3   12    22    32    42    52   ← row 3 copied
4   13    23    33    43    53
```

---

### 7.5 `y$` / `y0`

---

**`y$` — yank from cursor to end of row (cursor at C2)**

Copies C2, D2, E2, … to the clipboard. Grid unchanged.

```
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51   ← copies C2, D2, E2
3   12    22    32    42    52
```

---

**`y0` — yank from start of row to left of cursor (cursor at C2)**

Copies A2, B2 to the clipboard. Grid unchanged.

```
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51   ← copies A2, B2
3   12    22    32    42    52
```

---

## 8. Paste — `p` and `P`

Paste **inserts** clipboard content into the grid, **shifting** existing content to make room. Two flavours:

- `p` — paste **after** the cursor position (cells) or **below** the cursor row (rows)
- `P` — paste **at** the cursor position (cells) or **at** the cursor row (rows)

A count `N` pastes the clipboard content N times in sequence.

---

### 8.1 Cell Clipboard (`p` / `P`)

| Command | Behaviour |
|---|---|
| `p` | Inserts clipboard cells starting at cursor+1; existing cells shift right |
| `P` | Inserts clipboard cells starting at cursor; existing cells shift right |

---

### 8.2 Row Clipboard (`p` / `P`)

| Command | Behaviour |
|---|---|
| `p` | Inserts clipboard rows starting at cursor_row+1; rows below shift downward |
| `P` | Inserts clipboard rows starting at cursor_row; rows below shift downward |

---

### 8.3 Column Clipboard (`p` / `P`)

For a `Column` clipboard, paste always **overwrites** — there is no vertical shift.

| Command | Behaviour |
|---|---|
| `p` | Overwrites current column starting at cursor_row+1 (one below cursor) |
| `P` | Overwrites current column starting at cursor_row (at cursor) |

Count works: `2p` pastes the column strip twice consecutively downward.

---

**Scenario: `y3j` from C2, then paste with cursor at C5**

Clipboard (Column): `["31", "32", "33"]`

```
Before pasting (cursor at C5):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24   [34]   44    54
6   15    25    35    45    55
7   16    26    36    46    56

After P (overwrite at cursor, C5):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24   [31]   44    54   ← C5 overwritten
6   15    25    32    45    55   ← C6 overwritten
7   16    26    33    46    56   ← C7 overwritten

After p (overwrite at cursor+1, C6):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24   [34]   44    54   ← C5 unchanged
6   15    25    31    45    55   ← C6 overwritten
7   16    26    32    46    56   ← C7 overwritten
8   17    27    33    47    57   ← C8 overwritten
```

> Note: for Column clipboard, `gp`/`gP` behave identically to `p`/`P`.

---

### 8.4 Count

`2p` pastes the clipboard content twice consecutively (cells appear doubled, rows are duplicated one after another, or column strips are written consecutively downward).

---

### 8.5 Examples

**Scenario: yank row 2, then paste below row 5**

Step 1 — `yy` with cursor at C2:

```
     A     B     C     D     E
1   10    20    30    40    50
2   11    21   [31]   41    51   ← yy: row 2 copied (col_offset=2)
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24    34    44    54
6   15    25    35    45    55
```

Step 2 — move cursor to row 5, press `p`:

```
Before `p` (cursor at C5):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24   [34]   44    54
6   15    25    35    45    55

After `p`  (row 2's content inserted at row 6; old row 6 shifts to row 7):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24   [34]   44    54
6   11    21    31    41    51   ← inserted (copy of original row 2)
7   15    25    35    45    55   ← shifted down
```

---

**`P` — paste at current row (same clipboard, cursor at C5)**

```
Before `P` (cursor at C5):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24   [34]   44    54
6   15    25    35    45    55

After `P`  (row 2's content inserted at row 5; old rows 5+ shift down):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   11    21   [31]   41    51   ← inserted (copy of original row 2)
6   14    24    34    44    54   ← shifted down
7   15    25    35    45    55   ← shifted down
```

---

## 9. Overwrite Paste — `gp` and `gP`

Overwrite paste writes clipboard content **over** existing cells without shifting anything. Row numbers and all other cells remain unchanged.

- `gp` — overwrite starting at **cursor+1** (or row below for row clipboard)
- `gP` — overwrite starting at **cursor** (or current row for row clipboard)

A count `N` writes the clipboard content N times consecutively.

---

### 9.1 Cell Clipboard

---

**`gP` — overwrite at cursor (cursor at B2)**

Clipboard: `["X", "Y"]`

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11   [21]   31    41    51
3   12    22    32    42    52

After `gP`  (B2="X", C2="Y"; nothing shifts):
     A     B     C     D     E
1   10    20    30    40    50
2   11   [ X]    Y    41    51
3   12    22    32    42    52
```

---

**`gp` — overwrite after cursor (cursor at B2)**

Clipboard: `["X", "Y"]`

```
Before:
     A     B     C     D     E
1   10    20    30    40    50
2   11   [21]   31    41    51
3   12    22    32    42    52

After `gp`  (C2="X", D2="Y"; nothing shifts):
     A     B     C     D     E
1   10    20    30    40    50
2   11   [21]    X     Y    51
3   12    22    32    42    52
```

---

### 9.2 Row Clipboard (with `col_offset`)

When a row is yanked with `yy`, the yank records the cursor column as **`col_offset`**. On `gP`/`gp`, the content is written starting at that column offset of the target row — not necessarily at column A.

This is the key difference from `p`/`P` (which always writes to column A for row clipboards):

> `yy` at column C → `col_offset = 2` → `gP` on row 5 writes starting at column C of row 5.

---

**Scenario: `yy` at C3, then `gP` on row 5**

Step 1 — `yy` with cursor at C3 (col_offset=2):

```
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22   [32]   42    52   ← yy: row 3 copied, col_offset=C (2)
4   13    23    33    43    53
5   14    24    34    44    54
```

Step 2 — move to row 5, press `gP`:

```
Before `gP` (cursor at C5):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24   [34]   44    54

After `gP`  (row 3's content written into row 5 starting at col_offset=C; nothing shifts):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24   [32]   42    52   ← overwritten from col C onward
```

Notice: A5 and B5 (columns before the offset) are **untouched**. Only C5 onward is overwritten.

---

**`gp` — overwrite row below (cursor at C5)**

Using the same clipboard (row 3, col_offset=2):

```
Before `gp` (cursor at C5):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24   [34]   44    54
6   15    25    35    45    55

After `gp`  (row 3's content written into row 6 starting at col C; nothing shifts):
     A     B     C     D     E
1   10    20    30    40    50
2   11    21    31    41    51
3   12    22    32    42    52
4   13    23    33    43    53
5   14    24   [34]   44    54
6   15    25    32    42    52   ← overwritten from col C onward
```

---

### 9.3 Column Clipboard

> **Column clipboard**: For a `Column` clipboard (produced by `yj`/`yk`), `gp`/`gP` behave identically to `p`/`P` — both overwrite without shifting. The distinction between `gp`/`gP` and `p`/`P` only matters for `Cells` and `Rows` clipboard types.

---

### 9.4 Count

`2gp` writes the clipboard content twice consecutively (the second write begins immediately after the first ends, honouring `col_offset` each time).

---

## 10. The `g`-Prefix

`g` is rSheet's general **modifier prefix** for alternate command variants. Pressing `g` alone places the editor in `GPrefix` state; the status bar shows `g`. A second keypress resolves the command.

| Keys | Command |
|---|---|
| `gp` | Overwrite paste after cursor |
| `gP` | Overwrite paste at cursor |
| `gd` + motion | Column-axis delete (cell or column scope) |
| `gdd` | Clear entire current column |

The status bar progression:

```
(nothing) → g  → status: "g"
g → d          → status: "gd"   (GdOperatorPending)
gd → j         → executes gdj, returns to Idle
gd → d         → executes gdd (shorthand), returns to Idle
```

Pressing `Esc` at any `g`-prefix state cancels and returns to Idle.

---

## 11. Quick Reference

### Navigation

| Key(s) | Count | Action |
|---|---|---|
| `h` / Left | Yes | Move one cell left |
| `l` / Right | Yes | Move one cell right |
| `j` / Down | Yes | Move one cell down |
| `k` / Up | Yes | Move one cell up |
| `w` | Yes | Move one cell right (alias for `l`) |
| `b` | Yes | Move one cell left (alias for `h`) |
| `0` / `Home` | No | Jump to column A of current row |
| `$` | No | Jump to last column of current row |
| `Shift+Home` | No | Jump to row 1 of current column |
| `Ctrl+Home` | No | Jump to cell A1 |

### Mode Transitions

| Key | Destination | Edit buffer |
|---|---|---|
| `a` / `F2` | Visual | — |
| `i` | Insert | Blank |
| `=` | Insert | Seeded with `=` |
| `Enter` | Insert | Current raw content |

### Cell and File Operations

| Key | Action |
|---|---|
| `Delete` / `Backspace` | Clear current cell |
| `s` / `S` | Save to file |
| `q` / `Q` | Quit |

### Row-Axis Delete (`d`)

| Command | Count | Clears | Cursor after |
|---|---|---|---|
| `dh` | Yes (cells left) | N cells left of cursor in current row | Leftmost cleared cell |
| `dl` | Yes (cells right) | N cells right of cursor in current row | Stays |
| `dj` | Yes (extra rows) | Current row + N rows below | Stays |
| `dk` | Yes (rows above) | N rows above + current row | First cleared row |
| `dd` | Yes (rows) | N rows starting at cursor row | Stays |
| `d0` | No | Column A to cell left of cursor | Column A |
| `d$` | No | Cursor cell to end of row | Stays |
| `dw` | Yes | Alias for `dl` | Stays |
| `db` | Yes | Alias for `dh` | Leftmost cleared |

### Column-Axis Delete (`gd`)

| Command | Count | Clears | Cursor after |
|---|---|---|---|
| `gdj` | Yes (cells below) | N cells below cursor in current column | Stays |
| `gdk` | Yes (cells above) | N cells above cursor in current column | Topmost cleared cell |
| `gdl` | Yes (columns right) | N entire columns to the right | Stays |
| `gdh` | Yes (columns left) | N entire columns to the left | Leftmost cleared column |
| `gdd` | Yes (columns) | N entire columns starting at cursor | Stays |
| `gd0` | No | Current column from row 1 to cursor row | Row 1 |
| `gd$` | No | Current column from cursor row to last row | Stays |

### Yank (`y`)

| Command | Count | Copies |
|---|---|---|
| `yh` | Yes | N cells left of cursor |
| `yl` | Yes | N cells right of cursor |
| `yj` | Yes (N cells) | N cells downward from cursor in current column (`Column` clipboard) |
| `yk` | Yes (N cells) | N cells upward ending at cursor in current column (`Column` clipboard) |
| `yy` | Yes (rows) | N rows starting at cursor (col_offset=cursor col) |
| `y0` | No | Column A to cell left of cursor |
| `y$` | No | Cursor cell to end of row |

### Paste

| Command | Count | Clipboard | Behaviour |
|---|---|---|---|
| `p` | Yes | Cells | Insert after cursor, shift right |
| `P` | Yes | Cells | Insert at cursor, shift right |
| `p` | Yes | Rows | Insert below cursor row, shift rows down |
| `P` | Yes | Rows | Insert at cursor row, shift rows down |
| `p` | Yes | Column | Overwrite current column from cursor+1 downward |
| `P` | Yes | Column | Overwrite current column from cursor downward |
| `gp` | Yes | Cells | Overwrite at cursor+1, no shift |
| `gP` | Yes | Cells | Overwrite at cursor, no shift |
| `gp` | Yes | Rows | Overwrite row below at col_offset, no shift |
| `gP` | Yes | Rows | Overwrite current row at col_offset, no shift |
| `gp` | Yes | Column | Same as `p` for Column clipboard |
| `gP` | Yes | Column | Same as `P` for Column clipboard |

### Count Grammar Summary

```
[count1] operator [count2] motion   →   effective = count1 × count2
```

Examples:

| Input | Breakdown | Effective count | Result |
|---|---|---|---|
| `3j` | motion only | 3 | Move 3 rows down |
| `dj` | op + motion | 1 | Clear current row + 1 below |
| `d3j` | op + count + motion | 3 | Clear current row + 3 below (4 total) |
| `2dj` | count + op + motion | 2 | Clear current row + 2 below (3 total) |
| `2d3j` | count + op + count + motion | 6 | Clear current row + 6 below (7 total) |
| `2dd` | count + op shorthand | 2 | Clear 2 rows |
| `3yy` | count + op shorthand | 3 | Yank 3 rows |
