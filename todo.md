# TODO

## Next

### Normal mode command fixes

#### general refactors
  -  Arrows should be treated exactly like hjkl when in normal mode. for example d + right arrow should be like d+l
  - ENTER should no longer go to edit and insert mode, it should be treated like a j (going down), so d + ENTER should be like d + j
  - '=' should clear normal mode and start insert mode in the current cell, erasing the current cell and adding a '=' as the first character (like what happens when you press = when in visual mode)

#### Better delete 
  - Delete should not remove a line, it should just make it empty. The line numbers should not change after a delete. 
  - Alternate delete with gd for vertical removals (keep d for horizontal removals). gor example gd1j removes one cell below  (vertically), while d1j removes an entire line below

#### better paste
    - Paste should have an alternate version, denominated by 'g'. Several commands will be in the future given an alternate version, and the g before a command is the idiomatic way of saying this for them
                  Key | Behaviour |
                |-----|-----------|
                | `p` | Insert after cursor (current behaviour) |
                | `P` | Insert before cursor (current behaviour) |
                | `gp` | Overwrite paste starting at cursor+1 (cells) / cursor row+1 (rows) |
                | `gP` | Overwrite paste starting at cursor (cells) / cursor row (rows)
  - Additionally, "yy" followed by paste should paste a line, but shifted depending on the horizontal value of the cursor. It should behave like 'yNl', where N is the amount of cells in the line
      - This is to fix the behaviour where you yank a line and when you paste it below and a few cells to the side, it is pasted on the same line as the cursor, but not starting at its column



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
