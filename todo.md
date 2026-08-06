# TODO

## Next



## New test scenarios

- Dynamic IRS calculation
- simple budget sheet
- stock analysis sheet

## Future

- new delete operator: D for line removal with updates to all the cells affected by this removal (if cell X1 depends on a cell on line R, and line R is removed, then cell X1 needs its formula updated(shortened))
- Operator to add a new empty line below, using a similar DAG and formula update logic as the operator D
- Visual mode shift selection
  - TODO! specify
- F3 to toggle values being shown or fomulas

- Cosmetic improvements : header with file name, and program name 
- merge cells feature. this would force us to include some GUI comands in the csv so that the view changes

- language support
- support for AA, BB, AB, so longer lines than just until Z
- different tabs  Can ratatui tabs be used?
- graphs on a different tab.
- bar charts
- Split terminal into multiple panes, so that tabs can be shown side by side, moved around
- Dynamic variables, using a VARIABLE_NAME=EXPRESSION syntax, and then using VARIABLE_NAME in formulas. This would allow for more complex formulas and easier updates to multiple cells that depend on the same variable.
- Some codename for each cell, for example "=sum(A1:SELF)" would sum all the cells in the same column as the current cell, from A1 to the current cell. This would allow for more dynamic formulas that can adapt to changes in the spreadsheet structure.
