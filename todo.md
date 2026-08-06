# TODO

## Next

### Normal mode command fixes


#### Better delete 
  - Alternate delete with gd for vertical removals (keep d for horizontal removals). gor example gd1j removes one cell below  (vertically), while d1j removes an entire line below


### visual/insert mode fixes
- i for insert mode should keep the same buffer as was before. right now it erases the cell contents when entering a cell with i for insert mode
- F2 should enter insert mode on the current cell in normal and visual mode, like pressing i on normal mode


## Future

- new yank kind: Y for relative yank
- new delete operator: D for line removal with updates to all the cells affected by this removal (if cell X1 depends on a cell on line R, and line R is removed, then cell X1 needs its formula updated(shortened))
- Operator to add a new empty line below, using a similar DAG and formula update logic as the operator D
- Visual mode shift selection
- F3 to toggle values being shown or fomulas

- Cosmetic improvements : header with file name, and program name 
- merge cells feature. this would force us to include some GUI comands in the csv so that the view changes

- language support
- support for AA, BB, AB, so longer lines than just until Z
- different tabs  Can ratatui tabs be used?
- graphs on a different tab.
- bar charts
- Split terminal into multiple panes, so that tabs can be shown side by side, moved around
