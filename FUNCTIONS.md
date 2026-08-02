# Supported Functions in rSheet

`rSheet` supports a rich set of built-in spreadsheet functions. Functions are case-insensitive and support scalar numbers, cell references (`A1`), cell ranges (`A1:A10`), multiple range arguments (`A1:A10, B1:B10`), nested function calls (`SUM(POW(2,3), SQRT(16))`), and arithmetic expressions inside function arguments.

---

## Cell Ranges

You can specify cell ranges using colon notation `START:END`.

- **Single range**: `=SUM(A1:A10)` calculates the sum of cells `A1` through `A10`.
- **2D range**: `=AVG(A1:C5)` calculates the average of all cells from `A1` to `C5`.
- **Multiple ranges**: `=SUM(A1:A10, B1:B10)` sums all cells in `A1:A10` and `B1:B10`.
- **Mixed arguments**: `=SUM(A1:A10, B1, 100, POW(2, 3))` combines ranges, cell references, literals, and sub-expressions.

---

## Function Reference

### Mathematical Functions

#### `POW(base, exponent)` / `POWER(base, exponent)`
Calculates `base` raised to the power of `exponent`.
- **Arguments**: 2 values or cell references
- **Example**: `=POW(2, 3)` returns `8`
- **Example**: `=POWER(A1, 2)` returns `A1` squared

#### `SQRT(number)`
Calculates the square root of a non-negative number.
- **Arguments**: 1 value or cell reference
- **Example**: `=SQRT(16)` returns `4`
- **Example**: `=SQRT(A1)` returns square root of cell `A1`

---

### Statistical / Aggregation Functions

#### `SUM(val1, val2, ...)`
Calculates the sum of all values in arguments and cell ranges.
- **Arguments**: Any number of values, cell references, or cell ranges
- **Example**: `=SUM(A1:A10)`
- **Example**: `=SUM(A1:A10, B1:B10)`

#### `AVG(val1, val2, ...)` / `AVERAGE(val1, val2, ...)`
Calculates the arithmetic mean of all arguments and cell ranges.
- **Arguments**: Any number of values, cell references, or cell ranges
- **Example**: `=AVG(A1:A10)`
- **Example**: `=AVERAGE(A1:A5, B1:B5)`

#### `MAX(val1, val2, ...)`
Finds the maximum value among all arguments and cell ranges.
- **Arguments**: Any number of values, cell references, or cell ranges
- **Example**: `=MAX(A1:A10)`

#### `MIN(val1, val2, ...)`
Finds the minimum value among all arguments and cell ranges.
- **Arguments**: Any number of values, cell references, or cell ranges
- **Example**: `=MIN(A1:A10)`

#### `COUNT(val1, val2, ...)`
Counts the number of numeric values in the arguments and cell ranges.
- **Arguments**: Any number of values, cell references, or cell ranges
- **Example**: `=COUNT(A1:A10)`
- **Example**: `=COUNT(A1:A10, B1:B10)`

#### `MEDIAN(val1, val2, ...)`
Calculates the median (middle value) of all numbers in the arguments and cell ranges.
- **Arguments**: Any number of values, cell references, or cell ranges
- **Example**: `=MEDIAN(A1:A5)`
- **Example**: `=MEDIAN(1, 3, 5, 7, 9)` returns `5`
