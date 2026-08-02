# TODO

## Now



## Next

- Special mode, similar to vim's special mode. It should triggered only in normal mode, by pressing ESC. It can be left by pressing ESC again.
  - All special mode commands work by typing a letter, then pressing enter, when in special mode
    - vim like jumps: 2j to jump 2 lines down, 2k to jump 2 lines up, 5l to jump 5 lines left, 5h to jump 5 lines right, if the jump goes out of bounds, it should saturate at the edge of the sheet
    - cell jumping in special mode: for example, A2 then enter to go to cell A2
    - d to delete the cell
    - dd to delete the line
    - dc to delete the column
    - More commands to come, we should just do the easy ones first

- END to go to last edited cell in line
- SHIFT + END to go to last cell edited in col.

## Future

- F3 to toggle values being shown or fomulas
- Cosmetic improvements : header with file name, and program name 
- merge cells feature. this would force us to include some GUI comands in the csv so that the view changes

- language support
- support for AA, BB, AB, so longer lines than just until Z
- different tabs  Can ratatui tabs be used?
- graphs on a different tab.
- bar charts
- Split terminal into multiple panes, so that tabs can be shown side by side, moved around


## Completed

✓ **Function Support** (Completed)
  - Added function parsing and evaluation to the expression engine
  - Implemented functions: POW, SUM, AVG, MAX, MIN
  - Functions integrate seamlessly with the DAG dependency tracker
  - Cell references in function arguments are automatically tracked as dependencies
  - Supports nested function calls and mixing functions with arithmetic
  - Circular dependency detection works through function calls
  - See FUNCTIONS.md for complete documentation

✓ **Centered Cell Edit Modal** (Completed)
  - Implemented floating edit modal dialog centered on the screen (`src/ui/modal.rs`)
  - Modal clears background grid, displays cell reference header (e.g. `Edit Cell [A1]`), input buffer, and control footer
  - Updated grid rendering so background spreadsheet stays intact with target cell highlighted
  - Updated key binding logic: `Enter`/`F2` commits changes, `Esc` cancels editing
  - Supports full multi-line wrapping and exact terminal cursor placement
