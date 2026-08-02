# TODO

## Completed

✓ **Function Support** (Completed)
  - Added function parsing and evaluation to the expression engine
  - Implemented functions: POW, SUM, AVG, MAX, MIN
  - Functions integrate seamlessly with the DAG dependency tracker
  - Cell references in function arguments are automatically tracked as dependencies
  - Supports nested function calls and mixing functions with arithmetic
  - Circular dependency detection works through function calls
  - See FUNCTIONS.md for complete documentation

## Next

- more complex navigation 
  - HOME to go to start of line
  - END to go to end of line
  - CTRL + HOME to go to start of file
  - CTRL + END to go to end of file
  - Special mode after pressing ESC to go to special mode (esc again to leave it). 
    - vim like jumps, by pressing ESC to go to special mode, then 2j to jump 2 lines and so forth
    - cell jumping in special mode (for example A2  then enter to go cell A2. the letter needs to be capitalized for obvious reasons)
- Delete to delete the content of a cell
- edit mode should make a cell larger, it should be superimposed to the grid, and be made temporarily larger
- F3 to toggle values being shown or fomulas
- cursor capture to highlight cells, choose a cell. scroll right and left using shift + mouse wheel

## Future

- Cosmetic improvements : header with file name, and program name 
- merge cells feature. this would force us to include some GUI comands in the csv so that the view changes
- even more complex navigation
- language support
- support for AA, BB, AB, so longer lines than just until Z
- different tabs
- graphs on a different tab
