# TODO

## Next

- new feature: the value shown in a cell can be different than the csv value. For example
 - "= 1+1" should evaluate to 2 
 - How would wer implement this type of on the fly evaluation? Give me some idead. Brainstorm with me. I want the csv to save the string, but then  I want the grid to shwo the result of that operation, or ERROR if it makes no sense. The operations shoudl all start with a =  


## After

- more complex navigation 
  - HOME to go to start of line
  - END to go to end of line
  - CTRL + HOME to go to start of file
  - CTRL + END to go to end of file
  - Special mode after pressing ESC to go to special mode (esc again to leave it). 
    - vim like jumps, by pressing ESC to go to special mode, then 2j to jump 2 lines and so forth
    - cell jumping in special mode (for example A2  then enter to go cell A2. the letter needs to be capitalized for obvious reasons)

## Future

- Cosmetic improvements : header with file name, and program name 
- different tabs
- even more complex navigation
- graphs
- language support
