# TODO

## Now

- more complex navigation 
  - HOME to go to start of line when in normal mode
  - END to go to end of line when in normal mode
  - CTRL + HOME to go to start of file when in normal mode
  - CTRL + END to go to end of file when in normal mode
- More complex editing
  - Delete to delete the content of a cell, in edit mode and normal mode
  - "=" when in normal mode should delete the cell contents, start edit mode on the cell, and automatically add a = to the start of the cell 
  - Typing any number when in normal mode should delete the cell contents, start edit mode on the cell, and automatically add the number to the start of the cell


## Next

- cursor capture to highlight cells, choose a cell. scroll right and left using shift + mouse wheel

## Future

- F3 to toggle values being shown or fomulas
- Cosmetic improvements : header with file name, and program name 
- merge cells feature. this would force us to include some GUI comands in the csv so that the view changes
- even more complex navigation
    - Special mode after pressing ESC to go to special mode (esc again to leave it). 
      - vim like jumps, by pressing ESC to go to special mode, then 2j to jump 2 lines and so forth
      - cell jumping in special mode (for example A2  then enter to go cell A2. the letter needs to be capitalized for obvious reasons)
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
